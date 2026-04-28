//! Integration tests — warnings.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_136_w010_unacknowledged_cross_cutting() {
    let h = harness_with_domains();

    // Cross-cutting ADR
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nCross-cutting error model.\n");

    // Feature that neither links nor acknowledges ADR-013
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    let out = h.run(&["graph", "check"]);
    // Should be warning (exit 2) not error
    assert!(
        out.exit_code == 2 || out.stderr.contains("W010"),
        "Expected W010 warning, got exit {} stderr:\n{}",
        out.exit_code, out.stderr
    );
    assert!(
        out.stderr.contains("W010"),
        "Should contain W010 warning code, got stderr:\n{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("FT-009") && out.stderr.contains("ADR-013"),
        "W010 should name FT-009 and ADR-013, got stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_137_w011_domain_gap() {
    let h = harness_with_domains();

    // Domain-scoped security ADR
    h.write("docs/adrs/ADR-020-security-policy.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity policy.\n");

    // Feature declares security domain but doesn't link or acknowledge
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    let out = h.run(&["graph", "check"]);
    assert!(
        out.stderr.contains("W011"),
        "Should contain W011 warning for domain gap, got stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_424_w016_for_accepted_adr_without_content_hash() {
    let h = Harness::new();
    // Create an ADR file manually with status: accepted but no content-hash
    // (simulating a pre-existing ADR that predates this feature)
    h.write(
        "docs/adrs/ADR-001-legacy.md",
        "---\nid: ADR-001\ntitle: Legacy ADR\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nLegacy decision body.\n",
    );

    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W016");

    // When no other errors, exit code should be 2 (warning only)
    // Note: W001 (orphaned) will also fire, but that's also just a warning
    assert_eq!(
        out.exit_code, 2,
        "W016 without errors should give exit code 2.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
}

#[test]
fn tc_443_w017_does_not_fire_for_planned_feature_with_proposed_adr() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["graph", "check"]);
    // W017 should NOT appear for planned features
    assert!(
        !out.stderr.contains("W017"),
        "W017 should not fire for planned features.\nStderr: {}",
        out.stderr
    );
}

#[test]
fn tc_476_w019_suppressed_when_responsibility_field_absent() {
    let h = Harness::new();
    h.write("docs/features/FT-099-grocery.md", "---\nid: FT-099\ntitle: Grocery List Management\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nGrocery lists.\n");
    let out = h.run(&["graph", "check"]);
    assert!(!out.stderr.contains("W019"), "W019 should be suppressed when responsibility absent: {}", out.stderr);
}

#[test]
fn tc_637_w028_fires_when_due_date_passed_and_status_not_complete() {
    let h = fixture_with_domains();
    // FT-009 overdue (1970 is always in the past), in-progress.
    h.write(
        "docs/features/FT-009-overdue.md",
        "---\nid: FT-009\ntitle: Overdue\nphase: 1\nstatus: in-progress\ndue-date: \"1970-01-01\"\ndepends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    // FT-010 overdue but complete — W028 should NOT fire.
    h.write(
        "docs/features/FT-010-complete-past.md",
        "---\nid: FT-010\ntitle: Past Complete\nphase: 1\nstatus: complete\ndue-date: \"1970-01-01\"\ndepends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    h.write(
        "docs/adrs/ADR-045-planning.md",
        "---\nid: ADR-045\ntitle: Planning ADR\nstatus: accepted\nfeatures:\n- FT-009\n- FT-010\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: cross-cutting\n---\n\nSeed.\n",
    );
    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W028");
    // FT-009 overdue message mentions the feature id.
    assert!(
        out.stderr.contains("FT-009"),
        "W028 output should name FT-009: {}",
        out.stderr
    );
    // FT-010 should not be named in W028 output.
    let w028_chunk: String = out
        .stderr
        .split("\n\n")
        .filter(|s| s.contains("W028"))
        .collect::<Vec<_>>()
        .join("\n\n");
    assert!(
        !w028_chunk.contains("FT-010"),
        "complete features must not trigger W028; w028 chunk: {}",
        w028_chunk
    );
    // Exit 2 (W-class only), never 1.
    assert_eq!(
        out.exit_code, 2,
        "W-class only should exit 2; stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_638_w029_fires_within_configurable_warning_window_and_can_be_disabled() {
    let h = fixture_with_domains();
    // Set due-date 1 day in the future (within the 3-day default window).
    let tomorrow = (chrono::Local::now().date_naive()
        + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let far = (chrono::Local::now().date_naive()
        + chrono::Duration::days(90))
        .format("%Y-%m-%d")
        .to_string();
    h.write(
        "docs/features/FT-009-soon.md",
        &format!(
            "---\nid: FT-009\ntitle: Soon\nphase: 1\nstatus: in-progress\ndue-date: \"{}\"\ndepends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\nSeed.\n",
            tomorrow
        ),
    );
    h.write(
        "docs/features/FT-010-far.md",
        &format!(
            "---\nid: FT-010\ntitle: Far\nphase: 1\nstatus: in-progress\ndue-date: \"{}\"\ndepends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\nSeed.\n",
            far
        ),
    );
    h.write(
        "docs/adrs/ADR-045-planning.md",
        "---\nid: ADR-045\ntitle: Planning ADR\nstatus: accepted\nfeatures:\n- FT-009\n- FT-010\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: cross-cutting\n---\n\nSeed.\n",
    );
    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W029");
    assert!(
        out.stderr.contains("FT-009"),
        "W029 should name the near-future FT-009: {}",
        out.stderr
    );
    assert!(
        !out
            .stderr
            .split("\n\n")
            .filter(|s| s.contains("W029"))
            .any(|s| s.contains("FT-010")),
        "W029 should not fire for a date beyond the window: {}",
        out.stderr
    );

    // Disable W029 via [planning].due-date-warning-days = 0.
    let toml = h.read("product.toml");
    h.write(
        "product.toml",
        &format!("{}\n[planning]\ndue-date-warning-days = 0\n", toml),
    );
    let out_disabled = h.run(&["graph", "check"]);
    assert!(
        !out_disabled.stderr.contains("W029"),
        "W029 should be silenced when due-date-warning-days = 0: {}",
        out_disabled.stderr
    );
}

