//! Context bundle assembly for ADR seeds (ADR-006).

use super::bundle_feature_inner;
use crate::graph::KnowledgeGraph;

/// Assemble context for an ADR (all linked features + all linked tests).
pub fn bundle_adr(
    graph: &KnowledgeGraph,
    adr_id: &str,
    depth: usize,
) -> Option<String> {
    let adr = graph.adrs.get(adr_id)?;
    let reachable = graph.bfs(adr_id, depth);

    let feature_ids: Vec<String> = reachable
        .iter()
        .filter(|id| graph.features.contains_key(id.as_str()))
        .cloned()
        .collect();

    let test_ids: Vec<String> = reachable
        .iter()
        .filter(|id| graph.tests.contains_key(id.as_str()))
        .cloned()
        .collect();

    let mut out = String::new();
    out.push_str(&format!(
        "# Context Bundle: {} — {}\n\n---\n\n",
        adr.front.id, adr.front.title
    ));
    out.push_str(&format!(
        "## {} — {}\n\n{}\n\n---\n\n",
        adr.front.id, adr.front.title, adr.body
    ));

    for fid in &feature_ids {
        if let Some(f) = graph.features.get(fid.as_str()) {
            out.push_str(&format!(
                "## Feature: {} — {}\n\n{}\n\n---\n\n",
                f.front.id, f.front.title, f.body
            ));
        }
    }

    if !test_ids.is_empty() {
        out.push_str("## Test Criteria\n\n");
        for tid in &test_ids {
            if let Some(tc) = graph.tests.get(tid.as_str()) {
                out.push_str(&format!(
                    "### {} — {} ({})\n\n{}\n\n",
                    tc.front.id, tc.front.title, tc.front.test_type, tc.body
                ));
            }
        }
    }

    Some(out)
}

/// Bundle every feature in a given phase by concatenating per-feature bundles.
pub fn bundle_phase(
    graph: &KnowledgeGraph,
    phase: u32,
    depth: usize,
    adrs_only: bool,
    order_by_centrality: bool,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Context Bundle: Phase {}\n\n---\n\n", phase));

    let mut feature_ids: Vec<&String> = graph
        .features
        .values()
        .filter(|f| f.front.phase == phase)
        .map(|f| &f.front.id)
        .collect();
    feature_ids.sort();

    for fid in &feature_ids {
        if let Some(bundle) = bundle_feature_inner(graph, fid, depth, order_by_centrality, adrs_only, None) {
            out.push_str(&bundle);
        }
    }

    out
}
