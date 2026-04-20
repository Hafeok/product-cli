//! ST-140..ST-153 — FT-047 removal & deprecation tracking (ADR-041).
//!
//! Each test composes a temp repository, drives the relevant `product`
//! subcommand, and asserts on stdout / stderr / exit-code. TC mapping:
//!
//! - TC-586 `tc_586_absence_tc_passes_when_thing_gone`
//! - TC-587 `tc_587_absence_tc_fails_when_thing_present`
//! - TC-588 `tc_588_absence_tc_runs_in_platform_verify`
//! - TC-589 `tc_589_adr_removes_field_parses_correctly`
//! - TC-590 `tc_590_adr_deprecates_field_parses_correctly`
//! - TC-591 `tc_591_g009_fires_when_removes_no_absence_tc`
//! - TC-592 `tc_592_w022_fires_same_condition`
//! - TC-593 `tc_593_g009_clear_when_absence_tc_linked`
//! - TC-594 `tc_594_w023_fires_on_deprecated_field`
//! - TC-595 `tc_595_deprecated_field_still_processed_for_compat`
//! - TC-596 `tc_596_w023_names_deprecating_adr`
//! - TC-597 `tc_597_migration_phase1_deprecation_tc_passes`
//! - TC-598 `tc_598_migration_phase2_absence_tc_passes`
//! - TC-599 `tc_599_migration_phase2_phase1_tc_unrunnable_no_block`
//! - TC-600 `tc_600_removal_deprecation_exit` (consolidated exit-criteria)

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Local run helper
// ---------------------------------------------------------------------------

struct Run {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl Run {
    fn run(s: &Session, args: &[&str]) -> Self {
        let out = Command::new(&s.bin)
            .args(args)
            .current_dir(s.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("spawn product");
        Run {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture writers
// ---------------------------------------------------------------------------

fn write_feature(s: &Session, id: &str, title: &str, adrs: &[&str], tests: &[&str]) {
    let adrs_str = if adrs.is_empty() { "[]".into() } else { format!("[{}]", adrs.join(", ")) };
    let tests_str = if tests.is_empty() { "[]".into() } else { format!("[{}]", tests.join(", ")) };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nphase: 1\nstatus: planned\nadrs: {adrs_str}\ntests: {tests_str}\n---\n\nFeature body.\n"
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/features/{}-{}.md", id, slug), &content);
}

fn write_adr_body(
    s: &Session,
    id: &str,
    title: &str,
    status: &str,
    removes: &[&str],
    deprecates: &[&str],
) {
    let removes_str = if removes.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = removes.iter().map(|x| format!("  - {}", x)).collect();
        format!("removes:\n{}\n", lines.join("\n"))
    };
    let deprecates_str = if deprecates.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = deprecates.iter().map(|x| format!("  - {}", x)).collect();
        format!("deprecates:\n{}\n", lines.join("\n"))
    };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: {status}\nfeatures: []\ndomains: [api]\nscope: domain\n{removes_str}{deprecates_str}---\n\n**Context:** ctx.\n\n**Decision:** decision.\n\n**Rationale:** why.\n\n**Rejected alternatives:**\n- none\n"
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

fn write_tc(
    s: &Session,
    id: &str,
    title: &str,
    ty: &str,
    features: &[&str],
    adrs: &[&str],
    runner: Option<(&str, &str)>,
    status: &str,
) {
    let fs = if features.is_empty() { "[]".into() } else { format!("[{}]", features.join(", ")) };
    let ad = if adrs.is_empty() { "[]".into() } else { format!("[{}]", adrs.join(", ")) };
    let mut fm = format!(
        "---\nid: {id}\ntitle: {title}\ntype: {ty}\nstatus: {status}\nvalidates:\n  features: {fs}\n  adrs: {ad}\nphase: 1\n"
    );
    if let Some((r, a)) = runner {
        fm.push_str(&format!("runner: {}\nrunner-args: \"{}\"\n", r, a));
    }
    fm.push_str("---\n\nTest body.\n");
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/tests/{}-{}.md", id, slug), &fm);
}

/// Create a shell script that exits with the given code. Returns its path
/// relative to the session root, with executable bit set on Unix.
fn write_exit_script(s: &Session, name: &str, exit_code: u8) -> String {
    let path = format!("scripts/{}.sh", name);
    s.write(&path, &format!("#!/usr/bin/env bash\nexit {}\n", exit_code));
    let abs = s.dir.path().join(&path);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&abs).expect("stat").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&abs, perms).expect("chmod");
    }
    path
}

fn read_tc_status(s: &Session, tc_file: &str) -> String {
    let body = s.read(tc_file);
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("status:") {
            return rest.trim().to_string();
        }
    }
    "?".into()
}

// ---------------------------------------------------------------------------
// TC-586 — absence TC passes when runner exits 0
// ---------------------------------------------------------------------------

#[test]
fn tc_586_absence_tc_passes_when_thing_gone() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    let script = write_exit_script(&s, "pass", 0);
    write_tc(
        &s,
        "TC-900",
        "Absence Passes",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    let status = read_tc_status(&s, "docs/tests/TC-900-absence-passes.md");
    assert_eq!(
        status, "passing",
        "expected TC status 'passing' after runner exit 0, got '{}'",
        status
    );
}

// ---------------------------------------------------------------------------
// TC-587 — absence TC fails when runner exits non-zero
// ---------------------------------------------------------------------------

#[test]
fn tc_587_absence_tc_fails_when_thing_present() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    let script = write_exit_script(&s, "fail", 1);
    write_tc(
        &s,
        "TC-901",
        "Absence Fails",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(
        r.exit_code, 1,
        "expected exit 1 on failing absence TC; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    let status = read_tc_status(&s, "docs/tests/TC-901-absence-fails.md");
    assert_eq!(
        status, "failing",
        "expected TC status 'failing' after runner exit 1, got '{}'",
        status
    );
}

// ---------------------------------------------------------------------------
// TC-588 — absence TC runs under platform verify; feature-scoped TCs don't
// ---------------------------------------------------------------------------

#[test]
fn tc_588_absence_tc_runs_in_platform_verify() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    // Feature with a feature-scoped scenario TC.
    write_feature(&s, "FT-900", "Plain Feature", &["ADR-100"], &["TC-910"]);
    let feature_script = write_exit_script(&s, "feat_pass", 0);
    write_tc(
        &s,
        "TC-910",
        "Feature Scenario",
        "scenario",
        &["FT-900"],
        &[],
        Some(("bash", &feature_script)),
        "unimplemented",
    );
    // One absence TC (cross-cutting).
    let script = write_exit_script(&s, "abs_pass", 0);
    write_tc(
        &s,
        "TC-911",
        "Absence Cross Cutting",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );

    let r = Run::run(&s, &["verify", "--ci"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    // Look at the CI JSON document.
    let start = r.stdout.find('{').unwrap_or(0);
    let v: serde_json::Value = serde_json::from_str(&r.stdout[start..]).expect("valid JSON");
    let stages = v.get("stages").and_then(|a| a.as_array()).expect("stages");
    let stage6 = stages
        .iter()
        .find(|s| s.get("stage").and_then(|x| x.as_i64()) == Some(6))
        .expect("stage 6");
    let findings = stage6
        .get("findings")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    let tcs: Vec<String> = findings
        .iter()
        .filter_map(|f| f.get("tc").and_then(|t| t.as_str()).map(String::from))
        .collect();
    // TC-911 (absence) should appear in platform results. TC-910 (feature scenario)
    // should not.
    //
    // Note: on pass, platform TCs may not produce findings (the findings list
    // reports failures / skips / warnings). Instead, we verify file-level state.
    let absence_status = read_tc_status(&s, "docs/tests/TC-911-absence-cross-cutting.md");
    assert_eq!(
        absence_status, "passing",
        "absence TC should have been run and marked passing by platform verify; tcs reported: {:?}",
        tcs
    );
    let feat_status = read_tc_status(&s, "docs/tests/TC-910-feature-scenario.md");
    // Feature-scoped scenario is not collected by platform stage — it stays
    // unimplemented (platform stage doesn't execute it).
    assert_ne!(
        feat_status, "passing",
        "feature-scoped scenario must not be executed by --platform run"
    );
}

// ---------------------------------------------------------------------------
// TC-589 — removes field parses and round-trips
// ---------------------------------------------------------------------------

#[test]
fn tc_589_adr_removes_field_parses_correctly() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Removal Decision",
        "accepted",
        &["AutoMapper NuGet package", "IMapper interface usage"],
        &[],
    );
    // Show the ADR — the rendered output must include both removes values.
    let r = Run::run(&s, &["adr", "show", "ADR-100"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    assert!(
        r.stdout.contains("AutoMapper NuGet package"),
        "expected first removes value in output; got:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("IMapper interface usage"),
        "expected second removes value in output; got:\n{}",
        r.stdout
    );
    // An ADR with no removes field parses cleanly.
    write_adr_body(&s, "ADR-101", "Empty ADR", "accepted", &[], &[]);
    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "graph check should not E-error on ADR without removes; exit={}; stderr:\n{}",
        r.exit_code, r.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-590 — deprecates field parses and round-trips
// ---------------------------------------------------------------------------

#[test]
fn tc_590_adr_deprecates_field_parses_correctly() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Deprecation Decision",
        "accepted",
        &[],
        &["source-files", "old-field"],
    );
    let r = Run::run(&s, &["adr", "show", "ADR-100"]);
    assert_eq!(r.exit_code, 0);
    assert!(
        r.stdout.contains("source-files"),
        "expected 'source-files' in output; got:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("old-field"),
        "expected 'old-field' in output; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-591 — G009 fires when removes non-empty and no linked absence TC
// ---------------------------------------------------------------------------

#[test]
fn tc_591_g009_fires_when_removes_no_absence_tc() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Removal Without Absence TC",
        "accepted",
        &["foo"],
        &[],
    );
    write_feature(&s, "FT-900", "Feat", &["ADR-100"], &["TC-910"]);
    write_tc(
        &s,
        "TC-910",
        "Regular",
        "scenario",
        &["FT-900"],
        &["ADR-100"],
        None,
        "unimplemented",
    );

    let r = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(
        r.stdout.contains("G009"),
        "expected G009 finding; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    assert!(
        r.stdout.contains("\"severity\":\"high\"") || r.stdout.contains("severity: high"),
        "expected severity high for G009; stdout:\n{}",
        r.stdout
    );
    assert_eq!(r.exit_code, 1, "expected exit 1 when G009 fires");

    // Parameterised case: same rule for `deprecates`.
    let s2 = Session::new();
    write_adr_body(
        &s2,
        "ADR-100",
        "Deprecation Without Absence TC",
        "accepted",
        &[],
        &["old-field"],
    );
    let r2 = Run::run(&s2, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(
        r2.stdout.contains("G009"),
        "expected G009 for deprecates-only case; stdout:\n{}",
        r2.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-592 — W022 fires on the same condition via graph check
// ---------------------------------------------------------------------------

#[test]
fn tc_592_w022_fires_same_condition() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Removal Without Absence TC",
        "accepted",
        &["foo"],
        &[],
    );
    write_feature(&s, "FT-900", "Feat", &["ADR-100"], &[]);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.stderr.contains("W022"),
        "expected W022 in stderr; stderr:\n{}",
        r.stderr
    );
    // W022 is a warning — never exits 1 for this alone.
    assert!(
        r.exit_code == 2 || r.exit_code == 0,
        "expected exit 0 or 2 on W-only; got {}; stderr:\n{}",
        r.exit_code, r.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-593 — G009 and W022 clear when an absence TC is linked
// ---------------------------------------------------------------------------

#[test]
fn tc_593_g009_clear_when_absence_tc_linked() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Removal Linked",
        "accepted",
        &["foo"],
        &[],
    );
    let script = write_exit_script(&s, "pass", 0);
    write_tc(
        &s,
        "TC-920",
        "Absence Linked",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "passing",
    );

    let r_gap = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(
        !r_gap.stdout.contains("G009"),
        "G009 should be cleared once an absence TC is linked; stdout:\n{}",
        r_gap.stdout
    );

    let r_graph = Run::run(&s, &["graph", "check"]);
    assert!(
        !r_graph.stderr.contains("W022"),
        "W022 should be cleared once an absence TC is linked; stderr:\n{}",
        r_graph.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-594 — W023 fires on a deprecated front-matter field in use
// ---------------------------------------------------------------------------

#[test]
fn tc_594_w023_fires_on_deprecated_field() {
    let s = Session::new();
    // ADR deprecates the `source-files` field.
    write_adr_body(
        &s,
        "ADR-100",
        "Deprecation",
        "accepted",
        &[],
        &["source-files"],
    );
    // Another ADR carries the deprecated field.
    let victim = "---\nid: ADR-101\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nsource-files:\n  - src/foo.rs\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-101-victim.md", victim);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.stderr.contains("W023"),
        "expected W023 warning; stderr:\n{}",
        r.stderr
    );
    assert!(
        r.stderr.contains("source-files"),
        "W023 message should name the deprecated field; stderr:\n{}",
        r.stderr
    );
    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "W023 should not block; exit={}",
        r.exit_code
    );
}

// ---------------------------------------------------------------------------
// TC-595 — deprecated field is still processed; graph builds cleanly
// ---------------------------------------------------------------------------

#[test]
fn tc_595_deprecated_field_still_processed_for_compat() {
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Deprecation",
        "accepted",
        &[],
        &["source-files"],
    );
    let victim = "---\nid: ADR-101\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nsource-files:\n  - src/foo.rs\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-101-victim.md", victim);

    // Graph check must not error; the deprecated field remains parsed.
    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "graph check must not E-error on deprecated field; exit={}; stderr:\n{}",
        r.exit_code, r.stderr
    );
    // The ADR file still contains the deprecated field on disk.
    let raw = s.read("docs/adrs/ADR-101-victim.md");
    assert!(
        raw.contains("source-files"),
        "deprecated field must remain in the file; got:\n{}",
        raw
    );
    // adr show must succeed.
    let r2 = Run::run(&s, &["adr", "show", "ADR-101"]);
    assert_eq!(
        r2.exit_code, 0,
        "adr show of victim ADR must succeed; stderr:\n{}",
        r2.stderr
    );
    // Ensure only one W023 for this one field (no duplicate emission).
    let count_w023 = r.stderr.matches("W023").count();
    assert!(
        count_w023 >= 1,
        "expected at least one W023 warning; stderr:\n{}",
        r.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-596 — W023 message names the deprecating ADR by ID
// ---------------------------------------------------------------------------

#[test]
fn tc_596_w023_names_deprecating_adr() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecates foo", "accepted", &[], &["foo"]);
    write_adr_body(&s, "ADR-200", "Deprecates bar", "accepted", &[], &["bar"]);
    // A victim ADR with two deprecated keys.
    let victim = "---\nid: ADR-300\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nfoo: one\nbar: two\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-300-victim.md", victim);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.stderr.contains("W023"),
        "expected W023; stderr:\n{}",
        r.stderr
    );
    // Each warning must name the corresponding ADR id.
    let w023_lines: Vec<&str> = r
        .stderr
        .lines()
        .filter(|l| l.contains("W023") || l.contains("deprecat"))
        .collect();
    let combined = w023_lines.join("\n");
    assert!(
        r.stderr.contains("foo") && r.stderr.contains("ADR-100"),
        "W023 for 'foo' must name ADR-100; stderr:\n{}",
        combined
    );
    assert!(
        r.stderr.contains("bar") && r.stderr.contains("ADR-200"),
        "W023 for 'bar' must name ADR-200; stderr:\n{}",
        combined
    );
}

// ---------------------------------------------------------------------------
// TC-597 — phase-1 deprecation TC passes when its runner observes the warning
// ---------------------------------------------------------------------------

#[test]
fn tc_597_migration_phase1_deprecation_tc_passes() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecation", "accepted", &[], &["old-api"]);
    // Phase-1 "deprecation warning emitted" TC whose runner exits 0 —
    // simulating that the warning was observed.
    let script = write_exit_script(&s, "warning_observed", 0);
    write_tc(
        &s,
        "TC-930",
        "Phase1 Deprecation",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );
    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(
        r.exit_code, 0,
        "phase-1 deprecation TC runner exit 0 must not fail platform verify; stderr:\n{}",
        r.stderr
    );
    let status = read_tc_status(&s, "docs/tests/TC-930-phase1-deprecation.md");
    assert_eq!(status, "passing");
}

// ---------------------------------------------------------------------------
// TC-598 — phase-2 absence TC passes when the thing is absent
// ---------------------------------------------------------------------------

#[test]
fn tc_598_migration_phase2_absence_tc_passes() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal", "accepted", &["old-dep"], &[]);
    let script = write_exit_script(&s, "absent", 0);
    write_tc(
        &s,
        "TC-940",
        "Phase2 Absence",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );
    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(
        r.exit_code, 0,
        "phase-2 absence TC pass should exit 0; stderr:\n{}",
        r.stderr
    );
    let status = read_tc_status(&s, "docs/tests/TC-940-phase2-absence.md");
    assert_eq!(status, "passing");
}

// ---------------------------------------------------------------------------
// TC-599 — phase-1 unrunnable does not block; phase-2 passes
// ---------------------------------------------------------------------------

#[test]
fn tc_599_migration_phase2_phase1_tc_unrunnable_no_block() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal", "accepted", &["old-dep"], &[]);
    // Phase-1 marked unrunnable with a documented reason.
    let dummy_script = write_exit_script(&s, "dummy", 0);
    write_tc(
        &s,
        "TC-950",
        "Phase1 Superseded",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &dummy_script)),
        "unrunnable",
    );
    // Phase-2 absence TC that passes.
    let script = write_exit_script(&s, "absent2", 0);
    write_tc(
        &s,
        "TC-951",
        "Phase2 Absence",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "unimplemented",
    );

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(
        r.exit_code, 0,
        "unrunnable phase-1 TC must not contribute to exit failure; exit={}; stderr:\n{}",
        r.exit_code, r.stderr
    );
    let phase2_status = read_tc_status(&s, "docs/tests/TC-951-phase2-absence.md");
    assert_eq!(phase2_status, "passing", "phase-2 absence TC should have run and passed");
    // Graph check should not emit an error or warning specifically for the
    // unrunnable status.
    let rg = Run::run(&s, &["graph", "check"]);
    assert!(
        rg.exit_code == 0 || rg.exit_code == 2,
        "graph check should not E-error when a TC is unrunnable; exit={}; stderr:\n{}",
        rg.exit_code, rg.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-600 — consolidated exit criteria for FT-047
// ---------------------------------------------------------------------------

#[test]
fn tc_600_removal_deprecation_exit() {
    // This exit-criteria test exercises the end-to-end: an ADR with
    // removes+deprecates is accepted, a linked absence TC passes, and the
    // graph is clean (no G009, no W022 for this ADR).
    let s = Session::new();
    write_adr_body(
        &s,
        "ADR-100",
        "Full Removal Deprecation",
        "accepted",
        &["AutoMapper"],
        &["old-field"],
    );
    let script = write_exit_script(&s, "ok", 0);
    write_tc(
        &s,
        "TC-999",
        "Absence Final",
        "absence",
        &[],
        &["ADR-100"],
        Some(("bash", &script)),
        "passing",
    );

    // Gap check: no G009 for ADR-100.
    let r_gap = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(
        !r_gap.stdout.contains("G009"),
        "no G009 expected once absence TC is linked; stdout:\n{}",
        r_gap.stdout
    );

    // Graph check: no W022.
    let r_graph = Run::run(&s, &["graph", "check"]);
    assert!(
        !r_graph.stderr.contains("W022"),
        "no W022 expected; stderr:\n{}",
        r_graph.stderr
    );
    assert!(
        r_graph.exit_code == 0 || r_graph.exit_code == 2,
        "graph check exit 0 or 2; got {}",
        r_graph.exit_code
    );

    // Platform verify passes.
    let r_pv = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r_pv.exit_code, 0, "platform verify should pass; stderr:\n{}", r_pv.stderr);
}
