//! Integration tests — schema.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn it_012_schema_forward_error() {
    let h = Harness::new();
    // Overwrite product.toml with future schema
    h.write("product.toml", "name = \"test\"\nschema-version = \"99\"\n");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(1)
        .assert_stderr_contains("E008");
}

#[test]
fn it_013_schema_backward_warning() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0)
        .assert_stderr_contains("W007");
}

#[test]
fn it_014_migrate_schema_dry_run() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\n");
    let before = h.read("docs/features/FT-001-test.md");
    h.run(&["migrate", "schema", "--dry-run"]).assert_exit(0);
    let after = h.read("docs/features/FT-001-test.md");
    assert_eq!(before, after, "dry-run should not modify files");
}

#[test]
fn it_016_migrate_prd_validate() {
    let h = Harness::new();
    h.write("source.md", "# PRD\n\n## Feature One\n\nContent.\n\n## Feature Two\n\nMore.\n");
    let out = h.run(&["migrate", "from-prd", "source.md", "--validate"]);
    out.assert_exit(0)
        .assert_stdout_contains("Migration plan");
    // No feature files should be created
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .collect();
    assert_eq!(entries.len(), 0, "validate should not create files");
}

#[test]
fn it_018_migrate_source_unchanged() {
    let h = Harness::new();
    let source_content = "# PRD\n\n## Feature One\n\nContent.\n";
    h.write("source.md", source_content);
    h.run(&["migrate", "from-prd", "source.md", "--execute"]);
    let after = h.read("source.md");
    assert_eq!(source_content, after, "source must be unchanged");
}

#[test]
fn tc_060_schema_version_forward_error() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"99\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(1)
        .assert_stderr_contains("E008");
}

#[test]
fn tc_061_schema_version_backward_warning() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n");
    let out = h.run(&["graph", "check"]);
    // Should complete (exit 0 or 2 for warnings) and show W007
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "backward compat should not hard-error, got exit code {}: stderr={}",
        out.exit_code, out.stderr
    );
    out.assert_stderr_contains("W007");
}

#[test]
fn tc_062_schema_migrate_dry_run() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\n");
    let before_feature = h.read("docs/features/FT-001-test.md");
    let before_config = h.read("product.toml");
    h.run(&["migrate", "schema", "--dry-run"]).assert_exit(0);
    let after_feature = h.read("docs/features/FT-001-test.md");
    let after_config = h.read("product.toml");
    assert_eq!(before_feature, after_feature, "dry-run should not modify feature files");
    assert_eq!(before_config, after_config, "dry-run should not modify product.toml");
}

#[test]
fn tc_063_schema_migrate_idempotent() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\n");
    h.run(&["migrate", "schema"]).assert_exit(0);
    let out2 = h.run(&["migrate", "schema"]);
    out2.assert_exit(0);
    // Second run should report 0 files changed (already at current schema)
    assert!(
        out2.stdout.contains("0 files") || out2.stdout.contains("already at") || out2.stdout.contains("up to date"),
        "second run should report no changes needed, got stdout:\n{}",
        out2.stdout
    );
}

#[test]
fn tc_064_schema_migrate_preserves_unknown_fields() {
    let h = Harness::new();
    // Use schema-version "0" to trigger migration
    h.write("product.toml", "name = \"test\"\nschema-version = \"0\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\ncustom-tag: foo\n---\n\nBody.\n");
    h.run(&["migrate", "schema"]).assert_exit(0);
    let content = h.read("docs/features/FT-001-test.md");
    assert!(
        content.contains("custom-tag: foo"),
        "custom-tag should be preserved after migration, got: {}",
        content
    );
}

#[test]
fn tc_065_schema_version_mismatch_format() {
    let h = Harness::new();
    h.write("product.toml", "name = \"test\"\nschema-version = \"99\"\n");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(1)
        .assert_stderr_contains("E008");
    // Check that the error includes declared and supported versions and hint
    assert!(
        out.stderr.contains("99"),
        "E008 should include declared version 99, got: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("hint") || out.stderr.contains("upgrade"),
        "E008 should include an upgrade hint, got: {}",
        out.stderr
    );
}

#[test]
fn tc_617_schema_includes_formal_blocks_section() {
    let h = Harness::new();
    let out = h.run(&["schema"]);
    out.assert_exit(0);

    // Top-level heading must be present …
    assert!(
        out.stdout.contains("## Formal Blocks"),
        "schema output must include '## Formal Blocks' heading; got:\n{}",
        out.stdout
    );

    // … and it must come *after* the Dependency section.
    let dep_idx = out.stdout.find("## Dependency").expect("Dependency heading");
    let fb_idx = out.stdout.find("## Formal Blocks").expect("Formal Blocks heading");
    assert!(
        fb_idx > dep_idx,
        "Formal Blocks section must follow Dependency; dep_idx={} fb_idx={}",
        dep_idx, fb_idx
    );

    // All five AISP block names appear verbatim in parser-accepted and
    // human-readable spellings.
    for name in &[
        "Sigma-Types",
        "Gamma-Invariants",
        "Lambda-Scenario",
        "Lambda-ExitCriteria",
        "Epsilon",
    ] {
        assert!(
            out.stdout.contains(name),
            "formal block section missing '{}'; got:\n{}",
            name, out.stdout
        );
    }
    // And their Unicode block-type labels (authoritative from the parser).
    for label in &[
        "\u{27E6}\u{03A3}:Types\u{27E7}",
        "\u{27E6}\u{0393}:Invariants\u{27E7}",
        "\u{27E6}\u{039B}:Scenario\u{27E7}",
        "\u{27E6}\u{039B}:ExitCriteria\u{27E7}",
        "\u{27E6}\u{0395}\u{27E7}",
    ] {
        assert!(out.stdout.contains(label), "missing Unicode block label '{}'", label);
    }

    // The TC schema cross-references the Formal Blocks section.
    let tc_out = h.run(&["schema", "test"]);
    tc_out.assert_exit(0);
    assert!(
        tc_out.stdout.contains("Formal Blocks"),
        "TC schema should cross-reference 'Formal Blocks'; got:\n{}",
        tc_out.stdout
    );

    // The W004 / G002 contract is named for each mechanic-bearing tc-type.
    for tc_type in &["invariant", "chaos", "exit-criteria"] {
        assert!(
            out.stdout.contains(tc_type),
            "formal block section must name tc-type '{}'",
            tc_type
        );
    }
    assert!(out.stdout.contains("W004"), "W004 contract must be named");
}

#[test]
fn tc_618_schema_type_formal_returns_just_formal_section() {
    let h = Harness::new();

    // Named-flag invocation (the exact form in the TC scenario).
    let out = h.run(&["schema", "--type", "formal"]);
    out.assert_exit(0);

    for name in &[
        "Sigma-Types",
        "Gamma-Invariants",
        "Lambda-Scenario",
        "Lambda-ExitCriteria",
        "Epsilon",
    ] {
        assert!(
            out.stdout.contains(name),
            "formal-only render missing '{}'; got:\n{}",
            name, out.stdout
        );
    }

    // The targeted render must not contain the other top-level schema
    // headings — those belong to the `schema --all` / default render.
    for heading in &["## Feature", "## ADR", "## Test Criterion", "## Dependency"] {
        assert!(
            !out.stdout.contains(heading),
            "formal-only render must not contain '{}'; got:\n{}",
            heading, out.stdout
        );
    }

    // Positional invocation accepts `formal` too (mirrors `schema feature`).
    let out_positional = h.run(&["schema", "formal"]);
    out_positional.assert_exit(0);
    assert!(out_positional.stdout.contains("Sigma-Types"));

    // Unknown types still return a non-zero exit with the existing hint.
    let bad = h.run(&["schema", "--type", "unknown"]);
    assert_ne!(bad.exit_code, 0, "unknown --type should fail; got 0");
    assert!(
        bad.stderr.contains("Unknown artifact type") || bad.stdout.contains("Unknown artifact type"),
        "unknown --type should surface the existing error hint; stdout: {} stderr: {}",
        bad.stdout, bad.stderr
    );
}

