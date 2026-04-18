//! Stage 6 — platform TCs (FT-044 step 6, ADR-025).
//!
//! Wraps `product verify --platform` — TCs linked to cross-cutting ADRs,
//! regardless of feature association.

use super::types::{Finding, StageResult, StageStatus};
use crate::config::ProductConfig;
use crate::fileops;
use crate::graph::KnowledgeGraph;
use crate::implement::verify as verify_impl;
use crate::parser;
use crate::types::{AdrScope, TestStatus};
use std::path::Path;

pub(super) fn run(
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
) -> StageResult {
    let mut platform_tc_ids: Vec<String> = collect_platform_tcs(graph);
    platform_tc_ids.sort();

    let mut findings: Vec<Finding> = Vec::new();
    let mut status = StageStatus::Pass;
    let now = chrono::Utc::now().to_rfc3339();

    for tc_id in &platform_tc_ids {
        let tc = match graph.tests.get(tc_id.as_str()) {
            Some(t) => t,
            None => continue,
        };
        let content = std::fs::read_to_string(&tc.path).unwrap_or_default();
        let runner = verify_impl::extract_yaml_field_public(&content, "runner");
        let runner_args = verify_impl::extract_yaml_field_public(&content, "runner-args");
        if tc.front.status == TestStatus::Unrunnable {
            findings.push(Finding::Tc {
                tc: tc.front.id.clone(),
                feature: None,
                status: "unrunnable".into(),
                reason: Some("acknowledged".into()),
            });
            status = status.merge(StageStatus::Warning);
            continue;
        }
        if runner.is_empty() {
            findings.push(Finding::Tc {
                tc: tc.front.id.clone(),
                feature: None,
                status: "unimplemented".into(),
                reason: None,
            });
            status = status.merge(StageStatus::Warning);
            continue;
        }
        match verify_impl::run_tc_public(&runner, &runner_args, root) {
            (true, dur, _) => {
                let _ = verify_impl::update_tc_status_public(
                    &tc.path, "passing", &now, None, Some(dur),
                );
            }
            (false, dur, msg) => {
                let _ = verify_impl::update_tc_status_public(
                    &tc.path, "failing", &now, Some(&msg), Some(dur),
                );
                findings.push(Finding::Tc {
                    tc: tc.front.id.clone(),
                    feature: None,
                    status: "failing".into(),
                    reason: None,
                });
                status = status.merge(StageStatus::Fail);
            }
        }
    }

    let summary = if platform_tc_ids.is_empty() {
        "no platform TCs".into()
    } else {
        render_summary(status, &platform_tc_ids, &findings)
    };

    regen_checklist(config, root);

    StageResult {
        stage: 6,
        name: "platform-tcs",
        status,
        findings,
        summary,
    }
}

fn collect_platform_tcs(graph: &KnowledgeGraph) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for adr in graph.adrs.values() {
        if adr.front.scope == AdrScope::CrossCutting {
            for tc in graph.tests.values() {
                if tc.front.validates.adrs.contains(&adr.front.id)
                    && !out.contains(&tc.front.id)
                {
                    out.push(tc.front.id.clone());
                }
            }
        }
    }
    out
}

fn render_summary(
    status: StageStatus,
    platform_tc_ids: &[String],
    findings: &[Finding],
) -> String {
    match status {
        StageStatus::Pass => format!("{}/{} passing", platform_tc_ids.len(), platform_tc_ids.len()),
        StageStatus::Warning => "warnings present".into(),
        StageStatus::Fail => {
            let n_fail = findings
                .iter()
                .filter(|f| matches!(f, Finding::Tc { status, .. } if status == "failing"))
                .count();
            format!("{} failing", n_fail)
        }
    }
}

fn regen_checklist(config: &ProductConfig, root: &Path) {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    if let Ok(loaded) = parser::load_all(&features_dir, &adrs_dir, &tests_dir) {
        let new_graph = KnowledgeGraph::build(loaded.features, loaded.adrs, loaded.tests);
        let checklist_path = config.resolve_path(root, &config.paths.checklist);
        if let Some(parent) = checklist_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = fileops::write_file_atomic(
            &checklist_path,
            &crate::checklist::generate(&new_graph),
        );
    }
}
