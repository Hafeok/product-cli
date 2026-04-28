//! Integration tests — tc.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_027_exit_code_clean() {
    let h = fixture_minimal();
    h.run(&["graph", "check"]).assert_exit(0);
}

#[test]
fn tc_028_exit_code_broken_link() {
    let h = fixture_broken_link();
    h.run(&["graph", "check"]).assert_exit(1);
}

#[test]
fn tc_029_exit_code_warnings_only() {
    let h = fixture_orphaned_adr();
    h.run(&["graph", "check"]).assert_exit(2);
}

#[test]
fn tc_030_exit_code_ci_pipeline() {
    // Clean graph → exit 0
    let h = fixture_minimal();
    h.run(&["graph", "check"]).assert_exit(0);

    // Broken link → exit 1 (error)
    let h2 = fixture_broken_link();
    h2.run(&["graph", "check"]).assert_exit(1);

    // Warning-only (orphaned ADR) → exit 2
    let h3 = fixture_orphaned_adr();
    h3.run(&["graph", "check"]).assert_exit(2);
}

#[test]
fn tc_080_exit_criteria() {
    let h = Harness::new();
    let adr_source = r#"# ADRs

## ADR-001: Test ADR

**Status:** Accepted

Some context.

### Exit criteria

- `exit_binary_compiles` — binary compiles successfully
- `exit_all_tests_pass` — all tests pass
"#;
    h.write("source-adrs.md", adr_source);
    let out = h.run(&["migrate", "from-adrs", "source-adrs.md", "--execute"]);
    out.assert_exit(0);

    // Check that test criteria files were created
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/tests"))
        .expect("readdir")
        .flatten()
        .collect();
    assert!(
        !entries.is_empty(),
        "should have created test criteria files"
    );

    // Verify at least one test file has type: exit-criteria
    let mut found_exit_criteria = false;
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("type: exit-criteria") {
            found_exit_criteria = true;
            break;
        }
    }
    assert!(
        found_exit_criteria,
        "should have extracted at least one exit-criteria test from ### Exit criteria heading"
    );
}

#[test]
fn tc_084_validates_adrs() {
    let h = Harness::new();
    let adr_source = r#"# ADRs

## ADR-005: Storage Engine

**Status:** Accepted

Context.

### Test coverage

- `storage_init` — initializes storage
- `storage_read` — reads from storage
"#;
    h.write("source-adrs.md", adr_source);
    let out = h.run(&["migrate", "from-adrs", "source-adrs.md", "--execute"]);
    out.assert_exit(0);

    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/tests"))
        .expect("readdir")
        .flatten()
        .collect();
    assert!(entries.len() >= 2, "should create at least 2 test criteria");

    // Every test extracted from ADR-005 must validate ADR-005
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        assert!(
            content.contains("ADR-005"),
            "test file {} should have validates.adrs containing ADR-005, got:\n{}",
            entry.file_name().to_string_lossy(),
            content
        );
    }
}

#[test]
fn tc_085_validates_features() {
    let h = Harness::new();
    let prd_source = "# PRD\n\n## Feature Alpha\n\nAlpha content.\n\n## Feature Beta\n\nBeta content.\n";
    h.write("source-prd.md", prd_source);
    let out = h.run(&["migrate", "from-prd", "source-prd.md", "--execute"]);
    out.assert_exit(0);

    // Features extracted from PRD should have empty adrs and tests lists (not inferred)
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .flatten()
        .collect();
    assert_eq!(entries.len(), 2, "should create 2 features");

    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        // adrs and tests should be empty arrays
        assert!(
            content.contains("adrs: []"),
            "feature {} should have empty adrs (not inferred), got:\n{}",
            entry.file_name().to_string_lossy(),
            content
        );
        assert!(
            content.contains("tests: []"),
            "feature {} should have empty tests (not inferred), got:\n{}",
            entry.file_name().to_string_lossy(),
            content
        );
    }
}

#[test]
fn tc_275_exit_criteria_heading_context() {
    let h = Harness::new();

    // ADR with a ### Exit criteria section whose bullets do NOT contain "exit"
    // in their titles — the heading context should set type: exit-criteria.
    let adr_source = r#"# ADRs

## ADR-010: Deployment Pipeline

**Status:** Accepted

Pipeline deploys the system.

### Exit criteria

- `binary_compiles_arm64` — ARM64 binary compiles successfully
- `all_tests_pass` — full test suite passes
- `cluster_healthy` — cluster reports healthy after deploy
"#;
    h.write("source-adrs.md", adr_source);
    let out = h.run(&["migrate", "from-adrs", "source-adrs.md", "--execute"]);
    out.assert_exit(0);

    // All three bullets should produce type: exit-criteria files
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/tests"))
        .expect("readdir")
        .flatten()
        .collect();
    assert_eq!(entries.len(), 3, "should create 3 test criteria files");

    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        assert!(
            content.contains("type: exit-criteria"),
            "all bullets under ### Exit criteria should have type: exit-criteria, \
             but {} has:\n{}",
            entry.file_name().to_string_lossy(),
            content
        );
    }

    // Validate mode also shows exit-criteria type in plan output
    // (re-create harness to avoid conflicts from existing files)
    let h2 = Harness::new();
    h2.write("source-adrs.md", adr_source);
    let out = h2.run(&["migrate", "from-adrs", "source-adrs.md", "--validate"]);
    out.assert_exit(0)
        .assert_stdout_contains("exit-criteria");
}

#[test]
fn tc_601_tc_type_exit_criteria_drives_phase_gate() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001.md",
        "---\nid: FT-001\ntitle: Feature FT-001\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001, TC-002]\n---\n\nBody\n",
    );
    ft048_write_feature(&h, "FT-002", 2, &[]);
    ft048_write_tc(&h, "TC-001", "Phase1 Exit", "exit-criteria", "failing", "FT-001", 1);
    ft048_write_tc(&h, "TC-002", "Scenario", "scenario", "failing", "FT-001", 1);
    let out = h.run(&["feature", "next"]);
    assert!(
        out.stdout.contains("locked") || out.stdout.contains("TC-001") || out.stderr.contains("TC-001"),
        "expected gate-locked report. stdout: {} stderr: {}",
        out.stdout, out.stderr
    );
    ft048_write_tc(&h, "TC-001", "Phase1 Exit", "exit-criteria", "passing", "FT-001", 1);
    let out = h.run(&["feature", "next"]);
    out.assert_stdout_contains("FT-002");
}

#[test]
fn tc_602_tc_type_invariant_requires_formal_block() {
    let h = Harness::new();
    ft048_write_feature(&h, "FT-001", 1, &["TC-001"]);
    ft048_write_tc(&h, "TC-001", "Inv", "invariant", "unimplemented", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(out.stderr.contains("W004"), "expected W004. stderr: {}", out.stderr);
    ft048_write_tc(&h, "TC-001", "Inv", "scenario", "unimplemented", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(!out.stderr.contains("W004"), "no W004 for scenario. stderr: {}", out.stderr);
}

#[test]
fn tc_603_tc_type_chaos_requires_formal_block() {
    let h = Harness::new();
    ft048_write_feature(&h, "FT-001", 1, &["TC-001"]);
    ft048_write_tc(&h, "TC-001", "Chaos", "chaos", "unimplemented", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(out.stderr.contains("W004"), "expected W004. stderr: {}", out.stderr);
}

#[test]
fn tc_604_tc_type_absence_drives_g009() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001.md",
        "---\nid: FT-001\ntitle: F\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-removes.md",
        "---\nid: ADR-001\ntitle: Remove Foo\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\nremoves:\n  - foo-library\n---\n\n**Rejected alternatives:**\n- none\n",
    );
    h.write(
        "docs/tests/TC-001.md",
        "---\nid: TC-001\ntitle: Scenario Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nBody\n",
    );
    let out = h.run(&["graph", "check"]);
    assert!(out.stderr.contains("W022"), "expected W022. stderr: {}", out.stderr);
    h.write(
        "docs/tests/TC-002.md",
        "---\nid: TC-002\ntitle: Abs\ntype: absence\nstatus: passing\nvalidates:\n  features: []\n  adrs: [ADR-001]\nphase: 1\n---\n\nBody\n",
    );
    let out = h.run(&["graph", "check"]);
    assert!(!out.stderr.contains("W022"), "W022 should clear. stderr: {}", out.stderr);
}

#[test]
fn tc_616_tc_types_system_exit() {
    let h = ft048_tc_types(&["contract"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001", "TC-002", "TC-003", "TC-004", "TC-005"]);
    ft048_write_tc(&h, "TC-001", "X", "exit-criteria", "passing", "FT-001", 1);
    h.write(
        "docs/tests/TC-002.md",
        "---\nid: TC-002\ntitle: I\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ x }\n",
    );
    h.write(
        "docs/tests/TC-003.md",
        "---\nid: TC-003\ntitle: C\ntype: chaos\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ y }\n",
    );
    ft048_write_tc(&h, "TC-004", "Sc", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-005", "Ct", "contract", "passing", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "exit 0 or 2; got {}; stderr: {}",
        out.exit_code, out.stderr
    );
    assert!(!out.stderr.contains("E006"), "no E006 expected");
    assert!(!out.stderr.contains("E017"), "no E017 expected");
}

