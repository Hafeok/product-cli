//! Pattern section assembly for context bundles (FT-071, ADR-050).
//!
//! Pure helpers: given a knowledge graph and a feature id, compute the set
//! of patterns to include in the bundle in topological order so every
//! prerequisite renders before its dependant.

use crate::graph::pattern_topo::{collect_transitive, topo_sort_patterns, TopoResult};
use crate::graph::KnowledgeGraph;
use crate::types::PatternStatus;

/// Resolve every pattern transitively required by `feature_id` in topological
/// order (prerequisites first). When a cycle is present the resolved set is
/// returned in arbitrary order — the cycle itself surfaces via E031 in
/// `product graph check`.
pub fn collect_patterns_topo(graph: &KnowledgeGraph, feature_id: &str) -> Vec<String> {
    let feature = match graph.features.get(feature_id) {
        Some(f) => f,
        None => return Vec::new(),
    };
    let seeds: Vec<String> = feature
        .front
        .patterns
        .iter()
        .filter(|id| graph.patterns.contains_key(id.as_str()))
        .cloned()
        .collect();
    if seeds.is_empty() {
        return Vec::new();
    }
    let all = collect_transitive(graph, &seeds);
    match topo_sort_patterns(graph, &all) {
        TopoResult::Ordered(o) => o,
        TopoResult::Cycle(_) => all,
    }
}

/// Render the `## Patterns` section. Returns an empty string when no
/// patterns apply to the feature so the caller can skip cleanly.
pub fn render_patterns_section(graph: &KnowledgeGraph, feature_id: &str) -> String {
    let ids = collect_patterns_topo(graph, feature_id);
    if ids.is_empty() {
        return String::new();
    }
    let mut out = String::from("## Patterns\n\n");
    for pat_id in &ids {
        if let Some(pat) = graph.patterns.get(pat_id.as_str()) {
            out.push_str(&format!("### {} — {}\n\n", pat.front.id, pat.front.title));
            if pat.front.status == PatternStatus::Deprecated {
                let by = pat
                    .front
                    .deprecated_by
                    .as_deref()
                    .map(|r| format!(" (replaced by {})", r))
                    .unwrap_or_default();
                out.push_str(&format!("**Status:** Deprecated{}\n\n", by));
            }
            if !pat.body.is_empty() {
                out.push_str(&format!("{}\n\n", pat.body.trim()));
            }
            out.push_str("---\n\n");
        }
    }
    out
}
