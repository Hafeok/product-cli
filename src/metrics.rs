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

    // spec_coverage: features with ≥1 linked ADR / total
    let with_adr = graph.features.values().filter(|f| !f.front.adrs.is_empty()).count() as f64;
    let spec_coverage = with_adr / total_features;

    // test_coverage: features with ≥1 linked TC / total
    let with_test = graph.features.values().filter(|f| !f.front.tests.is_empty()).count() as f64;
    let test_coverage = with_test / total_features;

    // exit_criteria_coverage: features with exit-criteria TC / total
    let with_exit = graph.features.values().filter(|f| {
        f.front.tests.iter().any(|tid| {
            graph.tests.get(tid.as_str())
                .map(|t| t.front.test_type == TestType::ExitCriteria)
                .unwrap_or(false)
        })
    }).count() as f64;
    let exit_criteria_coverage = with_exit / total_features;

    // phi: mean formal block coverage across invariant+chaos TCs
    let invariant_chaos: Vec<&TestCriterion> = graph.tests.values()
        .filter(|t| t.front.test_type == TestType::Invariant || t.front.test_type == TestType::Chaos)
        .collect();
    let with_formal = invariant_chaos.iter().filter(|t| !t.formal_blocks.is_empty()).count();
    let phi = if invariant_chaos.is_empty() { 1.0 } else { with_formal as f64 / invariant_chaos.len() as f64 };

    // gap_density: count unsuppressed gaps / total ADRs
    let baseline = gap::GapBaseline::load(&root.join("gaps.json"));
    let reports = gap::check_all(graph, &baseline);
    let total_unsuppressed: usize = reports.iter()
        .flat_map(|r| &r.findings)
        .filter(|f| !f.suppressed)
        .count();
    let total_adrs = graph.adrs.len().max(1) as f64;
    let gap_density = total_unsuppressed as f64 / total_adrs;

    // gap_resolution_rate: resolved / (resolved + suppressed)
    let resolved = baseline.resolved.len();
    let suppressed = baseline.suppressions.len();
    let gap_resolution_rate = if resolved + suppressed == 0 { 1.0 } else { resolved as f64 / (resolved + suppressed) as f64 };

    // drift_density: placeholder
    let drift_baseline = crate::drift::DriftBaseline::load(&root.join("drift.json"));
    let drift_density = drift_baseline.suppressions.len() as f64 / total_adrs;

    // centrality_stability: placeholder
    let centrality_stability = 0.0;

    // implementation_velocity: features marked complete
    let velocity = graph.features.values()
        .filter(|f| f.front.status == FeatureStatus::Complete)
        .count();

    // Current git commit
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or_default();

    MetricSnapshot {
        date: chrono::Utc::now().to_rfc3339(),
        commit,
        spec_coverage,
        test_coverage,
        exit_criteria_coverage,
        phi,
        gap_density,
        gap_resolution_rate,
        drift_density,
        centrality_stability,
        implementation_velocity: velocity,
    }
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

/// Render ASCII sparkline for a metric over time
pub fn render_trend(snapshots: &[MetricSnapshot], metric_name: &str) -> String {
    let values: Vec<f64> = snapshots.iter().map(|s| {
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
    }).collect();

    if values.is_empty() {
        return format!("{}: no data\n", metric_name);
    }

    let current = values.last().copied().unwrap_or(0.0);
    let delta_7d = if values.len() >= 7 {
        current - values[values.len() - 7]
    } else if values.len() >= 2 {
        current - values[0]
    } else {
        0.0
    };

    let trend_arrow = if delta_7d > 0.01 { "↑" }
        else if delta_7d < -0.01 { "↓" }
        else { "→" };

    // Simple sparkline using block chars
    let min = values.iter().cloned().fold(f64::MAX, f64::min);
    let max = values.iter().cloned().fold(f64::MIN, f64::max);
    let range = (max - min).max(0.01);
    let blocks = "▁▂▃▄▅▆▇█";
    let block_chars: Vec<char> = blocks.chars().collect();

    let sparkline: String = values.iter().map(|v| {
        let idx = (((v - min) / range) * 7.0) as usize;
        block_chars[idx.min(7)]
    }).collect();

    format!(
        "{}: {} current={:.3} Δ={:+.3} trend={}\n",
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
mod tests {
    use super::*;

    #[test]
    fn snapshot_serializes() {
        let s = MetricSnapshot {
            date: "2026-04-12".to_string(),
            commit: "abc123".to_string(),
            spec_coverage: 0.87,
            test_coverage: 0.72,
            exit_criteria_coverage: 0.61,
            phi: 0.68,
            gap_density: 0.03,
            gap_resolution_rate: 0.75,
            drift_density: 0.1,
            centrality_stability: 0.02,
            implementation_velocity: 2,
        };
        let json = serde_json::to_string(&s).expect("serialize");
        assert!(json.contains("spec_coverage"));
    }

    #[test]
    fn append_and_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("metrics.jsonl");
        let s = MetricSnapshot {
            date: "2026-04-12".to_string(),
            commit: "test".to_string(),
            spec_coverage: 0.9,
            test_coverage: 0.8,
            exit_criteria_coverage: 0.7,
            phi: 0.6,
            gap_density: 0.1,
            gap_resolution_rate: 0.5,
            drift_density: 0.0,
            centrality_stability: 0.0,
            implementation_velocity: 1,
        };
        append_snapshot(&s, &path).expect("append");
        append_snapshot(&s, &path).expect("append again");
        let loaded = load_snapshots(&path);
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn threshold_breach_detected() {
        let s = MetricSnapshot {
            date: "test".to_string(),
            commit: String::new(),
            spec_coverage: 0.5,
            test_coverage: 0.3,
            exit_criteria_coverage: 0.0,
            phi: 0.4,
            gap_density: 0.8,
            gap_resolution_rate: 0.1,
            drift_density: 0.5,
            centrality_stability: 0.0,
            implementation_velocity: 0,
        };
        let mut thresholds = std::collections::HashMap::new();
        thresholds.insert("spec_coverage".to_string(), ThresholdConfig { min: Some(0.9), max: None, severity: "error".to_string() });
        thresholds.insert("gap_density".to_string(), ThresholdConfig { min: None, max: Some(0.2), severity: "warning".to_string() });

        let (errors, warnings) = check_thresholds(&s, &thresholds);
        assert!(!errors.is_empty(), "spec_coverage below threshold should be error");
        assert!(!warnings.is_empty(), "gap_density above threshold should be warning");
    }

    #[test]
    fn trend_renders() {
        let snapshots: Vec<MetricSnapshot> = (0..5).map(|i| MetricSnapshot {
            date: format!("2026-04-{:02}", i + 1),
            commit: String::new(),
            spec_coverage: 0.5 + i as f64 * 0.1,
            test_coverage: 0.5,
            exit_criteria_coverage: 0.5,
            phi: 0.5,
            gap_density: 0.1,
            gap_resolution_rate: 0.5,
            drift_density: 0.0,
            centrality_stability: 0.0,
            implementation_velocity: 0,
        }).collect();
        let output = render_trend(&snapshots, "spec_coverage");
        assert!(output.contains("spec_coverage"));
        assert!(output.contains("↑") || output.contains("→"));
    }
}
