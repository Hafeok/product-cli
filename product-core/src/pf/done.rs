//! The §7.2 delivery predicates — computed `done`, plus the closed-cut check.
//!
//! `done` is computed, not judged: a deliverable is done when every in-scope
//! What element passes domain conformance, every Decider over an in-scope
//! aggregate is sound + complete (behavioural conformance), and every acceptance
//! criterion is recorded passing. A release is done when all its features are
//! done and its cut is closed — no in-scope element depends on an excluded one.
//! Done is exactly as honest as those verifications are strong.

use std::collections::BTreeSet;

use super::bundle::{covered, dependencies};
use super::decider::Decider;
use super::decider_sim::simulate;
use super::deliverable::Deliverable;
use super::model::DomainGraph;
use super::slice::Slice;
use super::validate::validate_graph;

/// One sub-verification contributing to a deliverable's done verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub kind: String,
    pub subject: String,
    pub passing: bool,
    pub detail: String,
}

/// The computed done verdict for a deliverable.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureDone {
    pub id: String,
    pub done: bool,
    pub checks: Vec<Check>,
}

impl FeatureDone {
    /// Fraction of checks passing (1.0 when there are none to fail).
    pub fn progress(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }
        self.checks.iter().filter(|c| c.passing).count() as f64 / self.checks.len() as f64
    }
}

/// Compute `feature_done` for a deliverable. `conformed` is the set of Decider
/// ids with a recorded passing behavioural-conformance verdict (§6.3) — an
/// in-scope Decider must both simulate sound+complete *and* be conformed.
pub fn feature_done(
    d: &Deliverable,
    slice: &Slice,
    graph: &DomainGraph,
    deciders: &[Decider],
    conformed: &BTreeSet<String>,
) -> FeatureDone {
    let scope = covered(graph, &slice.anchors, slice.depth());
    let mut checks = Vec::new();
    domain_checks(graph, &scope, &mut checks);
    behavioural_checks(deciders, &scope, conformed, &mut checks);
    acceptance_checks(d, &mut checks);
    // A deliverable with no resolved scope (and so no checks) is not done.
    let done = !checks.is_empty() && checks.iter().all(|c| c.passing);
    FeatureDone { id: d.id.clone(), done, checks }
}

/// Domain conformance: each in-scope element has no blocking validation error.
fn domain_checks(graph: &DomainGraph, scope: &BTreeSet<String>, out: &mut Vec<Check>) {
    let violations = validate_graph(graph);
    for id in scope {
        let bad: Vec<&str> = violations
            .iter()
            .filter(|v| &v.focus == id && v.severity == "violation")
            .map(|v| v.path.as_str())
            .collect();
        out.push(Check {
            kind: "domain".to_string(),
            subject: id.clone(),
            passing: bad.is_empty(),
            detail: if bad.is_empty() { "conformant".to_string() } else { format!("violations: {}", bad.join(", ")) },
        });
    }
}

/// Behavioural conformance for each Decider over an in-scope aggregate: it must
/// simulate sound + complete (§3.3, before realisation) *and* have a recorded
/// passing conformance verdict (§6.3, realised code == oracle).
fn behavioural_checks(deciders: &[Decider], scope: &BTreeSet<String>, conformed: &BTreeSet<String>, out: &mut Vec<Check>) {
    for dec in deciders.iter().filter(|d| scope.contains(&d.decides_for)) {
        let findings = simulate(dec);
        out.push(Check {
            kind: "behavioural-sim".to_string(),
            subject: dec.id.clone(),
            passing: findings.is_empty(),
            detail: if findings.is_empty() { "sound + complete".to_string() } else { format!("{} finding(s)", findings.len()) },
        });
        let conformed_ok = conformed.contains(&dec.id);
        out.push(Check {
            kind: "behavioural-conform".to_string(),
            subject: dec.id.clone(),
            passing: conformed_ok,
            detail: if conformed_ok { "realised behaviour matches the oracle".to_string() } else { "pending — run `decider conform`".to_string() },
        });
    }
}

/// Acceptance: every criterion is recorded `passing`.
fn acceptance_checks(d: &Deliverable, out: &mut Vec<Check>) {
    for a in &d.acceptance {
        out.push(Check {
            kind: "acceptance".to_string(),
            subject: a.id.clone(),
            passing: a.status == "passing",
            detail: a.status.clone(),
        });
    }
}

/// The §7.2 "cut is closed" check: every in-scope node's directed dependencies
/// (that exist in the graph) are also in scope. Returns the (node, missing-dep)
/// pairs — empty means the cut is closed.
pub fn cut_closed(graph: &DomainGraph, scope: &BTreeSet<String>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for id in scope {
        for dep in dependencies(graph, id) {
            if graph.kind_of(&dep).is_some() && !scope.contains(&dep) {
                out.push((id.clone(), dep));
            }
        }
    }
    out
}

/// The computed done verdict for a release.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDone {
    pub id: String,
    pub done: bool,
    pub members: Vec<FeatureDone>,
    /// (node, excluded-dependency) pairs; empty means the cut is closed.
    pub open_edges: Vec<(String, String)>,
}

impl ReleaseDone {
    pub fn closed(&self) -> bool {
        self.open_edges.is_empty()
    }
}

/// Compute `release_done`: all member features done AND the cut is closed.
pub fn release_done(id: &str, members: &[(Deliverable, Slice)], graph: &DomainGraph, deciders: &[Decider], conformed: &BTreeSet<String>) -> ReleaseDone {
    let mut union = BTreeSet::new();
    let mut feature_results = Vec::new();
    for (d, s) in members {
        union.extend(covered(graph, &s.anchors, s.depth()));
        feature_results.push(feature_done(d, s, graph, deciders, conformed));
    }
    let open_edges = cut_closed(graph, &union);
    let all_done = !feature_results.is_empty() && feature_results.iter().all(|f| f.done);
    ReleaseDone { id: id.to_string(), done: all_done && open_edges.is_empty(), members: feature_results, open_edges }
}

#[cfg(test)]
#[path = "done_tests.rs"]
mod tests;
