//! Architectural fitness functions — continuous metric tracking (ADR-024)
//!
//! `product metrics record` — append snapshot to metrics.jsonl
//! `product metrics threshold` — CI gate on configured thresholds
//! `product metrics trend` — ASCII sparkline

use crate::error::{ProductError, Result};
use crate::gap;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Metric snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub date: String,
    #[serde(default)]
    pub commit: String,
    pub spec_coverage: f64,
    pub test_coverage: f64,
    pub exit_criteria_coverage: f64,
    pub phi: f64,
    pub gap_density: f64,
    pub gap_resolution_rate: f64,
    pub drift_density: f64,
    pub centrality_stability: f64,
    pub implementation_velocity: usize,
}

/// Compute a metric snapshot from the current graph state
pub fn record(graph: &KnowledgeGraph, root: &Path) -> MetricSnapshot {
    let total_features = graph.features.len().max(1) as f64;
    let total_adrs = graph.adrs.len().max(1) as f64;

    let (gap_density, gap_resolution_rate) = compute_gap_metrics(graph, root, total_adrs);
    let drift_density = compute_drift_density(root, total_adrs);
    let commit = current_git_commit();

    MetricSnapshot {
        date: chrono::Utc::now().to_rfc3339(),
        commit,
        spec_coverage: compute_spec_coverage(graph, total_features),
        test_coverage: compute_test_coverage(graph, total_features),
        exit_criteria_coverage: compute_exit_criteria_coverage(graph, total_features),
        phi: compute_phi(graph),
        gap_density,
        gap_resolution_rate,
        drift_density,
        centrality_stability: 0.0,
        implementation_velocity: graph.features.values()
            .filter(|f| f.front.status == FeatureStatus::Complete)
            .count(),
    }
}

/// Features with at least one linked ADR / total features.
fn compute_spec_coverage(graph: &KnowledgeGraph, total: f64) -> f64 {
    let count = graph.features.values().filter(|f| !f.front.adrs.is_empty()).count() as f64;
    count / total
}

/// Features with at least one linked TC / total features.
fn compute_test_coverage(graph: &KnowledgeGraph, total: f64) -> f64 {
    let count = graph.features.values().filter(|f| !f.front.tests.is_empty()).count() as f64;
    count / total
}

/// Features with an exit-criteria TC / total features.
fn compute_exit_criteria_coverage(graph: &KnowledgeGraph, total: f64) -> f64 {
    let count = graph.features.values().filter(|f| {
        f.front.tests.iter().any(|tid| {
            graph.tests.get(tid.as_str())
                .map(|t| t.front.test_type == TestType::ExitCriteria)
                .unwrap_or(false)
        })
    }).count() as f64;
    count / total
}

/// Formal block coverage across invariant+chaos TCs.
fn compute_phi(graph: &KnowledgeGraph) -> f64 {
    let invariant_chaos: Vec<&TestCriterion> = graph.tests.values()
        .filter(|t| t.front.test_type == TestType::Invariant || t.front.test_type == TestType::Chaos)
        .collect();
    if invariant_chaos.is_empty() {
        return 1.0;
    }
    let with_formal = invariant_chaos.iter().filter(|t| !t.formal_blocks.is_empty()).count();
    with_formal as f64 / invariant_chaos.len() as f64
}

/// Gap density and gap resolution rate.
fn compute_gap_metrics(graph: &KnowledgeGraph, root: &Path, total_adrs: f64) -> (f64, f64) {
    let baseline = gap::GapBaseline::load(&root.join("gaps.json"));
    let reports = gap::check_all(graph, &baseline);
    let total_unsuppressed: usize = reports.iter()
        .flat_map(|r| &r.findings)
        .filter(|f| !f.suppressed)
        .count();
    let gap_density = total_unsuppressed as f64 / total_adrs;

    let resolved = baseline.resolved.len();
    let suppressed = baseline.suppressions.len();
    let rate = if resolved + suppressed == 0 { 1.0 } else { resolved as f64 / (resolved + suppressed) as f64 };
    (gap_density, rate)
}

/// Drift density from the drift baseline.
fn compute_drift_density(root: &Path, total_adrs: f64) -> f64 {
    let drift_baseline = crate::drift::DriftBaseline::load(&root.join("drift.json"));
    drift_baseline.suppressions.len() as f64 / total_adrs
}

/// Get the short git commit hash, or empty string if not available.
fn current_git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or_default()
}

/// Append a snapshot to metrics.jsonl
pub fn append_snapshot(snapshot: &MetricSnapshot, path: &Path) -> Result<()> {
    let line = serde_json::to_string(snapshot).map_err(|e| {
        ProductError::IoError(format!("failed to serialize metric: {}", e))
    })?;

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ProductError::IoError(format!("failed to open metrics.jsonl: {}", e)))?;
    writeln!(file, "{}", line).map_err(|e| ProductError::IoError(e.to_string()))?;
    Ok(())
}

/// Load all snapshots from metrics.jsonl, returning snapshots and any warnings
pub fn load_snapshots_with_warnings(path: &Path) -> (Vec<MetricSnapshot>, Vec<String>) {
    if !path.exists() {
        return (Vec::new(), Vec::new());
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut snapshots = Vec::new();
    let mut warnings = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip git merge conflict markers
        if trimmed.starts_with('<') || trimmed.starts_with('>') || trimmed.starts_with('=') {
            warnings.push(format!(
                "warning[W009]: metrics.jsonl line {}: merge conflict marker detected, skipping",
                line_num + 1
            ));
            continue;
        }
        match serde_json::from_str::<MetricSnapshot>(trimmed) {
            Ok(s) => snapshots.push(s),
            Err(_) => {
                // Try to recover multiple JSON objects on one line (bad merge)
                let mut recovered = try_split_json_objects(trimmed);
                if recovered.is_empty() {
                    warnings.push(format!(
                        "warning[W009]: metrics.jsonl line {}: malformed record, skipping",
                        line_num + 1
                    ));
                } else {
                    warnings.push(format!(
                        "warning[W009]: metrics.jsonl line {}: recovered {} records from malformed line (possible merge conflict)",
                        line_num + 1,
                        recovered.len()
                    ));
                    snapshots.append(&mut recovered);
                }
            }
        }
    }

    (snapshots, warnings)
}

/// Load all snapshots from metrics.jsonl (legacy interface, no warnings)
pub fn load_snapshots(path: &Path) -> Vec<MetricSnapshot> {
    load_snapshots_with_warnings(path).0
}

/// Try to split a line containing multiple concatenated JSON objects
fn try_split_json_objects(line: &str) -> Vec<MetricSnapshot> {
    let mut results = Vec::new();
    let mut depth = 0i32;
    let mut start = None;

    for (i, ch) in line.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let fragment = &line[s..=i];
                        if let Ok(snapshot) = serde_json::from_str::<MetricSnapshot>(fragment) {
                            results.push(snapshot);
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Threshold checking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default = "default_severity")]
    pub severity: String,
}

fn default_severity() -> String {
    "warning".to_string()
}

/// Check a snapshot against thresholds, returns (errors, warnings)
pub fn check_thresholds(
    snapshot: &MetricSnapshot,
    thresholds: &std::collections::HashMap<String, ThresholdConfig>,
) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let metrics = [
        ("spec_coverage", snapshot.spec_coverage),
        ("test_coverage", snapshot.test_coverage),
        ("exit_criteria_coverage", snapshot.exit_criteria_coverage),
        ("phi", snapshot.phi),
        ("gap_density", snapshot.gap_density),
        ("gap_resolution_rate", snapshot.gap_resolution_rate),
        ("drift_density", snapshot.drift_density),
    ];

    for (name, value) in &metrics {
        if let Some(threshold) = thresholds.get(*name) {
            let breached = if let Some(min) = threshold.min {
                *value < min
            } else if let Some(max) = threshold.max {
                *value > max
            } else {
                false
            };

            if breached {
                let msg = format!(
                    "{}: {:.3} (threshold: {})",
                    name, value,
                    if let Some(min) = threshold.min { format!("min {:.3}", min) }
                    else if let Some(max) = threshold.max { format!("max {:.3}", max) }
                    else { "none".to_string() }
                );
                if threshold.severity == "error" {
                    errors.push(msg);
                } else {
                    warnings.push(msg);
                }
            }
        }
    }

    (errors, warnings)
}

// ---------------------------------------------------------------------------
// Trend display
// ---------------------------------------------------------------------------

/// Extract metric value from a snapshot by name.
fn extract_metric_value(s: &MetricSnapshot, metric_name: &str) -> f64 {
    match metric_name {
        "spec_coverage" => s.spec_coverage,
        "test_coverage" => s.test_coverage,
        "exit_criteria_coverage" => s.exit_criteria_coverage,
        "phi" => s.phi,
        "gap_density" => s.gap_density,
        "gap_resolution_rate" => s.gap_resolution_rate,
        "drift_density" => s.drift_density,
        "centrality_stability" => s.centrality_stability,
        "implementation_velocity" => s.implementation_velocity as f64,
        _ => 0.0,
    }
}

/// Compute the 7-day (or oldest available) delta.
fn compute_delta(values: &[f64]) -> f64 {
    let current = values.last().copied().unwrap_or(0.0);
    if values.len() >= 7 {
        current - values[values.len() - 7]
    } else if values.len() >= 2 {
        current - values[0]
    } else {
        0.0
    }
}

/// Build an ASCII sparkline string from a value series.
fn build_sparkline(values: &[f64]) -> String {
    let min = values.iter().cloned().fold(f64::MAX, f64::min);
    let max = values.iter().cloned().fold(f64::MIN, f64::max);
    let range = (max - min).max(0.01);
    let block_chars: Vec<char> = "\u{2581}\u{2582}\u{2583}\u{2584}\u{2585}\u{2586}\u{2587}\u{2588}".chars().collect();

    values.iter().map(|v| {
        let idx = (((v - min) / range) * 7.0) as usize;
        block_chars[idx.min(7)]
    }).collect()
}

/// Render ASCII sparkline for a metric over time
pub fn render_trend(snapshots: &[MetricSnapshot], metric_name: &str) -> String {
    let values: Vec<f64> = snapshots.iter().map(|s| extract_metric_value(s, metric_name)).collect();

    if values.is_empty() {
        return format!("{}: no data\n", metric_name);
    }

    let current = values.last().copied().unwrap_or(0.0);
    let delta_7d = compute_delta(&values);
    let trend_arrow = if delta_7d > 0.01 { "\u{2191}" }
        else if delta_7d < -0.01 { "\u{2193}" }
        else { "\u{2192}" };
    let sparkline = build_sparkline(&values);

    format!(
        "{}: {} current={:.3} \u{0394}={:+.3} trend={}\n",
        metric_name, sparkline, current, delta_7d, trend_arrow
    )
}

/// Render a summary table of all metrics
pub fn render_summary(snapshot: &MetricSnapshot) -> String {
    let mut out = String::new();
    out.push_str("Metric                      Value\n");
    out.push_str("───────────────────────── ─────────\n");
    out.push_str(&format!("spec_coverage              {:.3}\n", snapshot.spec_coverage));
    out.push_str(&format!("test_coverage              {:.3}\n", snapshot.test_coverage));
    out.push_str(&format!("exit_criteria_coverage     {:.3}\n", snapshot.exit_criteria_coverage));
    out.push_str(&format!("phi                        {:.3}\n", snapshot.phi));
    out.push_str(&format!("gap_density                {:.3}\n", snapshot.gap_density));
    out.push_str(&format!("gap_resolution_rate        {:.3}\n", snapshot.gap_resolution_rate));
    out.push_str(&format!("drift_density              {:.3}\n", snapshot.drift_density));
    out.push_str(&format!("centrality_stability       {:.3}\n", snapshot.centrality_stability));
    out.push_str(&format!("implementation_velocity    {}\n", snapshot.implementation_velocity));
    out
}


#[cfg(test)]
mod tests;
