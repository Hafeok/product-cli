//! Unit tests for architectural fitness functions (ADR-024)

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
    assert!(output.contains("\u{2191}") || output.contains("\u{2192}"));
}
