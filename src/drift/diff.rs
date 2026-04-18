//! Drift diff bundle assembly — LLM-ready markdown input (FT-045, ADR-040).
//!
//! Emits a self-contained markdown document containing:
//!   - Instructions section listing drift codes D001–D004
//!   - Implementation Anchor — completion tag + timestamp
//!   - Changes Since Completion — git diff bounded to implementation files
//!   - Governing ADRs — depth-2 bundle for the ADRs linked to the feature
//!
//! Product makes zero LLM calls. The user pipes the output to the LLM of
//! their choice.

use crate::author::prompts as prompt_defs;
use crate::context;
use crate::graph::KnowledgeGraph;
use crate::tags;
use std::path::Path;

fn instructions_section(root: &Path) -> String {
    let content = match prompt_defs::get(root, "drift-analysis") {
        Ok(c) if !c.trim().is_empty() => c,
        _ => prompt_defs::default_content("drift-analysis"),
    };
    let mut out = String::new();
    out.push_str("## Instructions\n\n");
    out.push_str(content.trim_end());
    out.push_str("\n\n");
    out
}

/// Result of assembling a drift diff for a feature. `warn_w020` is `true`
/// when no completion tag exists — the caller is expected to emit a W020
/// warning on stderr.
pub struct DriftDiff {
    pub markdown: String,
    pub warn_w020: bool,
}

/// Build a drift-diff bundle for a feature. Returns `None` if the feature
/// does not exist in the graph.
pub fn diff_for_feature(
    feature_id: &str,
    graph: &KnowledgeGraph,
    root: &Path,
    implementation_depth: usize,
) -> Option<DriftDiff> {
    let feature = graph.features.get(feature_id)?;

    let mut out = String::new();
    out.push_str(&format!(
        "# Drift Analysis Input: {} — {}\n\n",
        feature.front.id, feature.front.title
    ));
    out.push_str(&instructions_section(root));

    // Implementation Anchor
    out.push_str("## Implementation Anchor\n\n");
    out.push_str(&format!("Feature: {}\n", feature.front.id));

    let is_git = tags::is_git_repo(root);
    let tag_opt = if is_git {
        tags::find_completion_tag(root, feature_id)
    } else {
        None
    };

    let warn_w020 = tag_opt.is_none();
    let (changed_files, diff_text) = match &tag_opt {
        Some(tag_name) => {
            let ts = tags::tag_timestamp(root, tag_name).unwrap_or_else(|| "(unknown)".into());
            out.push_str(&format!("Completion tag: {} ({})\n", tag_name, ts));
            let (files, diff) = tags::check_drift_since_tag(root, tag_name, implementation_depth);
            let files_summary = summarise_impl_files(root, tag_name, implementation_depth);
            out.push_str(&format!("Implementation files: {}\n\n", files_summary));
            (files, diff)
        }
        None => {
            out.push_str("Completion tag: (none)\n");
            out.push_str("Implementation files: (unknown — no completion tag)\n\n");
            (Vec::new(), String::new())
        }
    };

    // Changes Since Completion
    out.push_str("## Changes Since Completion\n\n");
    if tag_opt.is_none() {
        out.push_str("(no completion tag — no changes recorded)\n\n");
    } else if changed_files.is_empty() {
        out.push_str("(no changes since completion)\n\n");
    } else {
        out.push_str("Changed files:\n");
        for f in &changed_files {
            out.push_str(&format!("- {}\n", f));
        }
        out.push('\n');
        if !diff_text.is_empty() {
            out.push_str("```diff\n");
            out.push_str(diff_text.trim_end());
            out.push_str("\n```\n\n");
        }
    }

    // Governing ADRs
    out.push_str("## Governing ADRs\n\n");
    if feature.front.adrs.is_empty() {
        out.push_str("(no governing ADRs linked)\n\n");
    } else {
        for adr_id in &feature.front.adrs {
            if let Some(bundle) = context::bundle_adr(graph, adr_id, 2) {
                out.push_str(&bundle);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }

    Some(DriftDiff { markdown: out, warn_w020 })
}

fn summarise_impl_files(root: &Path, tag_name: &str, depth: usize) -> String {
    let files = tags::implementation_files(root, tag_name, depth);
    if files.is_empty() {
        return "0 files".to_string();
    }
    let mut roots: Vec<String> = files
        .iter()
        .filter_map(|p| p.iter().next())
        .map(|s| s.to_string_lossy().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    roots.sort();
    let preview: Vec<String> = roots.into_iter().take(3).map(|s| format!("{}/", s)).collect();
    format!(
        "{} file{} across {}",
        files.len(),
        if files.len() == 1 { "" } else { "s" },
        if preview.is_empty() {
            "(unknown)".to_string()
        } else {
            preview.join(", ")
        }
    )
}

/// Render a short structural report listing changed files since the
/// completion tag. Used by `product drift check FT-XXX` (structural, no LLM).
pub struct StructuralReport {
    pub tag: Option<String>,
    pub tag_timestamp: Option<String>,
    pub changed_files: Vec<String>,
    pub is_git: bool,
}

pub fn structural_for_feature(
    feature_id: &str,
    graph: &KnowledgeGraph,
    root: &Path,
    implementation_depth: usize,
) -> Option<StructuralReport> {
    if !graph.features.contains_key(feature_id) {
        return None;
    }
    let is_git = tags::is_git_repo(root);
    if !is_git {
        return Some(StructuralReport {
            tag: None,
            tag_timestamp: None,
            changed_files: Vec::new(),
            is_git: false,
        });
    }
    match tags::find_completion_tag(root, feature_id) {
        Some(tag_name) => {
            let ts = tags::tag_timestamp(root, &tag_name);
            let (files, _) = tags::check_drift_since_tag(root, &tag_name, implementation_depth);
            Some(StructuralReport {
                tag: Some(tag_name),
                tag_timestamp: ts,
                changed_files: files,
                is_git: true,
            })
        }
        None => Some(StructuralReport {
            tag: None,
            tag_timestamp: None,
            changed_files: Vec::new(),
            is_git: true,
        }),
    }
}
