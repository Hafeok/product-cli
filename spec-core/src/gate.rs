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
    findings
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
mod tests;
