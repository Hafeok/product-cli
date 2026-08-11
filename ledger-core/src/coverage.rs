//! The coverage query — the primitive git cannot express (PRD §8).
//!
//! A claim about absence relative to a declared scope: per set and per
//! namespace, how many decisions sit in each of the seven disposition
//! states, with supersession chains walkable to their tips. Derived from
//! the same view the gate reads, so coverage, status and verify can never
//! disagree. Every report carries the honest limit: coverage is measured
//! against the *enumerated* set, and nothing verifies the set itself.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;

use crate::store::Store;
use crate::verify::state::{self, DispositionState};
use crate::verify::view::View;

/// The honest limit, stated verbatim in every rendering (PRD §8).
pub const HONEST_LIMIT: &str = "coverage is measured against the enumerated set; \
nothing verifies the set itself — enumeration completeness has no mechanical check";

/// State counts for one grouping key (a set or a namespace).
pub type StateCounts = BTreeMap<&'static str, usize>;

/// The coverage reading of a whole store.
#[derive(Debug, Serialize)]
pub struct CoverageReport {
    pub decisions: usize,
    pub by_set: BTreeMap<String, StateCounts>,
    pub by_namespace: BTreeMap<String, StateCounts>,
    /// Every decision's state, for machines.
    pub states: BTreeMap<String, DispositionState>,
    /// Supersession chains, root first, walked to the tip.
    pub chains: Vec<Vec<String>>,
    /// The §8 disclosure; serialized so machines carry it too.
    pub honest_limit: &'static str,
}

/// Compute coverage as of `today`, optionally scoped to one set.
pub fn coverage(store: &Store, today: NaiveDate, set: Option<&str>) -> CoverageReport {
    let view = View::build(store);
    let rows: Vec<_> = state::states(&view, today)
        .into_values()
        .filter(|r| set.is_none_or(|s| r.set == s))
        .collect();
    let mut by_set: BTreeMap<String, StateCounts> = BTreeMap::new();
    let mut by_namespace: BTreeMap<String, StateCounts> = BTreeMap::new();
    let mut states = BTreeMap::new();
    for row in &rows {
        *by_set.entry(row.set.clone()).or_default().entry(row.state.as_str()).or_default() += 1;
        *by_namespace
            .entry(row.namespace.clone())
            .or_default()
            .entry(row.state.as_str())
            .or_default() += 1;
        states.insert(row.decision.clone(), row.state);
    }
    CoverageReport {
        decisions: rows.len(),
        by_set,
        by_namespace,
        states,
        chains: chains(&view, &rows),
        honest_limit: HONEST_LIMIT,
    }
}

/// Chains rooted at decisions nobody supersedes, restricted to the scope.
fn chains(view: &View, rows: &[state::StateRow]) -> Vec<Vec<String>> {
    let superseders = state::supersession(view);
    let in_scope: std::collections::BTreeSet<&str> =
        rows.iter().map(|r| r.decision.as_str()).collect();
    let tips: std::collections::BTreeSet<&str> =
        superseders.values().map(String::as_str).collect();
    superseders
        .keys()
        .filter(|old| !tips.contains(old.as_str()))
        .filter(|old| in_scope.contains(old.as_str()))
        .map(|root| state::chain_from(root, &superseders))
        .collect()
}

/// The terminal rendering.
pub fn render(report: &CoverageReport) -> String {
    let mut lines = vec![format!("coverage — {} decision(s)", report.decisions)];
    lines.push("by set:".to_string());
    lines.extend(group_lines(&report.by_set));
    lines.push("by namespace:".to_string());
    lines.extend(group_lines(&report.by_namespace));
    if !report.chains.is_empty() {
        lines.push("supersession chains:".to_string());
        for chain in &report.chains {
            lines.push(format!("  {}", chain.join(" -> ")));
        }
    }
    lines.push(format!("the honest limit: {HONEST_LIMIT} (PRD §8)"));
    lines.join("\n")
}

fn group_lines(groups: &BTreeMap<String, StateCounts>) -> Vec<String> {
    if groups.is_empty() {
        return vec!["  (nothing enumerated)".to_string()];
    }
    groups
        .iter()
        .map(|(key, counts)| {
            let parts: Vec<String> = DispositionState::ALL
                .iter()
                .filter_map(|s| counts.get(s.as_str()).map(|n| format!("{} {n}", s.as_str())))
                .collect();
            format!("  {key}: {}", parts.join(" · "))
        })
        .collect()
}
