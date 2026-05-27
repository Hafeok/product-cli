//! Collect linked artifacts into a `Collected` view for the renderer.
//!
//! Pure walk over the assembled `KnowledgeGraph` — gathers feature, ADRs, TCs,
//! dependencies as a single struct.

use super::loader::Template;
use crate::graph::KnowledgeGraph;
use crate::types::*;

pub struct Collected<'a> {
    pub feature: &'a Feature,
    pub adrs: Vec<&'a Adr>,
    pub tests: Vec<&'a TestCriterion>,
    pub deps: Vec<&'a Dependency>,
    /// Patterns linked to the feature in topological order over `requires:`
    /// (FT-071, ADR-050). Prerequisite patterns precede their dependants.
    pub patterns: Vec<&'a Pattern>,
}

pub fn collect<'a>(
    graph: &'a KnowledgeGraph,
    feature: &'a Feature,
    depth: usize,
    tpl: &Template,
) -> Collected<'a> {
    let reachable = graph.bfs(&feature.front.id, depth);

    let mut adr_ids: Vec<String> = reachable
        .iter()
        .filter(|id| graph.adrs.contains_key(id.as_str()))
        .cloned()
        .collect();
    let centrality = graph.betweenness_centrality();
    if tpl.ordering.adrs_ordered_by == "centrality" {
        adr_ids.sort_by(|a, b| {
            let ca = centrality.get(a).copied().unwrap_or(0.0);
            let cb = centrality.get(b).copied().unwrap_or(0.0);
            cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        adr_ids.sort();
    }
    let adrs: Vec<&Adr> = adr_ids
        .iter()
        .filter_map(|id| graph.adrs.get(id.as_str()))
        .collect();

    let mut tests: Vec<&TestCriterion> = reachable
        .iter()
        .filter_map(|id| graph.tests.get(id.as_str()))
        .collect();
    if tpl.ordering.tcs_ordered_by == "type" {
        tests.sort_by(|a, b| {
            a.front
                .test_type
                .bundle_sort_key()
                .cmp(&b.front.test_type.bundle_sort_key())
        });
    } else {
        tests.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    }

    let deps: Vec<&Dependency> = reachable
        .iter()
        .filter_map(|id| graph.dependencies.get(id.as_str()))
        .collect();

    // FT-071: patterns in topological order over `requires:`.
    let pattern_ids = crate::context::collect_patterns_topo(graph, &feature.front.id);
    let patterns: Vec<&Pattern> = pattern_ids
        .iter()
        .filter_map(|id| graph.patterns.get(id.as_str()))
        .collect();

    Collected {
        feature,
        adrs,
        tests,
        deps,
        patterns,
    }
}
