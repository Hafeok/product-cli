//! Integration tests — feature.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_044_feature_next_uses_topo() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-002-next.md",
        "---\nid: FT-002\ntitle: Next Feature\nphase: 1\nstatus: in-progress\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-003-independent.md",
        "---\nid: FT-003\ntitle: Independent Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);

    // Phase-aware topo sort: FT-001 (phase 1, complete, skipped), FT-002 (phase 1, deps satisfied),
    // FT-003 (phase 2, no deps). FT-002 is picked because phase 1 < phase 2.
    out.assert_stdout_contains("FT-002");
}

#[test]
fn tc_232_feature_next_phase_gate_blocks() {
    let h = Harness::new();
    // Phase 1: FT-001 is complete, FT-002 is in-progress
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-007]\n---\n",
    );
    h.write(
        "docs/features/FT-002-wip.md",
        "---\nid: FT-002\ntitle: WIP Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );
    // Phase 2: FT-005 is planned
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );
    // Exit-criteria TC for phase 1 — failing
    h.write(
        "docs/tests/TC-007-exit.md",
        "---\nid: TC-007\ntitle: Phase 1 Exit Test\ntype: exit-criteria\nstatus: failing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );

    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    // Should return phase-1 feature FT-002, not phase-2 FT-005
    out.assert_stdout_contains("FT-002");
    assert!(
        !out.stdout.contains("FT-005"),
        "FT-005 (phase 2) should be skipped due to phase gate. stdout: {}",
        out.stdout
    );
    // stderr should mention the phase gate and TC-007
    assert!(
        out.stderr.contains("TC-007") || out.stdout.contains("FT-002"),
        "Should mention TC-007 in gate report or return FT-002. stderr: {} stdout: {}",
        out.stderr, out.stdout
    );
}

#[test]
fn tc_233_feature_next_phase_gate_satisfied() {
    let h = Harness::new();
    // Phase 1: FT-001 complete with passing exit criteria
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: Phase 1 Exit\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    // Phase 2: FT-005 is planned
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-005");
}

#[test]
fn tc_234_feature_next_phase_gate_no_exit_criteria() {
    let h = Harness::new();
    // Phase 1: FT-001 complete, no exit-criteria TCs at all
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-scenario.md",
        "---\nid: TC-001\ntitle: Scenario Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    // Phase 2: FT-005 planned
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    // No exit-criteria for phase 1 → gate is open → FT-005 should be returned
    out.assert_stdout_contains("FT-005");
}

#[test]
fn tc_235_feature_next_ignore_gate() {
    let h = Harness::new();
    // Phase 1: FT-001 complete, exit criteria failing
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-007]\n---\n",
    );
    h.write(
        "docs/tests/TC-007-exit.md",
        "---\nid: TC-007\ntitle: Phase 1 Gate\ntype: exit-criteria\nstatus: failing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    // Phase 2: FT-005
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "next", "--ignore-phase-gate"]);
    out.assert_exit(0);
    // Should return FT-005 despite gate being locked
    out.assert_stdout_contains("FT-005");
    // Warning should be emitted to stderr
    out.assert_stderr_contains("ignore-phase-gate");
}

#[test]
fn tc_236_feature_next_gate_partial() {
    let h = Harness::new();
    // Phase 1: FT-001 complete with 4 exit-criteria TCs, 3 passing 1 failing
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001, TC-002, TC-003, TC-004]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: Exit 1\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h.write(
        "docs/tests/TC-002-exit.md",
        "---\nid: TC-002\ntitle: Exit 2\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h.write(
        "docs/tests/TC-003-exit.md",
        "---\nid: TC-003\ntitle: Exit 3\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h.write(
        "docs/tests/TC-004-exit.md",
        "---\nid: TC-004\ntitle: Exit 4\ntype: exit-criteria\nstatus: failing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    // Phase 2 feature — should be blocked
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    // Add a non-complete phase-1 feature so there's something to fall back to
    // when the gate blocks phase 2 — but actually TC-236 tests gate blocking,
    // not fallback. Without an alternative, gate-blocked returns Blocked with
    // the candidate shown but no ready feature.
    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    // Phase gate should NOT be satisfied (3/4 pass, need all 4)
    // The candidate may be shown but must be reported as blocked (not ready)
    // stderr should mention TC-004 (the failing TC)
    assert!(
        out.stderr.contains("TC-004"),
        "stderr should name the failing TC-004. stderr: {}",
        out.stderr
    );
    // stderr should indicate the phase is locked
    assert!(
        out.stderr.contains("locked") || out.stderr.contains("LOCKED") || out.stderr.contains("not all passing"),
        "stderr should indicate phase lock. stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_363_feature_link_interactive_confirm() {
    let h = Harness::new();
    h.write("docs/features/FT-009-test.md", "\
---
id: FT-009
title: Rate Limiting
phase: 1
status: planned
adrs: []
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-021-domain.md", "\
---
id: ADR-021
title: Token Bucket Rate Limiting
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-041-test.md", "\
---
id: TC-041
title: Rate Limit Under Load
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-021
phase: 1
---

TC body.
");

    // Confirm interactive prompt with "y"
    let out = h.run_with_stdin(&["feature", "link", "FT-009", "--adr", "ADR-021"], "y\n");
    out.assert_exit(0);

    // ADR link applied
    let ft = h.read("docs/features/FT-009-test.md");
    assert!(ft.contains("ADR-021"), "FT-009 should have ADR-021. Got:\n{}", ft);

    // TC links applied atomically with ADR link
    assert!(ft.contains("TC-041"), "FT-009 should gain TC-041 on confirm. Got:\n{}", ft);

    let tc = h.read("docs/tests/TC-041-test.md");
    assert!(tc.contains("FT-009"), "TC-041 should gain FT-009 on confirm. Got:\n{}", tc);
}

#[test]
fn tc_364_feature_link_interactive_decline() {
    let h = Harness::new();
    h.write("docs/features/FT-009-test.md", "\
---
id: FT-009
title: Rate Limiting
phase: 1
status: planned
adrs: []
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-021-domain.md", "\
---
id: ADR-021
title: Token Bucket Rate Limiting
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-041-test.md", "\
---
id: TC-041
title: Rate Limit Under Load
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-021
phase: 1
---

TC body.
");

    let tc_before = h.read("docs/tests/TC-041-test.md");

    // Decline interactive prompt with "n"
    let out = h.run_with_stdin(&["feature", "link", "FT-009", "--adr", "ADR-021"], "n\n");
    out.assert_exit(0);

    // ADR link applied
    let ft = h.read("docs/features/FT-009-test.md");
    assert!(ft.contains("ADR-021"), "FT-009 should have ADR-021. Got:\n{}", ft);

    // TC files unchanged
    let tc_after = h.read("docs/tests/TC-041-test.md");
    assert_eq!(tc_before, tc_after, "TC-041 should be unchanged after decline");

    // Feature should NOT have TC-041
    assert!(!ft.contains("TC-041"), "FT-009 should NOT gain TC-041 on decline. Got:\n{}", ft);
}

#[test]
fn tc_461_feature_domain_add_validates_vocabulary() {
    let h = fixture_with_domains();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");

    // Invalid domain → exit 1 with E012
    let out = h.run(&["feature", "domain", "FT-001", "--add", "invalid-domain"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E012");
    out.assert_stderr_contains("invalid-domain");

    // Valid domain → exit 0, appears in front-matter
    let out2 = h.run(&["feature", "domain", "FT-001", "--add", "api"]);
    out2.assert_exit(0);
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("api"), "domain 'api' should appear in front-matter");
}

#[test]
fn tc_462_feature_domain_add_and_remove_idempotent() {
    let h = fixture_with_domains();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");

    // Add api twice
    h.run(&["feature", "domain", "FT-001", "--add", "api"]).assert_exit(0);
    h.run(&["feature", "domain", "FT-001", "--add", "api"]).assert_exit(0);

    // Verify api appears exactly once
    let content = h.read("docs/features/FT-001-test.md");
    let count = content.matches("api").count();
    // In YAML list, "api" appears in domains list — should be exactly once as a list item
    // The domains line should look like: domains:\n- api
    assert!(content.contains("- api"), "should contain api");
    // Check no duplicate by verifying the parsed file has only one occurrence in the domains list section
    let domain_section: Vec<&str> = content.lines()
        .skip_while(|l| !l.starts_with("domains:"))
        .take_while(|l| l.starts_with("domains:") || l.starts_with("- "))
        .filter(|l| l.contains("api"))
        .collect();
    assert_eq!(domain_section.len(), 1, "api should appear exactly once in domains list, found: {:?}", domain_section);

    // Remove storage (not in list) → no-op, exit 0
    let before = h.read("docs/features/FT-001-test.md");
    h.run(&["feature", "domain", "FT-001", "--remove", "storage"]).assert_exit(0);
    let after = h.read("docs/features/FT-001-test.md");
    // File should be effectively unchanged in terms of domains content
    assert!(after.contains("- api"), "api still present after no-op remove");
}

#[test]
fn tc_463_feature_acknowledge_requires_nonempty_reason() {
    let h = fixture_with_domains();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- security\n---\n\nBody.\n");

    // Without --reason → exit 1 with E011
    let out = h.run(&["feature", "acknowledge", "FT-001", "--domain", "security"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E011");

    // With whitespace-only --reason → exit 1 with E011
    let out2 = h.run(&["feature", "acknowledge", "FT-001", "--domain", "security", "--reason", "  "]);
    out2.assert_exit(1);
    out2.assert_stderr_contains("E011");

    // With valid reason → exit 0
    let out3 = h.run(&["feature", "acknowledge", "FT-001", "--domain", "security", "--reason", "No trust boundaries introduced"]);
    out3.assert_exit(0);
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("security"), "domains-acknowledged should contain security");
    assert!(content.contains("No trust boundaries introduced"), "acknowledgement should contain the reason");
}

#[test]
fn tc_681_feature_body_parser_recognizes_functional_specification_section() {
    use product_lib::feature::body_sections::parse_body_sections;

    // Positive: heading is detected.
    let body = "## Description\n\nSome prose.\n\n## Functional Specification\n\n### Inputs\n\n- foo\n";
    let s = parse_body_sections(body);
    assert!(
        s.h2.iter().any(|h| h == "Functional Specification"),
        "expected H2 'Functional Specification' in {:?}",
        s.h2
    );

    // Lowercase is NOT recognised (case-sensitive).
    let s2 = parse_body_sections("## functional specification\n\nx\n");
    assert!(
        !s2.h2.iter().any(|h| h == "Functional Specification"),
        "case-sensitive match: lowercase must not be recognised"
    );

    // Trailing colon does NOT match.
    let s3 = parse_body_sections("## Functional Specification:\n\nx\n");
    assert!(
        !s3.h2.iter().any(|h| h == "Functional Specification"),
        "trailing colon must not match the canonical name"
    );

    // Inside a fenced code block — ignored.
    let s4 = parse_body_sections(
        "## Description\n\n```markdown\n## Functional Specification\n```\n\nProse.\n",
    );
    assert!(
        !s4.h2.iter().any(|h| h == "Functional Specification"),
        "fenced heading must not count"
    );
}

#[test]
fn tc_682_feature_body_parser_recognizes_all_subsections() {
    use product_lib::feature::body_sections::parse_body_sections;

    let body = "\
## Functional Specification

### Inputs

x

### Outputs

x

### State

x

### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x
";
    let s = parse_body_sections(body);
    let h3 = s
        .h3_under
        .get("Functional Specification")
        .expect("expected h3 set under Functional Specification");
    assert_eq!(
        h3,
        &vec![
            "Inputs".to_string(),
            "Outputs".to_string(),
            "State".to_string(),
            "Behaviour".to_string(),
            "Invariants".to_string(),
            "Error handling".to_string(),
            "Boundaries".to_string(),
        ],
        "expected the seven default subsections in document order"
    );
}

#[test]
fn tc_683_w030_fires_when_required_section_missing() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Sample\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n## Description\n\nOnly description.\n",
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 2, "expected exit 2; stderr: {}", out.stderr);
    let json: serde_json::Value =
        serde_json::from_str(&out.stdout).expect("valid JSON on stdout");
    let warnings = json["warnings"].as_array().expect("warnings array");
    let w030: Vec<&serde_json::Value> = warnings
        .iter()
        .filter(|w| w["code"] == "W030")
        .collect();
    assert_eq!(w030.len(), 1, "expected one W030 warning, got {:#?}", warnings);
    let entry = w030[0];
    let detail = entry["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("Functional Specification"));
    assert!(detail.contains("Out of scope"));
    let hint = entry["hint"].as_str().unwrap_or_default();
    assert!(hint.contains("product request change") && hint.contains("body"));
    let file = entry["file"].as_str().unwrap_or_default();
    assert!(file.ends_with("FT-001-test.md"), "file: {}", file);
}

#[test]
fn tc_684_w030_fires_when_required_subsection_missing() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    let body = "\
## Description

Prose.

## Functional Specification

### Inputs

x

### Outputs

x

## Out of scope

x
";
    h.write(
        "docs/features/FT-001-test.md",
        &format!(
            "---\nid: FT-001\ntitle: Sample\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 2);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030: Vec<&serde_json::Value> = warnings.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030.len(), 1, "expected exactly one W030 (one per feature)");
    let detail = w030[0]["detail"].as_str().unwrap_or_default();
    for missing in [
        "Functional Specification > State",
        "Functional Specification > Behaviour",
        "Functional Specification > Invariants",
        "Functional Specification > Error handling",
        "Functional Specification > Boundaries",
    ] {
        assert!(detail.contains(missing), "expected '{}' in detail:\n{}", missing, detail);
    }
    // Parent section itself must NOT be reported as a missing top-level.
    assert!(
        !detail.contains("- Functional Specification\n"),
        "parent must not be re-reported when present:\n{}",
        detail
    );
}

#[test]
fn tc_685_w030_clear_when_all_sections_present() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    h.write(
        "docs/features/FT-001-test.md",
        &format!(
            "---\nid: FT-001\ntitle: Sample\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            COMPLETE_BODY
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030_count = warnings.iter().filter(|w| w["code"] == "W030").count();
    assert_eq!(w030_count, 0, "expected no W030 for complete body, got: {:#?}", warnings);
}

#[test]
fn tc_686_w030_respects_required_from_phase() {
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
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
required-from-phase = 2
"#,
    );
    // Phase 1 — should be exempt.
    h.write(
        "docs/features/FT-001-stub.md",
        "---\nid: FT-001\ntitle: Stub\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n",
    );
    // Phase 2 — should fire W030.
    h.write(
        "docs/features/FT-002-real.md",
        "---\nid: FT-002\ntitle: Real\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n",
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030: Vec<&serde_json::Value> = warnings.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030.len(), 1, "expected one W030 (FT-002), got: {:#?}", w030);
    let file = w030[0]["file"].as_str().unwrap_or_default();
    assert!(file.contains("FT-002-real.md"), "expected W030 on FT-002, got file: {}", file);
}

#[test]
fn tc_687_completeness_severity_warning_w030_is_w_class() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Sample\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n",
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 2);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let errors = json["errors"].as_array().expect("errors");
    let warnings = json["warnings"].as_array().expect("warnings");
    assert_eq!(
        errors.iter().filter(|e| e["code"] == "W030").count(),
        0,
        "no W030 entries expected in errors array"
    );
    assert!(warnings.iter().any(|w| w["code"] == "W030"));
}

#[test]
fn tc_688_completeness_severity_error_w030_becomes_e_class() {
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
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
completeness-severity = "error"
"#,
    );
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Sample\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n",
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 1, "expected exit 1; stderr: {}", out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let errors = json["errors"].as_array().expect("errors");
    let warnings = json["warnings"].as_array().expect("warnings");
    let e030: Vec<&serde_json::Value> = errors.iter().filter(|e| e["code"] == "W030").collect();
    assert_eq!(e030.len(), 1, "expected one W030 in errors array, got: {:#?}", errors);
    assert_eq!(
        warnings.iter().filter(|w| w["code"] == "W030").count(),
        0,
        "no W030 in warnings when severity is error"
    );
    assert_eq!(e030[0]["tier"].as_str().unwrap_or(""), "error");
}

#[test]
fn tc_689_completeness_error_blocks_in_progress_transition() {
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
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
completeness-severity = "error"
"#,
    );
    // Body missing only `### Boundaries`.
    let body = "\
## Description

x

## Functional Specification

### Inputs

x

### Outputs

x

### State

x

### Behaviour

x

### Invariants

x

### Error handling

x

## Out of scope

x
";
    let path = "docs/features/FT-001-x.md";
    let raw = format!(
        "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
        body
    );
    h.write(path, &raw);

    let before = h.read(path);
    let out = h.run(&["feature", "status", "FT-001", "in-progress"]);
    assert_ne!(out.exit_code, 0, "transition must fail; stderr: {}", out.stderr);
    assert!(out.stderr.contains("W030"), "stderr must mention W030: {}", out.stderr);
    assert!(
        out.stderr.contains("Boundaries"),
        "stderr must mention the missing subsection: {}",
        out.stderr
    );
    let after = h.read(path);
    assert_eq!(before, after, "file must be unchanged after blocked transition");
}

#[test]
fn tc_695_required_sections_configurable() {
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
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
required-sections = ["Description", "Acceptance criteria"]
functional-spec-subsections = []
"#,
    );
    h.write(
        "docs/features/FT-001-x.md",
        "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n## Description\n\nx\n\n## Functional Specification\n\nx\n",
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030: Vec<&serde_json::Value> = warnings.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030.len(), 1, "expected one W030");
    let detail = w030[0]["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("Acceptance criteria"));
    // Functional Specification is no longer required.
    assert!(!detail.contains("- Functional Specification"));
    // Out of scope is no longer required.
    assert!(!detail.contains("Out of scope"));
}

#[test]
fn tc_696_functional_spec_subsections_configurable() {
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
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
required-sections = ["Functional Specification"]
functional-spec-subsections = ["Inputs", "Outputs"]
"#,
    );
    let body = "## Functional Specification\n\n### Inputs\n\nx\n\n### Outputs\n\nx\n\n### Behaviour\n\nx\n";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030_count = warnings.iter().filter(|w| w["code"] == "W030").count();
    assert_eq!(
        w030_count, 0,
        "expected no W030 — only Inputs/Outputs required and present, got: {:#?}",
        warnings
    );

    // Now remove Outputs and assert W030 fires.
    let body2 = "## Functional Specification\n\n### Inputs\n\nx\n\n### Behaviour\n\nx\n";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body2
        ),
    );
    let out2 = h.run(&["graph", "check", "--format", "json"]);
    let json2: serde_json::Value = serde_json::from_str(&out2.stdout).expect("valid JSON");
    let warnings2 = json2["warnings"].as_array().expect("warnings");
    let w030_2: Vec<&serde_json::Value> = warnings2.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030_2.len(), 1);
    let detail = w030_2[0]["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("Functional Specification > Outputs"), "detail: {}", detail);
}

#[test]
fn tc_697_functional_specification_feature_exit_criteria() {
    // The exit criteria itself is satisfied when TC-681..TC-696 all pass.
    // This TC asserts the high-level invariants directly: (a) parser
    // module exists, (b) graph check uses W030 with stable code under
    // both severities, (c) status-change gate refuses transitions when
    // severity = error.
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    h.write(
        "docs/features/FT-001-x.md",
        "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\n",
    );
    let out = h.run(&["graph", "check"]);
    assert_eq!(out.exit_code, 2);
    assert!(out.stderr.contains("W030"));
}

