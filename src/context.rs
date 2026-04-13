//! Context bundle assembly (ADR-006, ADR-012)

use crate::formal;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashSet;

/// Assemble a context bundle for a feature
pub fn bundle_feature(
    graph: &KnowledgeGraph,
    feature_id: &str,
    depth: usize,
    order_by_centrality: bool,
) -> Option<String> {
    bundle_feature_inner(graph, feature_id, depth, order_by_centrality, false)
}

fn bundle_feature_inner(
    graph: &KnowledgeGraph,
    feature_id: &str,
    depth: usize,
    order_by_centrality: bool,
    adrs_only: bool,
) -> Option<String> {
    let feature = graph.features.get(feature_id)?;
    let reachable = graph.bfs(feature_id, depth);

    // Collect ADRs and tests from reachable set
    let mut adr_ids: Vec<String> = reachable
        .iter()
        .filter(|id| graph.adrs.contains_key(id.as_str()))
        .cloned()
        .collect();

    let mut test_ids: Vec<String> = if adrs_only {
        Vec::new()
    } else {
        reachable
            .iter()
            .filter(|id| graph.tests.contains_key(id.as_str()))
            .cloned()
            .collect()
    };

    // Order ADRs
    if order_by_centrality {
        let centrality = graph.betweenness_centrality();
        adr_ids.sort_by(|a, b| {
            let ca = centrality.get(a).copied().unwrap_or(0.0);
            let cb = centrality.get(b).copied().unwrap_or(0.0);
            cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        adr_ids.sort();
    }

    // Order tests by phase then type (exit-criteria first, then scenario, invariant, chaos)
    test_ids.sort_by(|a, b| {
        let ta = graph.tests.get(a.as_str());
        let tb = graph.tests.get(b.as_str());
        let phase_a = ta.map(|t| t.front.phase).unwrap_or(0);
        let phase_b = tb.map(|t| t.front.phase).unwrap_or(0);
        let type_order = |tt: TestType| -> u8 {
            match tt {
                TestType::ExitCriteria => 0,
                TestType::Scenario => 1,
                TestType::Invariant => 2,
                TestType::Chaos => 3,
            }
        };
        let type_a = ta.map(|t| type_order(t.front.test_type)).unwrap_or(9);
        let type_b = tb.map(|t| type_order(t.front.test_type)).unwrap_or(9);
        phase_a.cmp(&phase_b).then(type_a.cmp(&type_b))
    });

    // Handle ADR supersession: include superseded ADRs with annotation (TC-019)
    let mut final_adr_ids = Vec::new();
    let mut seen = HashSet::new();
    for id in &adr_ids {
        if !seen.contains(id) {
            seen.insert(id.clone());
            final_adr_ids.push(id.clone());
        }
    }

    // Compute aggregate evidence from all test criteria
    let all_evidence: Vec<&formal::EvidenceBlock> = test_ids
        .iter()
        .filter_map(|id| graph.tests.get(id.as_str()))
        .flat_map(|t| t.formal_blocks.iter())
        .filter_map(|b| match b {
            formal::FormalBlock::Evidence(e) => Some(e),
            _ => None,
        })
        .collect();

    let avg_delta = if all_evidence.is_empty() {
        0.0
    } else {
        all_evidence.iter().map(|e| e.delta).sum::<f64>() / all_evidence.len() as f64
    };
    let formal_count = test_ids
        .iter()
        .filter_map(|id| graph.tests.get(id.as_str()))
        .filter(|t| !t.formal_blocks.is_empty())
        .count();
    let phi = if test_ids.is_empty() {
        0
    } else {
        (formal_count * 100) / test_ids.len()
    };

    // Build the bundle
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "# Context Bundle: {} — {}\n\n",
        feature.front.id, feature.front.title
    ));

    // AISP header block
    out.push_str(&format!(
        "⟦Ω:Bundle⟧{{\n  feature≜{}:Feature\n  phase≜{}:Phase\n  status≜{:?}:FeatureStatus\n  generated≜{}\n  implementedBy≜⟨{}⟩:Decision+\n  validatedBy≜⟨{}⟩:TestCriterion+\n}}\n",
        feature.front.id,
        feature.front.phase,
        feature.front.status,
        chrono::Utc::now().to_rfc3339(),
        final_adr_ids.join(","),
        test_ids.join(","),
    ));
    if !all_evidence.is_empty() {
        out.push_str(&format!("⟦Ε⟧⟨δ≜{:.2};φ≜{};τ≜◊⁺⟩\n", avg_delta, phi));
    }
    out.push_str("\n---\n\n");

    // Feature content
    out.push_str(&format!(
        "## Feature: {} — {}\n\n{}\n\n---\n\n",
        feature.front.id, feature.front.title, feature.body
    ));

    // ADR content
    for adr_id in &final_adr_ids {
        if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
            if adr.front.status == AdrStatus::Superseded {
                let by_label = if let Some(by) = adr.front.superseded_by.first() {
                    format!(" by {}", by)
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "## {} — {} [SUPERSEDED{}]\n\n**Status:** Superseded{}\n\n{}\n\n---\n\n",
                    adr.front.id, adr.front.title, by_label, by_label, adr.body
                ));
            } else {
                out.push_str(&format!(
                    "## {} — {}\n\n**Status:** {:?}\n\n{}\n\n---\n\n",
                    adr.front.id, adr.front.title, adr.front.status, adr.body
                ));
            }
        }
    }

    // Test criteria
    if !test_ids.is_empty() {
        out.push_str("## Test Criteria\n\n");
        for test_id in &test_ids {
            if let Some(tc) = graph.tests.get(test_id.as_str()) {
                out.push_str(&format!(
                    "### {} — {} ({})\n\n{}\n\n",
                    tc.front.id, tc.front.title, tc.front.test_type, tc.body
                ));
            }
        }
    }

    // Depth warning
    let total = 1 + final_adr_ids.len() + test_ids.len();
    if depth >= 3 && total > 50 {
        eprintln!(
            "warning: bundle contains {} artifacts at depth {}. Consider narrowing scope.",
            total, depth
        );
    }

    Some(out)
}

/// Assemble context for an ADR (all linked features + all linked tests)
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

/// Bundle all features in a phase
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
        if let Some(bundle) = bundle_feature_inner(graph, fid, depth, order_by_centrality, adrs_only) {
            out.push_str(&bundle);
        }
    }

    out
}
