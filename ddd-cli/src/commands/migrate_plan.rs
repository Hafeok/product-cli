//! Planning the M8 ledger migration — what each historical entry becomes.
//!
//! Pure mapping from the `.ddd` store to planned ledger entries. The
//! judgment calls in here are the principal's rulings, transcribed: the
//! three watched-not-grounding edges carry the explicit `watched:` marker
//! and are never migrated as ground; `dec/ddd/interceptor-not-extension`
//! migrates indeterminate as-is (its claim edge marked, not laundered);
//! basis pins upgrade to content hashes recovered at their pinned state.

use std::path::Path;

use ddd_core::basis_pin::{claim_content_hash, decision_content_hash, seam_content_hash};
use ddd_core::decision::{BasedOn, BasisPin, Decision, DecisionKind};
use ddd_core::store::DddStore;
use ledger_core::author::AllocationArgs;
use ledger_core::allocation::AllocationKind;
use ledger_core::discharge::{DischargeRef, Stage};
use ledger_core::identity::Identity;
use ledger_core::version::BasisRef;

/// The principal (owner of the migrated set; actor of judgment
/// allocations; the risk record's acceptor of record).
pub const PRINCIPAL: &str = "emk@delegate.dk";

/// The three watched-not-grounding edges from the re-typing session's
/// handoff: retained claim edges that never grounded their decision.
/// Migrated with the explicit `watched:` marker, never as ground; the
/// ontology question is filed alongside (see [`question_entry`]).
const WATCHED: &[(&str, &str)] = &[
    ("dec/ddd/internal-not-surface", "DDD-adapter-02"),
    ("dec/rust/no-unwrap", "DDD-gates-01"),
    ("dec/ddd/m6-proceeds-no-flip", "DDD-adapter-01"),
];

/// Audit F-7h: indeterminate at format 2, awaiting the principal's
/// ruling — its untyped claim edge migrates marked, not typed.
const INDETERMINATE: &[(&str, &str)] = &[("dec/ddd/interceptor-not-extension", "DDD-arch-03")];

/// The historical id minted for the filed ontology question.
pub const QUESTION_ID: &str = "question/ddd/watched-edge-kind";

/// One planned ledger entry.
pub struct Planned {
    pub historical: String,
    pub kind: &'static str,
    pub statement: String,
    pub allocation: AllocationArgs,
    pub based_on: Vec<BasisRef>,
    pub note: String,
}

/// Every entry the migration files, in deterministic order: decisions
/// (incl. the risk record), then seams, then the ontology question.
pub fn plan(root: &Path, store: &DddStore) -> Result<Vec<Planned>, String> {
    let principal: Identity = PRINCIPAL.parse()?;
    let mut decisions = store.decisions.clone();
    decisions.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for d in &decisions {
        out.push(plan_decision(root, store, d, &principal)?);
    }
    let mut seams = store.seams.clone();
    seams.sort_by(|a, b| a.id.cmp(&b.id));
    for s in &seams {
        out.push(plan_seam(s));
    }
    out.push(question_entry(&principal)?);
    Ok(out)
}

fn plan_decision(
    root: &Path,
    store: &DddStore,
    d: &Decision,
    principal: &Identity,
) -> Result<Planned, String> {
    let mut based_on = vec![token(format!(
        "ddd-content:{}@{}",
        d.id,
        decision_content_hash(d)
    ))?];
    let mut flags = Vec::new();
    for basis in &d.based_on {
        based_on.push(basis_token(root, store, d, basis, &mut flags)?);
    }
    let allocation = decision_allocation(store, d, principal)?;
    let mut note = format!(
        "Migrated from {} (ddd M8). The historical entry remains in .ddd as the pinned \
         content artifact — rationale, principal, typed bases — sealed by the ddd-content \
         token; this ledger entry is the record of the decision, allocated and awaiting \
         the principal's acceptance. The concordance maps both ids permanently.",
        d.id
    );
    for f in &flags {
        note.push('\n');
        note.push_str(f);
    }
    Ok(Planned {
        historical: d.id.clone(),
        kind: if d.kind == DecisionKind::RiskAcceptance { "risk-acceptance" } else { "decision" },
        statement: d.title.clone(),
        allocation,
        based_on,
        note,
    })
}

/// The basis token one historical edge migrates as. Claim pins upgrade
/// to content hashes at their pinned state; watched and indeterminate
/// edges carry their markers instead of the grounding `claim:` scheme.
fn basis_token(
    root: &Path,
    store: &DddStore,
    d: &Decision,
    basis: &BasedOn,
    flags: &mut Vec<String>,
) -> Result<BasisRef, String> {
    match basis {
        BasedOn::Plain(claim) => {
            flags.push(format!("Edge {claim} was unpinned (format 1) and migrates unpinned."));
            token(format!("claim:{claim}"))
        }
        BasedOn::Pinned(pin) => {
            let hash = pinned_claim_hash(root, store, pin)?;
            let scheme = if WATCHED.contains(&(d.id.as_str(), pin.claim.as_str())) {
                flags.push(format!(
                    "Edge {} is watched, not grounding (re-typing handoff): the ontology \
                     question is filed as {QUESTION_ID}; this edge must not be read as ground.",
                    pin.claim
                ));
                "watched"
            } else if INDETERMINATE.contains(&(d.id.as_str(), pin.claim.as_str())) {
                flags.push(format!(
                    "Basis indeterminate (audit F-7h): the claim edge {} migrates marked, \
                     not typed — awaiting the principal's ruling.",
                    pin.claim
                ));
                "indeterminate"
            } else {
                "claim"
            };
            token(format!("{scheme}:{}@{hash}", pin.claim))
        }
        BasedOn::Typed(t) => {
            let ty = serde_json::to_value(t.basis_type)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "basis".to_string());
            match &t.reference {
                Some(r) => token(format!("{ty}:{r}")),
                // The statement itself lives in the pinned content.
                None => token(format!("{ty}:{}", d.id)),
            }
        }
    }
}

/// The claim's content hash at the pin's recorded state. When the claim
/// has not moved, that is its current content; when the pin records
/// drift that was deliberately kept, the pinned state is recovered from
/// git history — never guessed.
fn pinned_claim_hash(root: &Path, store: &DddStore, pin: &BasisPin) -> Result<String, String> {
    let current = store
        .claims
        .iter()
        .find(|c| c.id == pin.claim)
        .ok_or_else(|| format!("pin names {}, which is not in the store", pin.claim))?;
    if current.status == pin.status && current.changed == pin.changed {
        return Ok(claim_content_hash(current));
    }
    let rel = format!(".ddd/claims/{}.yaml", pin.claim);
    let revs = git_lines(root, &["log", "--format=%H", "--", &rel])?;
    for rev in revs {
        let Some(bytes) = ddd_core::gitrev::bytes_at(root, &rev, &rel)? else { continue };
        let Ok(text) = String::from_utf8(bytes) else { continue };
        let Ok(claim) = serde_yaml::from_str::<ddd_core::claim::Claim>(&text) else { continue };
        if claim.status == pin.status && claim.changed == pin.changed {
            return Ok(claim_content_hash(&claim));
        }
    }
    Err(format!(
        "no committed state of {} matches its pin ({}@{}) — the pinned content cannot be \
         recovered and must not be guessed",
        pin.claim,
        pin.status.as_str(),
        pin.changed
    ))
}

/// Allocation per the migration mapping: a decision cited by manifest
/// entries is a constraint discharged by those rules; the risk record is
/// the priced escape it states; everything else is the principal's
/// judgment.
fn decision_allocation(
    store: &DddStore,
    d: &Decision,
    principal: &Identity,
) -> Result<AllocationArgs, String> {
    if d.kind == DecisionKind::RiskAcceptance {
        return Ok(AllocationArgs {
            store: Some(AllocationKind::Escaped),
            exposure: Some(
                "24 undeclared What boundaries over 121 classified elements (8 acme, 16 \
                 product-cli): changes to them go unrecorded and the What surface has no \
                 green baseline. Bounded: both graphs are demonstration content with no \
                 second party, and nothing enforces these boundaries in either direction."
                    .to_string(),
            ),
            accepted_by: Some(principal.clone()),
            review_by: Some(
                chrono::NaiveDate::from_ymd_opt(2027, 2, 7).ok_or("bad review date")?,
            ),
            ..Default::default()
        });
    }
    let mut rules: Vec<DischargeRef> = Vec::new();
    for set in &store.manifests {
        for rule in &set.file.rules {
            if rule.decision.as_deref() == Some(d.id.as_str()) {
                rules.push(format!("analyzer:{}/{}", set.name, rule.rule_id).parse()?);
            }
        }
    }
    if rules.is_empty() {
        return Ok(AllocationArgs {
            store: Some(AllocationKind::Judgment),
            actor: Some(principal.clone()),
            ..Default::default()
        });
    }
    Ok(AllocationArgs {
        store: Some(AllocationKind::Constraint),
        discharge: rules,
        ..Default::default()
    })
}

fn plan_seam(s: &ddd_core::seam::SeamDeclaration) -> Planned {
    let discharge: Vec<DischargeRef> = format!("contract:{}", s.id)
        .parse()
        .map(|d| vec![d])
        .unwrap_or_default();
    Planned {
        historical: s.id.clone(),
        kind: "seam",
        statement: s.boundary.clone(),
        allocation: AllocationArgs {
            store: Some(AllocationKind::Criterion),
            discharge,
            discharge_stage: Some(Stage::Pr),
            ..Default::default()
        },
        based_on: token(format!("ddd-content:{}@{}", s.id, seam_content_hash(s)))
            .map(|t| vec![t])
            .unwrap_or_default(),
        note: format!(
            "Migrated from {} (ddd M8). The declaration remains in .ddd (verdict knowledge, \
             obligations, signed bindings) sealed by the ddd-content token; its checker is \
             the repository-diff contract check (the contract: discharge, spec v1.4 / \
             format 3). Awaiting the principal's acceptance.",
            s.id
        ),
    }
}

/// The ontology question the watched edges await (filed, not decided by
/// this migration): filed as a decision allocated to the principal's
/// judgment, awaiting acceptance like every migrated entry.
fn question_entry(principal: &Identity) -> Result<Planned, String> {
    Ok(Planned {
        historical: QUESTION_ID.to_string(),
        kind: "question",
        statement: "A watched-not-grounding edge — a claim a decision tracks without resting \
                    on — is either a distinct edge kind in the ddd ontology or is retired at \
                    the F-batch; until ruled, migrated watched edges carry the explicit \
                    watched: marker and are never read as ground."
            .to_string(),
        allocation: AllocationArgs {
            store: Some(AllocationKind::Judgment),
            actor: Some(principal.clone()),
            ..Default::default()
        },
        based_on: vec![token(
            "handoff:basis-quality-retyping-2026-08-12#watched-not-grounding".to_string(),
        )?],
        note: format!(
            "Filed by the M8 migration (not decided by it): three migrated edges carry the \
             watched: marker — {} — and the marker's standing needs the principal's ruling.",
            WATCHED
                .iter()
                .map(|(d, c)| format!("{c} on {d}"))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    })
}

fn token(s: String) -> Result<BasisRef, String> {
    s.parse()
}

fn git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect())
}
