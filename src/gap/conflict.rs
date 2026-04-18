//! ADR conflict-check bundle assembly (FT-045, ADR-040).
//!
//! Emits a self-contained markdown document containing:
//!   - Instructions section (conflict codes C001–C004)
//!   - Proposed ADR section (full body)
//!   - Existing ADRs to Check Against — the union of:
//!       - every cross-cutting ADR
//!       - every same-domain ADR
//!       - top-5 ADRs by betweenness centrality
//!
//! Product makes zero LLM calls. The user pipes the output to the LLM of
//! their choice.

use crate::author::prompts as prompt_defs;
use crate::graph::KnowledgeGraph;
use crate::types::{Adr, AdrScope};
use std::collections::HashSet;
use std::path::Path;

fn instructions_section(root: &Path) -> String {
    let content = match prompt_defs::get(root, "conflict-check") {
        Ok(c) if !c.trim().is_empty() => c,
        _ => prompt_defs::default_content("conflict-check"),
    };
    let mut out = String::new();
    out.push_str("## Instructions\n\n");
    out.push_str(content.trim_end());
    out.push_str("\n\n");
    out
}

/// Build a conflict-bundle for a proposed ADR. Returns `None` if the ADR is
/// missing from the graph.
pub fn bundle_for_adr(adr_id: &str, graph: &KnowledgeGraph, root: &Path) -> Option<String> {
    let proposed = graph.adrs.get(adr_id)?;
    let selected_ids = related_adr_ids(adr_id, proposed, graph);

    let mut out = String::new();
    out.push_str(&format!(
        "# Conflict Check Input: {} — {}\n\n",
        proposed.front.id, proposed.front.title
    ));
    out.push_str(&instructions_section(root));

    out.push_str("## Proposed ADR\n\n");
    append_adr_section(&mut out, proposed);

    out.push_str("## Existing ADRs to Check Against\n\n");
    if selected_ids.is_empty() {
        out.push_str("(no related ADRs found)\n\n");
    } else {
        for id in &selected_ids {
            if let Some(adr) = graph.adrs.get(id) {
                append_adr_section(&mut out, adr);
            }
        }
    }

    Some(out)
}

/// Union of cross-cutting + same-domain + top-5 ADRs by betweenness
/// centrality, excluding the proposed ADR itself. Returned sorted by ID.
fn related_adr_ids(
    adr_id: &str,
    proposed: &Adr,
    graph: &KnowledgeGraph,
) -> Vec<String> {
    let mut selected: HashSet<String> = HashSet::new();

    // Cross-cutting
    for adr in graph.adrs.values() {
        if adr.front.id != proposed.front.id && adr.front.scope == AdrScope::CrossCutting {
            selected.insert(adr.front.id.clone());
        }
    }
    // Same-domain
    if !proposed.front.domains.is_empty() {
        for adr in graph.adrs.values() {
            if adr.front.id == proposed.front.id {
                continue;
            }
            if adr.front.domains.iter().any(|d| proposed.front.domains.contains(d)) {
                selected.insert(adr.front.id.clone());
            }
        }
    }
    // Top-5 by centrality
    let centrality = graph.betweenness_centrality();
    let mut ranked: Vec<(String, f64)> = graph
        .adrs
        .keys()
        .filter(|id| id.as_str() != adr_id)
        .map(|id| (id.clone(), centrality.get(id).copied().unwrap_or(0.0)))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (id, _) in ranked.into_iter().take(5) {
        selected.insert(id);
    }

    let mut out: Vec<String> = selected.into_iter().collect();
    out.sort();
    out
}

fn append_adr_section(out: &mut String, adr: &Adr) {
    out.push_str(&format!(
        "### {} — {} [{}]\n\n",
        adr.front.id, adr.front.title, adr.front.scope
    ));
    out.push_str(&format!("**Status:** {:?}\n\n", adr.front.status));
    if !adr.front.domains.is_empty() {
        out.push_str(&format!("**Domains:** {}\n\n", adr.front.domains.join(", ")));
    }
    out.push_str(adr.body.trim_end());
    out.push_str("\n\n---\n\n");
}
