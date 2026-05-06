//! `product_preflight` MCP handler — read-only preflight envelope.

use super::shared::health_error;
use crate::config::ProductConfig;
use crate::domains;
use crate::graph::KnowledgeGraph;
use crate::tc::runner_required;
use crate::types::DependencyStatus;
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn handle_preflight(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            health_error(
                "E022",
                "health-check-id-not-found",
                json!({"message": "id parameter required"}),
            )
        })?;

    let feature = graph.features.get(id).ok_or_else(|| {
        health_error("E022", "health-check-id-not-found", json!({"id": id}))
    })?;

    // Runner-required gate first — short-circuit before domain or dep work.
    if runner_required::status_requires_runner(feature.front.status) {
        let offenders = runner_required::find_offenders(graph, id, feature.front.status);
        if !offenders.is_empty() {
            let tc_paths: Vec<String> = offenders
                .iter()
                .filter_map(|tid| graph.tests.get(tid.as_str()))
                .map(|t| t.path.display().to_string())
                .collect();
            return Err(health_error(
                "E024",
                "health-check-tc-runner-missing",
                json!({
                    "feature_id": id,
                    "tc_ids": offenders,
                    "tc_paths": tc_paths,
                }),
            ));
        }
    }

    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let result = domains::preflight(graph, id, &config.domains).map_err(|e| format!("{}", e))?;

    let cross_cutting = render_cross_cutting(&result);
    let domain_gaps_value = render_domain_gaps(&result);
    let (deps, dep_warnings) = render_deps(graph, id);

    let cross_cutting_gap_count = result
        .cross_cutting_gaps
        .iter()
        .filter(|g| g.status == domains::CoverageStatus::Gap)
        .count();
    let domain_gap_count = result
        .domain_gaps
        .iter()
        .filter(|g| g.status == domains::CoverageStatus::Gap)
        .count();

    let status = if cross_cutting_gap_count > 0 || domain_gap_count > 0 || dep_warnings > 0 {
        "warnings"
    } else {
        "clean"
    };

    Ok(json!({
        "status": status,
        "feature": id,
        "feature_domains": result.feature_domains,
        "cross_cutting_gaps": cross_cutting,
        "domain_gaps": domain_gaps_value,
        "dep_availability": deps,
        "summary": {
            "cross_cutting_gaps": cross_cutting_gap_count,
            "domain_gaps": domain_gap_count,
            "dep_warnings": dep_warnings,
        },
    }))
}

fn render_cross_cutting(result: &domains::PreflightResult) -> Vec<Value> {
    result
        .cross_cutting_gaps
        .iter()
        .map(|gap| {
            let (status_str, reason) = match &gap.status {
                domains::CoverageStatus::Linked => ("linked", None),
                domains::CoverageStatus::Acknowledged(r) => ("acknowledged", Some(r.clone())),
                domains::CoverageStatus::Gap => ("gap", None),
            };
            let mut obj = json!({
                "adr_id": gap.adr_id,
                "adr_title": gap.adr_title,
                "adr_domains": gap.adr_domains,
                "status": status_str,
            });
            if let Some(r) = reason {
                if let Some(map) = obj.as_object_mut() {
                    map.insert("reason".to_string(), Value::String(r));
                }
            }
            obj
        })
        .collect()
}

fn render_domain_gaps(result: &domains::PreflightResult) -> Vec<Value> {
    result
        .domain_gaps
        .iter()
        .map(|gap| {
            let (status_str, reason) = match &gap.status {
                domains::CoverageStatus::Linked => ("linked", None),
                domains::CoverageStatus::Acknowledged(r) => ("acknowledged", Some(r.clone())),
                domains::CoverageStatus::Gap => ("gap", None),
            };
            let top_adrs: Vec<Value> = gap
                .top_adrs
                .iter()
                .map(|(id, title)| json!({"id": id, "title": title}))
                .collect();
            let mut obj = json!({
                "domain": gap.domain,
                "adr_count": gap.adr_count,
                "status": status_str,
                "top_adrs": top_adrs,
            });
            if let Some(r) = reason {
                if let Some(map) = obj.as_object_mut() {
                    map.insert("reason".to_string(), Value::String(r));
                }
            }
            obj
        })
        .collect()
}

fn render_deps(graph: &KnowledgeGraph, feature_id: &str) -> (Vec<Value>, usize) {
    let mut entries = Vec::new();
    let mut warnings = 0usize;
    let feature_deps: Vec<_> = graph
        .dependencies
        .values()
        .filter(|d| d.front.features.contains(&feature_id.to_string()))
        .collect();
    for dep in feature_deps {
        let available = match &dep.front.availability_check {
            None => true,
            Some(cmd) => {
                let res = std::process::Command::new("sh")
                    .args(["-c", cmd])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                matches!(res, Ok(s) if s.success())
            }
        };
        let deprecated = matches!(
            dep.front.status,
            DependencyStatus::Deprecated | DependencyStatus::Migrating
        );
        if !available || deprecated {
            warnings += 1;
        }
        entries.push(json!({
            "id": dep.front.id,
            "title": dep.front.title,
            "type": dep.front.dep_type.to_string(),
            "available": available,
            "deprecated": deprecated,
            "status": dep.front.status.to_string(),
        }));
    }
    (entries, warnings)
}
