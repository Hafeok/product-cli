//! Pre-flight analysis: domain coverage, cross-cutting checks, dependency availability (ADR-030).

use product_core::domains;
use product_core::error::ProductError;
use product_core::graph::KnowledgeGraph;
use product_core::tc::runner_required;
use product_core::types::DependencyStatus;
use std::process;

use super::{load_graph, BoxResult};

/// FT-058 / E022: refuse preflight when an active feature has any TC
/// missing runner config — fail before the harness invokes the agent.
fn check_runner_required(graph: &KnowledgeGraph, id: &str) -> Result<(), ProductError> {
    let Some(feature) = graph.features.get(id) else {
        return Ok(());
    };
    if !runner_required::status_requires_runner(feature.front.status) {
        return Ok(());
    }
    let offenders = runner_required::find_offenders(graph, id, feature.front.status);
    if offenders.is_empty() {
        return Ok(());
    }
    let tc_paths: Vec<std::path::PathBuf> = offenders
        .iter()
        .filter_map(|tid| graph.tests.get(tid.as_str()).map(|t| t.path.clone()))
        .collect();
    Err(ProductError::TcRunnerMissing {
        feature_id: id.to_string(),
        tc_ids: offenders,
        tc_paths,
    })
}

pub(crate) fn handle_preflight(id: &str, fmt: &str) -> BoxResult {
    let (config, _root, graph) = load_graph()?;
    check_runner_required(&graph, id)?;

    let result = domains::preflight(
        &graph,
        id,
        &config.domains,
        &config.features.default_acknowledged_cross_cutting,
    )?;

    let feature_deps: Vec<_> = graph
        .dependencies
        .values()
        .filter(|d| d.front.features.contains(&id.to_string()))
        .collect();

    let dep_warnings = if fmt == "json" {
        // JSON output mode (FT-104 / TC-174). Dep availability is text-only;
        // a JSON consumer reads `is_clean` instead.
        print_preflight_json(&result, false);
        false
    } else {
        print!("{}", domains::render_preflight(&result));
        render_dep_availability(&feature_deps)
    };

    if !result.is_clean {
        process::exit(1);
    }
    if dep_warnings {
        process::exit(2);
    }
    Ok(())
}

/// FT-104 / TC-174 — JSON preflight output for programmatic consumers.
fn print_preflight_json(result: &domains::PreflightResult, dep_warnings: bool) {
    #[derive(serde::Serialize)]
    struct JsonGap {
        adr_id: String,
        adr_title: String,
        adr_domains: Vec<String>,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    }

    let cross_cutting_gaps: Vec<JsonGap> = result
        .cross_cutting_gaps
        .iter()
        .map(|g| {
            let (status_str, reason) = match &g.status {
                domains::CoverageStatus::Linked => ("linked".to_string(), None),
                domains::CoverageStatus::Acknowledged(r) => {
                    ("acknowledged".to_string(), Some(r.clone()))
                }
                domains::CoverageStatus::DefaultAcknowledged => {
                    ("default-acknowledged".to_string(), None)
                }
                domains::CoverageStatus::Rejected(r) => {
                    ("intentional".to_string(), Some(r.clone()))
                }
                domains::CoverageStatus::Gap => ("gap".to_string(), None),
            };
            JsonGap {
                adr_id: g.adr_id.clone(),
                adr_title: g.adr_title.clone(),
                adr_domains: g.adr_domains.clone(),
                status: status_str,
                reason,
            }
        })
        .collect();

    let output = serde_json::json!({
        "feature_id": result.feature_id,
        "feature_domains": result.feature_domains,
        "cross_cutting_gaps": cross_cutting_gaps,
        "is_clean": result.is_clean && !dep_warnings,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}

/// ADR-030 dependency availability section. Returns true if any dep failed
/// the availability probe or is deprecated/migrating.
fn render_dep_availability(
    feature_deps: &[&product_core::types::Dependency],
) -> bool {
    if feature_deps.is_empty() {
        return false;
    }
    let mut dep_warnings = false;
    println!();
    println!("\u{2501}\u{2501}\u{2501} Dependency Availability \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}");
    println!();
    for dep in feature_deps {
        if !probe_dep(dep) {
            dep_warnings = true;
        }
        if matches!(
            dep.front.status,
            DependencyStatus::Deprecated | DependencyStatus::Migrating
        ) {
            println!(
                "    \u{26A0}  status: {} \u{2014} consider migration",
                dep.front.status
            );
            dep_warnings = true;
        }
    }
    println!();
    dep_warnings
}

/// Run a single dependency's availability probe; print one row.
/// Returns true if the probe passed (or was skipped), false on failure.
fn probe_dep(dep: &product_core::types::Dependency) -> bool {
    match &dep.front.availability_check {
        None => {
            println!(
                "  {}  {:<25} [{} \u{2014} no check]    \u{2713}",
                dep.front.id, dep.front.title, dep.front.dep_type
            );
            true
        }
        Some(check_cmd) => {
            let status = std::process::Command::new("sh")
                .args(["-c", check_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            let ok = matches!(status, Ok(s) if s.success());
            if ok {
                println!(
                    "  {}  {:<25} [{}]         \u{2713}",
                    dep.front.id, dep.front.title, dep.front.dep_type
                );
            } else {
                println!(
                    "  {}  {:<25} [{}]         \u{2717} not running",
                    dep.front.id, dep.front.title, dep.front.dep_type
                );
            }
            ok
        }
    }
}
