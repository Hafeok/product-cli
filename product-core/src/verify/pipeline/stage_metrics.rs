//! Stage 4 — metrics thresholds (FT-044 step 4, ADR-024).
//!
//! Wraps `product metrics threshold`. Exit status by severity=error vs
//! severity=warning thresholds in `[metrics.thresholds]`.

use super::types::{Finding, StageResult, StageStatus};
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::metrics;
use std::path::Path;

pub(super) fn run(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    root: &Path,
) -> StageResult {
    let snapshot = metrics::record(graph, root);
    let thresholds = config
        .metrics
        .as_ref()
        .map(|m| &m.thresholds)
        .cloned()
        .unwrap_or_default();
    let (errors, warnings) = metrics::check_thresholds(&snapshot, &thresholds);

    let mut findings: Vec<Finding> = Vec::new();
    for e in &errors {
        if let Some(name) = e.split(':').next() {
            findings.push(Finding::Code(name.trim().to_string()));
        }
    }
    for w in &warnings {
        if let Some(name) = w.split(':').next() {
            findings.push(Finding::Code(name.trim().to_string()));
        }
    }

    let status = if !errors.is_empty() {
        StageStatus::Fail
    } else if !warnings.is_empty() {
        StageStatus::Warning
    } else {
        StageStatus::Pass
    };

    let summary = if errors.is_empty() && warnings.is_empty() {
        "clean".into()
    } else {
        format!("{} error(s), {} warning(s)", errors.len(), warnings.len())
    };

    StageResult {
        stage: 4,
        name: "metrics",
        status,
        findings,
        summary,
    }
}
