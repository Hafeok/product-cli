//! `product_drift_check` MCP handler — read-only structural drift envelope.

use super::shared::{drift_source_settings, health_error, status_for, summarize};
use crate::config::ProductConfig;
use crate::drift;
use crate::graph::KnowledgeGraph;
use crate::tags;
use crate::types::FeatureStatus;
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn handle_drift_check(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let all_complete = args
        .get("all_complete")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let files: Vec<String> = args
        .get("files")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if id.is_some() && all_complete {
        return Err(health_error(
            "E023",
            "health-check-conflicting-args",
            json!({"message": "id and all_complete are mutually exclusive"}),
        ));
    }

    let baseline_path = repo_root.join("drift.json");
    let baseline = drift::DriftBaseline::load(&baseline_path);
    let (source_roots, ignore) = drift_source_settings();

    let config = ProductConfig::load_from_root(repo_root)
        .map_err(|e| format!("{}", e))?;
    let impl_depth = config.tags.implementation_depth;

    if let Some(id_str) = id {
        let is_feature = graph.features.contains_key(id_str);
        let is_adr = graph.adrs.contains_key(id_str);
        if !is_feature && !is_adr {
            return Err(health_error(
                "E022",
                "health-check-id-not-found",
                json!({"id": id_str}),
            ));
        }

        if is_feature {
            return drift_check_feature(
                id_str,
                graph,
                repo_root,
                &baseline,
                &source_roots,
                &ignore,
                impl_depth,
            );
        }

        let findings = drift::check_adr(
            id_str, graph, repo_root, &baseline, &source_roots, &ignore, &files,
        );
        return Ok(json!({
            "status": status_for(&findings),
            "checked": { "scope": id_str },
            "findings": findings,
            "summary": summarize(&findings),
        }));
    }

    if all_complete {
        return drift_check_all_complete(
            graph,
            repo_root,
            &baseline,
            &source_roots,
            &ignore,
            impl_depth,
        );
    }

    drift_check_aggregate(
        graph,
        repo_root,
        &baseline,
        &source_roots,
        &ignore,
        &files,
    )
}

fn drift_check_aggregate(
    graph: &KnowledgeGraph,
    root: &Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    files: &[String],
) -> Result<Value, String> {
    let mut combined: Vec<drift::DriftFinding> = Vec::new();
    let adr_ids: Vec<String> = graph.adrs.keys().cloned().collect();
    for adr_id in &adr_ids {
        combined.extend(drift::check_adr(
            adr_id, graph, root, baseline, source_roots, ignore, files,
        ));
    }
    let is_git = tags::is_git_repo(root);
    let features_with_tags = graph
        .features
        .values()
        .filter(|f| f.front.status == FeatureStatus::Complete)
        .filter(|f| is_git && tags::find_completion_tag(root, &f.front.id).is_some())
        .count();
    Ok(json!({
        "status": status_for(&combined),
        "checked": {
            "scope": "all",
            "adrs": adr_ids.len(),
            "features_with_tags": features_with_tags,
        },
        "findings": combined,
        "summary": summarize(&combined),
    }))
}

fn drift_check_feature(
    feature_id: &str,
    graph: &KnowledgeGraph,
    root: &Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    impl_depth: usize,
) -> Result<Value, String> {
    let report = drift::structural_for_feature(feature_id, graph, root, impl_depth)
        .ok_or_else(|| {
            health_error(
                "E022",
                "health-check-id-not-found",
                json!({"id": feature_id}),
            )
        })?;

    if let Some(tag_name) = report.tag.clone() {
        if report.changed_files.is_empty() {
            return Ok(json!({
                "status": "clean",
                "checked": {
                    "scope": feature_id,
                    "tag": tag_name,
                    "tag_timestamp": report.tag_timestamp,
                },
                "findings": [],
                "summary": { "high": 0, "medium": 0, "low": 0, "suppressed": 0 },
            }));
        }

        let drift_id = format!("DRIFT-{}-TAG-drift", feature_id);
        let suppressed = baseline.is_suppressed(&drift_id);
        let finding = drift::DriftFinding {
            id: drift_id,
            code: "D003".to_string(),
            severity: drift::DriftSeverity::Medium,
            description: format!(
                "Implementation files changed since {} was completed ({})",
                feature_id, tag_name
            ),
            adr_id: feature_id.to_string(),
            source_files: report.changed_files.clone(),
            suggested_action:
                "Review changes to ensure they don't contradict governing ADRs".to_string(),
            suppressed,
        };
        let findings = vec![finding];
        return Ok(json!({
            "status": status_for(&findings),
            "checked": {
                "scope": feature_id,
                "tag": tag_name,
                "tag_timestamp": report.tag_timestamp,
            },
            "findings": findings,
            "summary": summarize(&findings),
        }));
    }

    // No tag — fall back to ADR-level structural drift for linked ADRs.
    let mut combined: Vec<drift::DriftFinding> = Vec::new();
    if let Some(feature) = graph.features.get(feature_id) {
        for adr_id in &feature.front.adrs {
            combined.extend(drift::check_adr(
                adr_id, graph, root, baseline, source_roots, ignore, &[],
            ));
        }
    }
    Ok(json!({
        "status": status_for(&combined),
        "checked": {
            "scope": feature_id,
            "tag": Value::Null,
            "tag_timestamp": Value::Null,
            "warning": "W020",
            "warning_message": format!(
                "no completion tag for {} — structural drift check cannot bound changes",
                feature_id
            ),
        },
        "findings": combined,
        "summary": summarize(&combined),
    }))
}

fn drift_check_all_complete(
    graph: &KnowledgeGraph,
    root: &Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    impl_depth: usize,
) -> Result<Value, String> {
    let is_git = tags::is_git_repo(root);
    let mut all_findings = Vec::new();
    let mut features_checked = 0;

    for feature in graph.features.values() {
        if feature.front.status != FeatureStatus::Complete {
            continue;
        }
        if is_git {
            if let Some(tag_name) = tags::find_completion_tag(root, &feature.front.id) {
                features_checked += 1;
                let (changed_files, _diff) =
                    tags::check_drift_since_tag(root, &tag_name, impl_depth);
                if !changed_files.is_empty() {
                    let id = format!("DRIFT-{}-TAG-drift", feature.front.id);
                    let suppressed = baseline.is_suppressed(&id);
                    all_findings.push(drift::DriftFinding {
                        id,
                        code: "D003".to_string(),
                        severity: drift::DriftSeverity::Medium,
                        description: format!(
                            "Implementation files changed since {} was completed ({})",
                            feature.front.id, tag_name
                        ),
                        adr_id: feature.front.id.clone(),
                        source_files: changed_files,
                        suggested_action:
                            "Review changes to ensure they don't contradict governing ADRs"
                                .to_string(),
                        suppressed,
                    });
                }
                continue;
            }
        }
        for adr_id in &feature.front.adrs {
            all_findings.extend(drift::check_adr(
                adr_id, graph, root, baseline, source_roots, ignore, &[],
            ));
        }
    }
    Ok(json!({
        "status": status_for(&all_findings),
        "checked": {
            "scope": "all-complete",
            "features_with_tags": features_checked,
        },
        "findings": all_findings,
        "summary": summarize(&all_findings),
    }))
}
