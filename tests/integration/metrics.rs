//! Integration tests — metrics.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_126_metrics_record_appends() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest.\n",
    );

    // Record twice
    let out1 = h.run(&["metrics", "record"]);
    out1.assert_exit(0);
    let out2 = h.run(&["metrics", "record"]);
    out2.assert_exit(0);

    // Check metrics.jsonl has two lines
    let content = h.read("metrics.jsonl");
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "metrics.jsonl should have 2 lines, got: {}", content);

    // Both lines should be valid JSON with required fields
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line)
            .expect("each line should be valid JSON");
        assert!(v.get("date").is_some(), "missing date field");
        assert!(v.get("spec_coverage").is_some(), "missing spec_coverage");
        assert!(v.get("test_coverage").is_some(), "missing test_coverage");
        assert!(v.get("phi").is_some(), "missing phi");
    }
}

#[test]
fn tc_127_metrics_threshold_error_exits_1() {
    let h = Harness::new();
    // Override product.toml with threshold config
    h.write(
        "product.toml",
        r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
[metrics.thresholds.spec_coverage]
min = 0.99
severity = "error"
"#,
    );
    // Create a feature without ADR links → spec_coverage = 0
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    let out = h.run(&["metrics", "threshold"]);
    out.assert_exit(1);
}

#[test]
fn tc_128_metrics_threshold_warning_exits_2() {
    let h = Harness::new();
    h.write(
        "product.toml",
        r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
[metrics.thresholds.spec_coverage]
min = 0.99
severity = "warning"
"#,
    );
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    let out = h.run(&["metrics", "threshold"]);
    out.assert_exit(2);
}

#[test]
fn tc_129_metrics_threshold_clean_exits_0() {
    let h = Harness::new();
    h.write(
        "product.toml",
        r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
[metrics.thresholds.spec_coverage]
min = 0.50
severity = "error"
"#,
    );
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );

    let out = h.run(&["metrics", "threshold"]);
    out.assert_exit(0);
}

#[test]
fn tc_130_metrics_trend_renders() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );

    // Write 10 metrics records directly to metrics.jsonl
    let mut records = String::new();
    for i in 0..10 {
        let cov = 0.5 + (i as f64) * 0.05;
        records.push_str(&format!(
            r#"{{"date":"2026-04-{:02}","commit":"abc{}","spec_coverage":{},"test_coverage":0.8,"exit_criteria_coverage":0.6,"phi":0.7,"gap_density":0.1,"gap_resolution_rate":0.5,"drift_density":0.0,"centrality_stability":0.0,"implementation_velocity":1}}"#,
            i + 1, i, cov
        ));
        records.push('\n');
    }
    h.write("metrics.jsonl", &records);

    let out = h.run(&["metrics", "trend"]);
    out.assert_exit(0);
    // Should contain sparkline output
    assert!(
        !out.stdout.is_empty(),
        "metrics trend should produce output"
    );
    assert!(
        out.stdout.contains("spec_coverage") || out.stdout.contains("phi"),
        "Should contain metric names in trend output, got: {}",
        out.stdout
    );
}

#[test]
fn tc_131_metrics_jsonl_merge_conflict_safe() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );

    // Two records on the same line (simulating a bad merge)
    let bad_line = r#"{"date":"2026-04-01","commit":"aaa","spec_coverage":0.8,"test_coverage":0.7,"exit_criteria_coverage":0.6,"phi":0.7,"gap_density":0.1,"gap_resolution_rate":0.5,"drift_density":0.0,"centrality_stability":0.0,"implementation_velocity":1}{"date":"2026-04-02","commit":"bbb","spec_coverage":0.9,"test_coverage":0.8,"exit_criteria_coverage":0.7,"phi":0.8,"gap_density":0.05,"gap_resolution_rate":0.6,"drift_density":0.0,"centrality_stability":0.0,"implementation_velocity":2}"#;
    let content = format!("{}\n", bad_line);
    h.write("metrics.jsonl", &content);

    let out = h.run(&["metrics", "trend"]);
    out.assert_exit(0);
    // Should emit a W-class warning about the malformed line
    assert!(
        out.stderr.contains("warning") || out.stderr.contains("W009"),
        "Should emit warning about merge conflict, got stderr: {}",
        out.stderr
    );
    // Should still produce output (recovered records)
    assert!(
        !out.stdout.is_empty(),
        "Should still render trend output despite malformed line"
    );
}

