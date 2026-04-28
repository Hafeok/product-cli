//! Integration tests — error_codes.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_058_error_internal_tier4() {
    let h = Harness::new();
    // Remove product.toml to trigger a config-not-found error
    std::fs::remove_file(h.dir.path().join("product.toml")).ok();
    let out = h.run(&["feature", "list"]);
    // Should exit non-zero (config not found is a fatal error)
    assert!(
        out.exit_code != 0,
        "Missing product.toml should produce non-zero exit"
    );
    // Should not panic
    assert!(
        !out.stderr.contains("panicked"),
        "Should not panic on missing config"
    );
}

#[test]
fn tc_059_error_stdout_clean() {
    let h = fixture_orphaned_adr();
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    // stdout should contain the feature listing, not warning diagnostics
    assert!(
        !out.stdout.contains("warning["),
        "Warnings should not appear on stdout: {}",
        out.stdout
    );
    // Warnings should be on stderr
    // (The orphan warning appears during graph check, not feature list,
    // but general principle: stdout is clean of diagnostics)
    assert!(
        !out.stdout.contains("error["),
        "Errors should not appear on stdout: {}",
        out.stdout
    );
}

#[test]
fn tc_055_error_broken_link_format() {
    let h = fixture_broken_link();
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    // File path present on stderr
    assert!(
        out.stderr.contains("FT-001-test.md"),
        "stderr should contain file path, got:\n{}",
        out.stderr
    );
    // Line number present (adrs: [ADR-999] is on line 7 of the fixture)
    assert!(
        out.stderr.contains(":7"),
        "stderr should contain line number, got:\n{}",
        out.stderr
    );
    // Offending content present (the YAML line with the broken reference)
    assert!(
        out.stderr.contains("ADR-999"),
        "stderr should contain offending reference, got:\n{}",
        out.stderr
    );
    // Hint present
    assert!(
        out.stderr.contains("hint:"),
        "stderr should contain a hint, got:\n{}",
        out.stderr
    );
    // Stdout should be empty (all diagnostics on stderr per ADR-013)
    assert!(
        out.stdout.is_empty(),
        "stdout should be empty, got:\n{}",
        out.stdout
    );
}

#[test]
fn tc_056_error_json_format() {
    let h = fixture_error_and_warning();
    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 1, "Expected exit code 1 for broken link");
    // JSON output goes to stdout (command output per ADR-013)
    let json: serde_json::Value = serde_json::from_str(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "Invalid JSON on stdout: {}\nstdout: {}\nstderr: {}",
            e, out.stdout, out.stderr
        )
    });
    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    let warnings = json["warnings"]
        .as_array()
        .expect("warnings should be an array");
    assert_eq!(errors.len(), 1, "Expected 1 error, got: {:?}", errors);
    assert_eq!(
        warnings.len(),
        1,
        "Expected 1 warning, got: {:?}",
        warnings
    );
    // Verify summary counts match
    assert_eq!(json["summary"]["errors"], 1);
    assert_eq!(json["summary"]["warnings"], 1);
}

#[test]
fn tc_057_error_no_panic_on_bad_yaml() {
    let h = Harness::new();
    // File with completely invalid YAML front-matter
    h.write(
        "docs/features/bad.md",
        "---\n{{{not: valid: yaml: [[[unterminated\n---\n\nBody.\n",
    );
    let out = h.run(&["graph", "check"]);
    assert_eq!(
        out.exit_code, 1,
        "Expected exit 1 for bad YAML.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    // Structured error on stderr (E001 for malformed front-matter)
    assert!(
        out.stderr.contains("error[E001]") || out.stderr.contains("E001"),
        "Expected structured E001 error on stderr, got:\n{}",
        out.stderr
    );
    // No panic
    assert!(
        !out.stderr.contains("panicked"),
        "Should not panic on bad YAML"
    );
    assert!(
        !out.stderr.contains("thread 'main' panicked"),
        "Should not panic on bad YAML"
    );
}

#[test]
fn tc_421_e014_on_accepted_adr_body_tamper() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    // Create and accept an ADR
    h.run(&["adr", "new", "Immutable ADR"]).assert_exit(0);

    let adr_dir = h.dir.path().join("docs/adrs");
    let entries: Vec<_> = std::fs::read_dir(&adr_dir)
        .expect("read")
        .filter_map(|e| e.ok())
        .collect();
    let adr_path = entries[0].path();
    let filename = adr_path.file_name().expect("fname").to_str().expect("utf8");
    let adr_id = &filename[..7];

    h.run(&["adr", "status", adr_id, "accepted"]).assert_exit(0);

    // Tamper with the body
    let content = std::fs::read_to_string(&adr_path).expect("read");
    let tampered = format!("{}\nThis is an unauthorized addition.\n", content.trim_end());
    std::fs::write(&adr_path, tampered).expect("write tampered");

    // graph check should emit E014
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E014");

    // Now test title tamper
    let content = std::fs::read_to_string(&adr_path).expect("read");
    let title_tampered = content.replace("title: Immutable ADR", "title: Changed Title");
    std::fs::write(&adr_path, title_tampered).expect("write title tampered");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E014");
}

#[test]
fn tc_422_e015_on_sealed_tc_body_tamper() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nBody.\n",
    );

    // Create a TC manually with body content
    let tc_body = "---\nid: TC-001\ntitle: Sealed Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n## Description\n\nThis is a detailed test specification.\n";
    h.write("docs/tests/TC-001-sealed-test.md", tc_body);

    // Seal the TC
    let out = h.run(&["hash", "seal", "TC-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("sealed");

    // Verify content-hash was written
    let tc_content = h.read("docs/tests/TC-001-sealed-test.md");
    assert!(
        tc_content.contains("content-hash: sha256:"),
        "Sealed TC should have content-hash.\nGot:\n{}",
        tc_content
    );

    // Tamper with the body
    let tampered = tc_content.replace(
        "This is a detailed test specification.",
        "This specification has been tampered with.",
    );
    std::fs::write(
        h.dir.path().join("docs/tests/TC-001-sealed-test.md"),
        tampered,
    )
    .expect("write tampered");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E015");

    // Test protected field tamper (type)
    let tc_content = h.read("docs/tests/TC-001-sealed-test.md");
    let type_tampered = tc_content.replace("type: scenario", "type: invariant");
    std::fs::write(
        h.dir.path().join("docs/tests/TC-001-sealed-test.md"),
        type_tampered,
    )
    .expect("write type tampered");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E015");

    // Test protected field tamper (validates.adrs)
    let tc_content = h.read("docs/tests/TC-001-sealed-test.md");
    let adrs_tampered = tc_content.replace("adrs: []", "adrs: [ADR-999]");
    std::fs::write(
        h.dir.path().join("docs/tests/TC-001-sealed-test.md"),
        adrs_tampered,
    )
    .expect("write adrs tampered");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E015");
}

#[test]
fn tc_446_e016_names_all_proposed_adrs_not_just_the_first() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: First Proposed ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/adrs/ADR-002-test.md",
        "---\nid: ADR-002\ntitle: Second Proposed ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E016");
    // Both ADRs should be named
    out.assert_stderr_contains("ADR-001");
    out.assert_stderr_contains("ADR-002");
}

#[test]
fn tc_610_e017_reserved_type_in_custom_list() {
    let h = ft048_tc_types(&["contract", "exit-criteria"]);
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E017");
    out.assert_stderr_contains("exit-criteria");
}

#[test]
fn tc_611_e017_fires_at_startup_not_lazily() {
    let h = ft048_tc_types(&["invariant"]);
    let out = h.run(&["--help"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E017");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E017");
}

