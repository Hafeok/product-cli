//! `ledger accept --group` — one invocation, one acceptance record per
//! decision.
//!
//! What is batched is the act. Each member still gets its own [`Acceptance`]
//! signing its own version hash, under the same [`AcceptanceScope::Version`]
//! a single accept uses: the binding is untouched, and every signature this
//! verb produces is exactly as bound as one typed on its own. There is no
//! shared record, no class scope, no hash ranging over members — a shortcut
//! that produced one record covering N decisions would be a different
//! meaning of acceptance wearing this verb's name.
//!
//! Three refusals stand between a selection and a signature, and none of
//! them is a batch-only leniency in the other direction. The selection must
//! still hash to the manifest a prior read enumerated; no member may be held
//! or forked; and every member runs the *same* delta-gate a single accept
//! runs — once alone, so a failure can be attributed, and once combined, so
//! the write is checked as the act it is.

use chrono::NaiveDate;

use crate::acceptance::{Acceptance, AcceptanceScope};
use crate::batch::{self, Outcome, Plan, Refusal};
use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::identity::Identity;
use crate::show::{screens, NoText, Selector};
use crate::store::Store;

use super::{Author, AuthorError};

/// What a batched acceptance covers, in one enumerated selection.
pub struct AcceptGroupArgs {
    /// The same selector syntax `ledger show` reads, so what was read is
    /// provably what is signed.
    pub selector: Selector,
    pub expires_at: Option<NaiveDate>,
    /// The manifest a prior read printed. Absent is the default posture:
    /// enumerate, print, write nothing.
    pub confirm: Option<String>,
}

impl Author {
    /// Enumerate the selection, and — only against a matching manifest —
    /// sign it.
    pub fn accept_group(&mut self, args: AcceptGroupArgs) -> Result<Outcome, AuthorError> {
        let store = self.load();
        let today = self.today();
        let plan = enumerate(&store, &args.selector, &self.who, today);
        if plan.members.is_empty() {
            return Err(AuthorError::Usage(format!(
                "{} matches no decision in this store — a selection of nothing is a misread selector, not an empty pass",
                plan.selector
            )));
        }
        let Some(confirmed) = args.confirm.clone() else {
            let refusal = blockers(&plan);
            return Ok(Outcome { plan, dry_run: true, signed: Vec::new(), filed: None, refusal });
        };
        if !batch::is_manifest_shaped(&confirmed) {
            return Err(AuthorError::Usage(format!(
                "`{confirmed}` is not a manifest — expected the `sha256:<64 hex>` digest a dry run prints"
            )));
        }
        if confirmed != plan.manifest {
            return Ok(drift(&store, &args.selector, plan, &confirmed, &self.who, today));
        }
        if let Some(refusal) = blockers(&plan) {
            return Ok(refuse(plan, refusal));
        }
        self.sign(&store, plan, args.expires_at)
    }

    /// File one change-set holding one acceptance per signable member.
    fn sign(
        &mut self,
        store: &Store,
        plan: Plan,
        expires_at: Option<NaiveDate>,
    ) -> Result<Outcome, AuthorError> {
        let mut candidate = self.shell(Some(note(&plan)))?;
        let mut signed: Vec<String> = Vec::new();
        for m in plan.signable() {
            candidate.acceptances.push(Acceptance {
                id: self.mint.mint_id("acc").map_err(AuthorError::Io)?,
                decision: m.decision.parse().map_err(AuthorError::Usage)?,
                version: m.version.parse().map_err(AuthorError::Usage)?,
                actor: self.who.clone(),
                at: self.now,
                scope: AcceptanceScope::Version,
                expires_at,
                signature: String::new(),
            });
            signed.push(m.decision.clone());
        }
        if signed.is_empty() {
            return Ok(refuse(plan, nothing_to_add()));
        }
        let baseline = self.gate(store);
        let detail = self.per_member(&baseline, store, &candidate);
        if !detail.is_empty() {
            return Ok(refuse(plan, gate_refusal(detail)));
        }
        match self.refusal_check_against(&baseline, store, &candidate, |_| false) {
            Ok(()) => {}
            Err(AuthorError::Refused(findings)) => {
                return Ok(refuse(plan, gate_refusal(lines(&findings))))
            }
            Err(other) => return Err(other),
        }
        let path = self.append(&candidate)?;
        Ok(Outcome { plan, dry_run: false, signed, filed: Some(path), refusal: None })
    }

    /// Gate each acceptance on its own, so a member that would fail as a
    /// single accept fails the batch with the same class, the same message,
    /// and its own decision named.
    fn per_member(&self, baseline: &[Finding], store: &Store, candidate: &ChangeSet) -> Vec<String> {
        let mut detail = Vec::new();
        for acceptance in &candidate.acceptances {
            let mut one = candidate.clone();
            one.acceptances = vec![acceptance.clone()];
            if let Err(AuthorError::Refused(findings)) =
                self.refusal_check_against(baseline, store, &one, |_| false)
            {
                for finding in &findings {
                    detail.push(format!("{} — {finding}", acceptance.decision));
                }
            }
        }
        detail
    }
}

/// The selection, read through `show` rather than a second traversal.
fn enumerate(store: &Store, selector: &Selector, who: &Identity, today: NaiveDate) -> Plan {
    batch::plan(&screens(store, selector, today, &NoText), selector, who, today)
}

fn note(plan: &Plan) -> String {
    format!(
        "batched acceptance over {} — {} record(s), each signing its own version hash, under manifest {}",
        plan.selector,
        plan.signable().count(),
        plan.manifest
    )
}

fn refuse(plan: Plan, refusal: Refusal) -> Outcome {
    Outcome { plan, dry_run: false, signed: Vec::new(), filed: None, refusal: Some(refusal) }
}

/// A member the record itself says is not ready. Naming it is the point:
/// a batch that signed around it would be making that judgment silently.
fn blockers(plan: &Plan) -> Option<Refusal> {
    let blocked = plan.blockers();
    if blocked.is_empty() {
        return None;
    }
    let held = blocked.iter().any(|m| m.standing == batch::Standing::Held.as_str());
    Some(Refusal {
        kind: if held { "held" } else { "forked" },
        message: format!(
            "{} member(s) of this selection are not for acceptance — the run stops on them rather than signing around them",
            blocked.len()
        ),
        detail: blocked
            .iter()
            .map(|m| format!("{} — {}", m.decision, m.because.clone().unwrap_or_default()))
            .collect(),
    })
}

fn drift(
    store: &Store,
    selector: &Selector,
    plan: Plan,
    confirmed: &str,
    who: &Identity,
    today: NaiveDate,
) -> Outcome {
    let detail = match batch::locate(store, selector, who, today, confirmed) {
        Some(before) => batch::changes(&before, &plan)
            .into_iter()
            .map(|c| format!("{} — {}", c.decision, c.what))
            .collect(),
        None => vec![
            "the manifest matches no state this log has passed through — it was read against a different selector, a different identity, or a store that has since been rewritten rather than appended to"
                .to_string(),
        ],
    };
    let message = format!(
        "the selection moved since it was read — you confirmed {confirmed}, this store now enumerates to {}",
        plan.manifest
    );
    refuse(plan, Refusal { kind: "drift", message, detail })
}

fn gate_refusal(detail: Vec<String>) -> Refusal {
    Refusal {
        kind: "gate",
        message:
            "a member would fail verify as a single accept, so it fails the batch — there is no batch-only leniency"
                .to_string(),
        detail,
    }
}

fn nothing_to_add() -> Refusal {
    Refusal {
        kind: "nothing",
        message:
            "every member of this selection already carries your live acceptance of its current hash — accepting again adds nothing"
                .to_string(),
        detail: Vec::new(),
    }
}

fn lines(findings: &[Finding]) -> Vec<String> {
    findings.iter().map(ToString::to_string).collect()
}
