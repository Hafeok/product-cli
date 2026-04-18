//! Stage 2 — graph structure (FT-044 step 2, ADR-009).
//!
//! Wraps `product graph check`. Errors: any E-class finding. Warnings:
//! W-class findings.

use super::types::{Finding, StageResult, StageStatus};
use crate::config::ProductConfig;
use crate::domains;
use crate::error::CheckResult;
use crate::graph::KnowledgeGraph;

pub(super) fn run(config: &ProductConfig, graph: &KnowledgeGraph) -> StageResult {
    let mut result: CheckResult = graph.check();
    domains::validate_domains(graph, &config.domains, &mut result.errors, &mut result.warnings);
    crate::graph::responsibility::check_responsibility(graph, config.responsibility(), &mut result);

    let mut findings: Vec<Finding> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for e in &result.errors {
        if seen.insert(e.code.clone()) {
            findings.push(Finding::Code(e.code.clone()));
        }
    }
    for w in &result.warnings {
        if seen.insert(w.code.clone()) {
            findings.push(Finding::Code(w.code.clone()));
        }
    }

    let status = if !result.errors.is_empty() {
        StageStatus::Fail
    } else if !result.warnings.is_empty() {
        StageStatus::Warning
    } else {
        StageStatus::Pass
    };

    let summary = if result.errors.is_empty() && result.warnings.is_empty() {
        "clean".into()
    } else {
        format!(
            "{} error(s), {} warning(s)",
            result.errors.len(),
            result.warnings.len()
        )
    };

    StageResult {
        stage: 2,
        name: "graph-structure",
        status,
        findings,
        summary,
    }
}
