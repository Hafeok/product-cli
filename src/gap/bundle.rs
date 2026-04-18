//! Gap analysis bundle assembly — LLM-ready markdown input (FT-045, ADR-040).
//!
//! Emits a self-contained markdown document containing:
//!   - Instructions section listing gap codes G001–G008
//!   - Context Bundle section — depth-2 bundle for the ADR
//!
//! No LLM call is made inside Product. The user pipes the output to their
//! LLM of choice.
//!
//! Supports three scopes:
//!   - `bundle_for_adr(adr_id)` — one ADR
//!   - `bundle_all(graph)` — every ADR
//!   - `bundle_changed(graph, root)` — ADRs changed in the last commit (with
//!     1-hop expansion per ADR-019)

use crate::author::prompts as prompt_defs;
use crate::context;
use crate::graph::KnowledgeGraph;
use std::path::Path;

/// Render the Instructions header for gap analysis bundles.
fn instructions_section(root: &Path) -> String {
    let content = match prompt_defs::get(root, "gap-analysis") {
        Ok(c) if !c.trim().is_empty() => c,
        _ => prompt_defs::default_content("gap-analysis"),
    };
    let mut out = String::new();
    out.push_str("## Instructions\n\n");
    out.push_str(content.trim_end());
    out.push_str("\n\n");
    out
}

/// Build a gap-analysis bundle for a single ADR. Returns `None` if the ADR
/// does not exist in the graph.
pub fn bundle_for_adr(
    adr_id: &str,
    graph: &KnowledgeGraph,
    root: &Path,
) -> Option<String> {
    let adr = graph.adrs.get(adr_id)?;
    let mut out = String::new();

    out.push_str(&format!(
        "# Gap Analysis Input: {} — {}\n\n",
        adr.front.id, adr.front.title
    ));
    out.push_str(&instructions_section(root));

    out.push_str("## Context Bundle\n\n");
    let bundle = context::bundle_adr(graph, adr_id, 2).unwrap_or_default();
    out.push_str(&bundle);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Build one bundle per ADR in the graph, sorted by ADR ID. Each bundle is a
/// self-contained document separated by a horizontal rule + newline.
pub fn bundle_all(graph: &KnowledgeGraph, root: &Path) -> String {
    let mut ids: Vec<&String> = graph.adrs.keys().collect();
    ids.sort();
    let mut out = String::new();
    for id in ids {
        if let Some(b) = bundle_for_adr(id, graph, root) {
            out.push_str(&b);
            out.push_str("\n---\n\n");
        }
    }
    out
}

/// Build one bundle per ADR in the "changed" scope (per ADR-019 `--changed`
/// rules): ADRs modified since the previous commit, expanded by one graph
/// hop so contradictions across shared features are caught.
///
/// Falls back to `bundle_all` when git is unavailable or no prior commit
/// exists — the prompt caller gets a well-formed document in every case.
pub fn bundle_changed(graph: &KnowledgeGraph, root: &Path) -> String {
    let changed = find_changed_adrs(root, graph);
    if changed.is_empty() {
        return bundle_all(graph, root);
    }

    let mut seen: Vec<String> = Vec::new();
    for id in &changed {
        if !seen.contains(id) {
            seen.push(id.clone());
        }
    }

    let mut out = String::new();
    for id in &seen {
        if let Some(b) = bundle_for_adr(id, graph, root) {
            out.push_str(&b);
            out.push_str("\n---\n\n");
        }
    }
    out
}

/// Discover ADRs affected by the last commit. Changed ADR IDs are expanded
/// with the set of ADRs that share any feature with them (1-hop).
fn find_changed_adrs(root: &Path, graph: &KnowledgeGraph) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .current_dir(root)
        .output();
    let changed_files = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    let mut changed_ids: Vec<String> = Vec::new();
    for line in changed_files.lines() {
        if line.contains("adrs/") {
            if let Some(id) = extract_adr_id_from_path(line) {
                if graph.adrs.contains_key(&id) && !changed_ids.contains(&id) {
                    changed_ids.push(id);
                }
            }
        }
    }

    // Expand with 1-hop neighbours through shared features (ADR-019).
    let mut expanded = changed_ids.clone();
    for adr_id in &changed_ids {
        for f in graph.features.values() {
            if f.front.adrs.contains(adr_id) {
                for other_adr in &f.front.adrs {
                    if graph.adrs.contains_key(other_adr)
                        && !expanded.contains(other_adr)
                    {
                        expanded.push(other_adr.clone());
                    }
                }
            }
        }
    }
    expanded.sort();
    expanded
}

fn extract_adr_id_from_path(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let parts: Vec<&str> = filename.splitn(3, '-').collect();
    if parts.len() >= 2 {
        Some(format!("{}-{}", parts[0], parts[1]))
    } else {
        None
    }
}
