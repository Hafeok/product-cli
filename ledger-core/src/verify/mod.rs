//! The gate — the whole of L0's checking surface.
//!
//! One pass over the log produces every finding: the parse gate, then the
//! ten classes. Failing for exactly those reasons is the format's measure of
//! success, so the orchestration here is deliberately flat — there is no
//! place for a rule to hide.
//!
//! Pendency is not a violation. A decision that is enumerated and allocated
//! but not yet signed is the normal state of work in progress; it is
//! reported as status, never as a failure, because a gate that fires on
//! ordinary work is a gate people learn to ignore.

pub mod disposition;
pub mod integrity;
pub mod state;
pub mod view;

use chrono::NaiveDate;
use serde::Serialize;

use crate::finding::{Finding, Gate};
use crate::store::Store;

use view::View;

/// What to check, as of when.
#[derive(Debug, Clone)]
pub struct Options {
    /// Which gate to apply. `None` runs every class.
    pub gate: Option<Gate>,
    /// The date expiry is judged against, so the rule is testable without
    /// waiting for a clock.
    pub today: NaiveDate,
    /// Whether to consult git for blame consistency (class `L009`).
    pub blame: bool,
}

impl Options {
    /// Every class, as of `today`, with blame consulted.
    pub fn full(today: NaiveDate) -> Self {
        Self { gate: None, today, blame: true }
    }
}

/// What one run found.
#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Entries that loaded cleanly, for the summary line.
    pub entries: usize,
    /// Decisions the log carries.
    pub decisions: usize,
    /// Allocated, awaiting acceptance — a visible status, not a failure.
    pub awaiting_acceptance: Vec<String>,
    /// Acceptances with no introducing commit yet, so `L009` was skipped.
    pub blame_uncommitted: usize,
    /// Whether git could be consulted at all.
    pub blame_unavailable: bool,
}

impl Report {
    /// Whether the gate passes.
    pub fn is_conformant(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Run the gate over a loaded store.
pub fn verify(store: &Store, opts: &Options) -> Report {
    let view = View::build(store);
    let mut findings = store.schema_findings.clone();
    findings.extend(view.findings.iter().cloned());
    findings.extend(disposition::unallocated(&view));
    findings.extend(disposition::expired(&view, opts.today));
    findings.extend(disposition::stranded(&view, store));
    findings.extend(disposition::model_acceptor(&view));
    findings.extend(disposition::model_judge(&view));
    findings.extend(integrity::hash_mismatch(&view));
    findings.extend(integrity::dangling_acceptance(&view));

    let mut report = Report {
        entries: store.entry_count(),
        decisions: view.latest.len(),
        awaiting_acceptance: awaiting(&view),
        ..Report::default()
    };
    if opts.blame {
        let outcome = integrity::blame_consistency(&view, store);
        findings.extend(outcome.findings);
        report.blame_uncommitted = outcome.uncommitted;
        report.blame_unavailable = outcome.no_repository;
    }
    if let Some(gate) = opts.gate {
        findings.retain(|f| f.class.in_gate(gate));
    }
    findings.sort_by(|a, b| (a.class, &a.subject).cmp(&(b.class, &b.subject)));
    findings.dedup();
    report.findings = findings;
    report
}

/// Decisions that are allocated but carry no live acceptance of their
/// current version. Status, not a finding.
fn awaiting(view: &View) -> Vec<String> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter(|p| p.allocation.is_some())
        .filter(|p| {
            !view
                .acceptances
                .iter()
                .map(|a| a.acceptance)
                .any(|a| !view.is_revoked(a) && view.signs_latest(a) && a.decision == p.decision)
        })
        .map(|p| p.decision.to_string())
        .collect()
}

#[path = "mod_tests.rs"]
#[cfg(test)]
mod tests;
