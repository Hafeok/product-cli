//! The whole-store gate: every structural verdict, composed.
//!
//! One pass over everything the flow has filed. The verbs call the same
//! judges before they write, so a rule cannot hold here while being softer at
//! the door — there is no second validation copy anywhere in this crate.

use crate::act::{Act, Rejection};
use crate::check::{self, Class, Finding};
use crate::inventory::Inventory;
use crate::metrics;
use crate::policy::{self, Policy};
use crate::policy_check::{self, PolicyFinding};
use crate::record::ActRecord;

/// Everything the flow has filed, plus what the last import observed.
#[derive(Debug, Clone, Default)]
pub struct SpecStore {
    pub records: Vec<ActRecord>,
    pub acts: Vec<Act>,
    pub rejections: Vec<Rejection>,
    pub inventory: Option<Inventory>,
    /// Every filed policy version. The one in force is derived from the
    /// supersession chain, never from id order.
    pub policy_versions: Vec<Policy>,
    /// Trusted public keys. Empty means signing is off for this repo.
    pub trust: Vec<crate::signing::TrustKey>,
}

impl SpecStore {
    /// Is this candidate still unreviewed?
    ///
    /// Ending a review session never converts the remainder into accepted or
    /// rejected — unreviewed stays unreviewed, the same rule as an unfilled
    /// slot reading *not stated*.
    pub fn is_unreviewed(&self, candidate_id: &str) -> bool {
        !self.rejections.iter().any(|r| r.candidate == candidate_id)
            && !self.acts.iter().any(|a| a.from_candidate.as_deref() == Some(candidate_id))
    }

    /// The policy in force, or `None` where a project has filed none.
    ///
    /// The default is structural only. A project with no policy gets verdicts
    /// on broken things and nothing else, because shipping default thresholds
    /// would presume a basis nobody stated.
    pub fn policy_in_force(&self) -> std::result::Result<Option<&Policy>, Vec<&Policy>> {
        policy::in_force(&self.policy_versions)
    }

    /// Acts claiming realisation at a given entry point.
    pub fn acts_at(&self, entry_point: &str) -> Vec<&Act> {
        self.acts.iter().filter(|a| a.realises(entry_point)).collect()
    }
}

/// Judge the whole store against the closed class set.
pub fn judge_store(store: &SpecStore) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(check::judge_store(&store.records));
    findings.extend(store.acts.iter().flat_map(judge_act));
    findings.extend(store.rejections.iter().flat_map(judge_rejection));
    findings.extend(judge_references(store));
    findings.extend(judge_drift(store));
    findings.extend(judge_policy(store));
    findings.extend(judge_claims(store));
    findings.extend(judge_signatures(store));
    findings
}

/// `S014` and `S015` over one signed thing.
///
/// Both classes are inert until the repo carries a trust root. That is the
/// adoption cost made honest: a team without keys set up can still run the
/// whole flow, and turning signing on is filing a key rather than flipping a
/// flag. What it is not is a per-record escape — once a key is trusted, every
/// act a principal owns needs one.
pub fn judge_signature(
    subject: &str,
    digest: &str,
    principal: &ledger_core::identity::Identity,
    signature: Option<&str>,
    trust: &[crate::signing::TrustKey],
) -> Vec<Finding> {
    judge_signature_at(subject, digest, principal, signature, trust, None)
}

/// [`judge_signature`], for an act with a known time.
///
/// An act performed before the repo trusted any key **could not** have been
/// signed, so it is not judged as unsigned. That is not a loophole a writer can
/// reach: the time is inside the digest, and adopting signing does not
/// retroactively invalidate a history nobody could have signed. What those
/// records have instead is `S003`/`S007` and the git history — stated as the
/// limit it is, rather than papered over by failing every old record at once.
pub fn judge_signature_at(
    subject: &str,
    digest: &str,
    principal: &ledger_core::identity::Identity,
    signature: Option<&str>,
    trust: &[crate::signing::TrustKey],
    acted_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Vec<Finding> {
    if trust.is_empty() {
        return Vec::new();
    }
    let predates_adoption = matches!(
        (acted_at, adopted_at(trust)),
        (Some(acted), Some(adopted)) if acted < adopted
    );
    match signature {
        // Absence is graced before adoption, and only absence. A signature
        // that is present is verified whenever it was written: the grace
        // exists because nobody could have signed, not because anything goes.
        None if predates_adoption => Vec::new(),
        None => vec![Finding::new(
            Class::S014,
            subject,
            "unsigned, and this repo trusts keys — a named principal is not a signed one",
        )],
        Some(signature) if !crate::signing::verifies(digest, signature, principal, trust) => {
            vec![Finding::new(
                Class::S015,
                subject,
                &format!(
                    "the signature does not verify under any key trusted for `{}`",
                    principal.as_str()
                ),
            )]
        }
        Some(_) => Vec::new(),
    }
}

/// When this repo started trusting keys: the earliest key's `added_at`.
fn adopted_at(trust: &[crate::signing::TrustKey]) -> Option<chrono::DateTime<chrono::Utc>> {
    trust.iter().map(|k| k.added_at).min()
}

/// `S014`/`S015` across every signed thing in the store.
///
/// Every subject is verified against its **recomputed** digest, never the one
/// it carries. Verifying the stored digest would check that the file agrees
/// with itself: an edited act keeps its old `binds`, so its old signature
/// would still verify and only `S007` would notice. Recomputing makes the
/// signature a guarantee on its own rather than one that leans on another
/// class being checked too.
pub fn judge_signatures(store: &SpecStore) -> Vec<Finding> {
    let mut findings = Vec::new();
    for record in &store.records {
        if let Some(closure) = &record.closure {
            findings.extend(judge_signature_at(
                &record.id,
                &crate::digest::closure_digest(&record.computed_binds(), closure),
                &closure.principal,
                closure.signature.as_deref(),
                &store.trust,
                Some(closure.at),
            ));
        }
    }
    for act in &store.acts {
        findings.extend(judge_signature_at(
            &act.id,
            &act.computed_binds(),
            &act.ratified_by,
            act.signature.as_deref(),
            &store.trust,
            Some(act.ratified_at),
        ));
    }
    for rejection in &store.rejections {
        findings.extend(judge_signature_at(
            &rejection.candidate,
            &rejection.computed_binds(),
            &rejection.principal,
            rejection.signature.as_deref(),
            &store.trust,
            Some(rejection.at),
        ));
    }
    findings
}

/// `S012` and `S013` — code claiming something the store does not carry.
///
/// The two attributes are references, and these are the classes that make them
/// worth writing: a `[Slice]` naming a slice nobody declared, or a
/// `[RealisesFact]` naming a determination no closure filed, is code asserting
/// a link to a specification that does not exist. It reads as governed and
/// is not, which is worse than being plainly ungoverned.
pub fn judge_claims(store: &SpecStore) -> Vec<Finding> {
    let Some(inventory) = &store.inventory else {
        return Vec::new();
    };
    let mut findings = Vec::new();

    for claim in inventory.claims_of("slice") {
        if !store.records.iter().any(|r| r.slice == claim.value) {
            findings.push(Finding::new(
                Class::S012,
                &claim.symbol,
                &format!(
                    "`[Slice(\"{}\")]` at {}:{} — no act-time record declares that slice",
                    claim.value, claim.file, claim.line
                ),
            ));
        }
    }

    for claim in inventory.claims_of("realises-fact") {
        if !filed_determinations(store).contains(&claim.value) {
            findings.push(Finding::new(
                Class::S013,
                &claim.symbol,
                &format!(
                    "`[RealisesFact(\"{}\")]` at {}:{} — no closure filed that determination",
                    claim.value, claim.file, claim.line
                ),
            ));
        }
    }

    findings
}

/// Every determination any closure filed.
fn filed_determinations(store: &SpecStore) -> std::collections::BTreeSet<String> {
    store
        .records
        .iter()
        .filter_map(|r| r.closure.as_ref())
        .flat_map(|c| c.determinations.iter().cloned())
        .collect()
}

/// `S008`–`S011` over the policy in force.
///
/// A forked policy chain is itself a finding: no ordering heuristic may pick
/// a side, so the caller is told rather than quietly given one of them.
pub fn judge_policy(store: &SpecStore) -> Vec<Finding> {
    match store.policy_in_force() {
        Ok(None) => Vec::new(),
        Ok(Some(policy)) => policy_check::judge_policy(policy, &metrics::compute(store)),
        Err(tips) => vec![Finding::new(
            Class::S008,
            "policy",
            &format!(
                "the policy chain forks at {} tips ({}) — only a recorded supersession settles it",
                tips.len(),
                tips.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", ")
            ),
        )],
    }
}

/// Run the project's own verdicts. Empty where no policy is in force.
pub fn run_policy(store: &SpecStore) -> Vec<PolicyFinding> {
    match store.policy_in_force() {
        Ok(Some(policy)) => policy_check::run(policy, &metrics::compute(store)),
        _ => Vec::new(),
    }
}

/// `S002` and `S007` over one ratified act.
pub fn judge_act(act: &Act) -> Vec<Finding> {
    let mut findings = Vec::new();
    if let Some(reason) = check::principal_refusal(&act.ratified_by) {
        findings.push(Finding::new(Class::S002, &act.id, &format!("ratified by {reason}")));
    }
    if act.binds != act.computed_binds() {
        findings.push(Finding::new(
            Class::S007,
            &act.id,
            "the act's binding does not match what it now says",
        ));
    }
    findings
}

/// `S002` and `S007` over one filed refusal.
pub fn judge_rejection(rejection: &Rejection) -> Vec<Finding> {
    let mut findings = Vec::new();
    if let Some(reason) = check::principal_refusal(&rejection.principal) {
        findings.push(Finding::new(
            Class::S002,
            &rejection.candidate,
            &format!("refused by {reason}"),
        ));
    }
    if rejection.binds != rejection.computed_binds() {
        findings.push(Finding::new(
            Class::S007,
            &rejection.candidate,
            "the refusal's binding does not match what it now says",
        ));
    }
    findings
}

/// `S006` — an act-time record pointing at an act that does not exist.
///
/// Only checked once at least one act is ratified. A store with no acts at
/// all is a repo mid-adoption, not a broken one, and failing every record in
/// it would make the first `implement` impossible to run.
pub fn judge_references(store: &SpecStore) -> Vec<Finding> {
    if store.acts.is_empty() {
        return Vec::new();
    }
    store
        .records
        .iter()
        .filter(|record| !store.acts.iter().any(|a| a.id == record.act_ref))
        .map(|record| {
            Finding::new(
                Class::S006,
                &record.id,
                &format!("names the act `{}`, which is not ratified", record.act_ref),
            )
        })
        .collect()
}

/// `S005` — an entry point no ratified act covers.
///
/// A refused candidate is covered: a principal looked at that entry point and
/// said it is not an act. That is a decision, and a decision is exactly what
/// this class exists to require. Silence is what it fails on.
pub fn judge_drift(store: &SpecStore) -> Vec<Finding> {
    let Some(inventory) = &store.inventory else {
        return Vec::new();
    };
    inventory
        .entry_points
        .iter()
        .filter(|entry| store.acts_at(&entry.id).is_empty())
        .filter(|entry| !refused(store, inventory, &entry.id))
        .map(|entry| {
            Finding::new(
                Class::S005,
                &entry.id,
                &format!(
                    "`{}` at {}:{} — no ratified act covers it",
                    entry.transport, entry.file, entry.line
                ),
            )
        })
        .collect()
}

/// Whether a principal refused the candidate that stands for this entry point.
fn refused(store: &SpecStore, inventory: &Inventory, entry_point: &str) -> bool {
    inventory
        .candidates
        .iter()
        .filter(|c| c.entry_point == entry_point)
        .any(|c| store.rejections.iter().any(|r| r.candidate == c.id))
}

#[path = "gate_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;

#[path = "gate_signing_tests.rs"]
#[cfg(test)]
mod signing_tests;
