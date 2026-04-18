//! Stage 5 — feature TCs (FT-044 step 5, ADR-021).
//!
//! For each feature reachable from the current phase gate: skip if
//! `status: planned`, otherwise run its configured TCs. Features in locked
//! phases (per ADR-040 phase-gate) are skipped with a named reason.

use super::types::{Finding, PipelineScope, StageResult, StageStatus};
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::implement::verify as verify_impl;
use crate::types::{FeatureStatus, TestStatus};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn run(
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
    scope: &PipelineScope,
) -> StageResult {
    let locked = locked_phases(graph);
    let mut findings: Vec<Finding> = Vec::new();
    let mut stage_status = StageStatus::Pass;

    let mut feature_ids: Vec<&String> = graph.features.keys().collect();
    feature_ids.sort();

    for fid in feature_ids {
        let feature = match graph.features.get(fid) {
            Some(f) => f,
            None => continue,
        };
        if let Some(p) = scope.phase {
            if feature.front.phase != p {
                continue;
            }
        }
        if scope.phase.is_none() && locked.contains(&feature.front.phase) {
            for tc_id in &feature.front.tests {
                findings.push(Finding::Tc {
                    tc: tc_id.clone(),
                    feature: Some(fid.clone()),
                    status: "skipped".into(),
                    reason: Some(format!("phase-{}-locked", feature.front.phase)),
                });
            }
            continue;
        }
        if feature.front.status == FeatureStatus::Planned
            || feature.front.status == FeatureStatus::Abandoned
        {
            continue;
        }

        let feature_status = run_feature_tcs(fid, graph, config, root, &mut findings);
        stage_status = stage_status.merge(feature_status);
    }

    let summary = match stage_status {
        StageStatus::Pass => "all passing".into(),
        StageStatus::Warning => "warnings present".into(),
        StageStatus::Fail => {
            let n_fail = findings
                .iter()
                .filter(|f| matches!(f, Finding::Tc { status, .. } if status == "failing"))
                .count();
            format!("{} failing", n_fail)
        }
    };

    StageResult {
        stage: 5,
        name: "feature-tcs",
        status: stage_status,
        findings,
        summary,
    }
}

/// Phase-lock detection per ADR-040: phase N is locked if phase N+1 contains
/// at least one feature with status `complete`.
fn locked_phases(graph: &KnowledgeGraph) -> std::collections::HashSet<u32> {
    let mut by_phase: BTreeMap<u32, Vec<&crate::types::Feature>> = BTreeMap::new();
    for f in graph.features.values() {
        by_phase.entry(f.front.phase).or_default().push(f);
    }
    let mut locked = std::collections::HashSet::new();
    let phases: Vec<u32> = by_phase.keys().copied().collect();
    for &p in &phases {
        if let Some(next_features) = by_phase.get(&(p + 1)) {
            if next_features
                .iter()
                .any(|f| f.front.status == FeatureStatus::Complete)
            {
                locked.insert(p);
            }
        }
    }
    locked
}

/// Run the TCs linked to `feature_id` without mutating feature status or
/// regenerating the checklist.
fn run_feature_tcs(
    feature_id: &str,
    graph: &KnowledgeGraph,
    config: &ProductConfig,
    root: &Path,
    findings: &mut Vec<Finding>,
) -> StageStatus {
    let feature = match graph.features.get(feature_id) {
        Some(f) => f,
        None => return StageStatus::Pass,
    };

    let now = chrono::Utc::now().to_rfc3339();
    let tc_ids = feature.front.tests.clone();
    let mut status = StageStatus::Pass;

    for tc_id in &tc_ids {
        let tc = match graph.tests.get(tc_id.as_str()) {
            Some(t) => t,
            None => continue,
        };
        let content = std::fs::read_to_string(&tc.path).unwrap_or_default();
        let runner = verify_impl::extract_yaml_field_public(&content, "runner");
        let runner_args = verify_impl::extract_yaml_field_public(&content, "runner-args");
        let requires = verify_impl::extract_yaml_list_public(&content, "requires");

        if tc.front.status == TestStatus::Unrunnable {
            findings.push(Finding::Tc {
                tc: tc.front.id.clone(),
                feature: Some(feature_id.to_string()),
                status: "unrunnable".into(),
                reason: Some("acknowledged".into()),
            });
            status = status.merge(StageStatus::Warning);
            continue;
        }
        if runner.is_empty() {
            findings.push(Finding::Tc {
                tc: tc.front.id.clone(),
                feature: Some(feature_id.to_string()),
                status: "unimplemented".into(),
                reason: None,
            });
            status = status.merge(StageStatus::Warning);
            continue;
        }

        if !requires.is_empty() {
            match check_prereqs(&requires, config, root) {
                PrereqCheck::AllOk => {}
                PrereqCheck::Missing(name) => {
                    findings.push(Finding::Tc {
                        tc: tc.front.id.clone(),
                        feature: Some(feature_id.to_string()),
                        status: "unrunnable".into(),
                        reason: Some(format!("prereq '{}' not satisfied", name)),
                    });
                    let _ = verify_impl::update_tc_status_public(
                        &tc.path,
                        "unrunnable",
                        &now,
                        Some(&format!("prerequisite '{}' not satisfied", name)),
                        None,
                    );
                    status = status.merge(StageStatus::Warning);
                    continue;
                }
                PrereqCheck::Undefined(name) => {
                    findings.push(Finding::Tc {
                        tc: tc.front.id.clone(),
                        feature: Some(feature_id.to_string()),
                        status: "unrunnable".into(),
                        reason: Some(format!("prereq '{}' not defined", name)),
                    });
                    status = status.merge(StageStatus::Warning);
                    continue;
                }
            }
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
                    feature: Some(feature_id.to_string()),
                    status: "failing".into(),
                    reason: None,
                });
                status = status.merge(StageStatus::Fail);
            }
        }
    }
    status
}

enum PrereqCheck {
    AllOk,
    Missing(String),
    Undefined(String),
}

fn check_prereqs(requires: &[String], config: &ProductConfig, root: &Path) -> PrereqCheck {
    use std::process::Command;
    for name in requires {
        match config.verify.prerequisites.get(name.as_str()) {
            None => return PrereqCheck::Undefined(name.clone()),
            Some(cmd) => {
                let ok = Command::new("bash")
                    .args(["-c", cmd])
                    .current_dir(root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !ok {
                    return PrereqCheck::Missing(name.clone());
                }
            }
        }
    }
    PrereqCheck::AllOk
}
