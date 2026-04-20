//! ST-147 — FT-047 removal & deprecation tracking (ADR-041).

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use std::process::{Command, Stdio};

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

#[test]
fn tc_586_absence_tc_passes_when_thing_gone() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    let script = write_exit_script(&s, "pass", 0);
    write_tc(&s, "TC-900", "Absence Passes", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let status = read_tc_status(&s, "docs/tests/TC-900-absence-passes.md");
    assert_eq!(status, "passing");
}

#[test]
fn tc_587_absence_tc_fails_when_thing_present() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    let script = write_exit_script(&s, "fail", 1);
    write_tc(&s, "TC-901", "Absence Fails", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 1, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let status = read_tc_status(&s, "docs/tests/TC-901-absence-fails.md");
    assert_eq!(status, "failing");
}

#[test]
fn tc_588_absence_tc_runs_in_platform_verify() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted", &["foo"], &[]);
    write_feature(&s, "FT-900", "Plain Feature", &["ADR-100"], &["TC-910"]);
    let fs = write_exit_script(&s, "feat_pass", 0);
    write_tc(&s, "TC-910", "Feature Scenario", "scenario", &["FT-900"], &[],
        Some(("bash", &fs)), "unimplemented");
    let script = write_exit_script(&s, "abs_pass", 0);
    write_tc(&s, "TC-911", "Absence Cross Cutting", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert_eq!(
        read_tc_status(&s, "docs/tests/TC-911-absence-cross-cutting.md"),
        "passing"
    );
    assert_ne!(
        read_tc_status(&s, "docs/tests/TC-910-feature-scenario.md"),
        "passing"
    );
}

#[test]
fn tc_589_adr_removes_field_parses_correctly() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Decision", "accepted",
        &["AutoMapper NuGet package", "IMapper interface usage"], &[]);
    let r = Run::run(&s, &["adr", "show", "ADR-100"]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.contains("AutoMapper NuGet package"), "stdout:\n{}", r.stdout);
    assert!(r.stdout.contains("IMapper interface usage"), "stdout:\n{}", r.stdout);

    write_adr_body(&s, "ADR-101", "Empty ADR", "accepted", &[], &[]);
    let r = Run::run(&s, &["graph", "check"]);
    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "graph check exit 0 or 2; got {}; stderr:\n{}",
        r.exit_code, r.stderr
    );
}

#[test]
fn tc_590_adr_deprecates_field_parses_correctly() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecation Decision", "accepted", &[],
        &["source-files", "old-field"]);
    let r = Run::run(&s, &["adr", "show", "ADR-100"]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.contains("source-files"), "stdout:\n{}", r.stdout);
    assert!(r.stdout.contains("old-field"), "stdout:\n{}", r.stdout);
}

#[test]
fn tc_591_g009_fires_when_removes_no_absence_tc() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Without Absence TC", "accepted", &["foo"], &[]);
    write_feature(&s, "FT-900", "Feat", &["ADR-100"], &["TC-910"]);
    write_tc(&s, "TC-910", "Regular", "scenario", &["FT-900"], &["ADR-100"],
        None, "unimplemented");

    let r = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(r.stdout.contains("G009"), "stdout:\n{}", r.stdout);
    assert_eq!(r.exit_code, 1);

    let s2 = Session::new();
    write_adr_body(&s2, "ADR-100", "Deprecation Without Absence TC", "accepted", &[], &["old-field"]);
    let r2 = Run::run(&s2, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(r2.stdout.contains("G009"), "stdout:\n{}", r2.stdout);
}

#[test]
fn tc_592_w022_fires_same_condition() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Without Absence TC", "accepted", &["foo"], &[]);
    write_feature(&s, "FT-900", "Feat", &["ADR-100"], &[]);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(r.stderr.contains("W022"), "stderr:\n{}", r.stderr);
    assert!(r.exit_code == 0 || r.exit_code == 2, "exit {}; stderr:\n{}", r.exit_code, r.stderr);
}

#[test]
fn tc_593_g009_clear_when_absence_tc_linked() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal Linked", "accepted", &["foo"], &[]);
    let script = write_exit_script(&s, "pass", 0);
    write_tc(&s, "TC-920", "Absence Linked", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "passing");

    let r_gap = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(!r_gap.stdout.contains("G009"), "stdout:\n{}", r_gap.stdout);

    let r_graph = Run::run(&s, &["graph", "check"]);
    assert!(!r_graph.stderr.contains("W022"), "stderr:\n{}", r_graph.stderr);
}

#[test]
fn tc_594_w023_fires_on_deprecated_field() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecation", "accepted", &[], &["source-files"]);
    let victim = "---\nid: ADR-101\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nsource-files:\n  - src/foo.rs\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-101-victim.md", victim);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(r.stderr.contains("W023"), "stderr:\n{}", r.stderr);
    assert!(r.stderr.contains("source-files"), "stderr:\n{}", r.stderr);
    assert!(r.exit_code == 0 || r.exit_code == 2);
}

#[test]
fn tc_595_deprecated_field_still_processed_for_compat() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecation", "accepted", &[], &["source-files"]);
    let victim = "---\nid: ADR-101\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nsource-files:\n  - src/foo.rs\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-101-victim.md", victim);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(r.exit_code == 0 || r.exit_code == 2);
    let raw = s.read("docs/adrs/ADR-101-victim.md");
    assert!(raw.contains("source-files"));
    let r2 = Run::run(&s, &["adr", "show", "ADR-101"]);
    assert_eq!(r2.exit_code, 0);
    assert!(r.stderr.matches("W023").count() >= 1, "stderr:\n{}", r.stderr);
}

#[test]
fn tc_596_w023_names_deprecating_adr() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecates foo", "accepted", &[], &["foo"]);
    write_adr_body(&s, "ADR-200", "Deprecates bar", "accepted", &[], &["bar"]);
    let victim = "---\nid: ADR-300\ntitle: Victim\nstatus: accepted\nfeatures: []\ndomains: [api]\nscope: domain\nfoo: one\nbar: two\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** r.\n\n**Rejected alternatives:** none.\n";
    s.write("docs/adrs/ADR-300-victim.md", victim);

    let r = Run::run(&s, &["graph", "check"]);
    assert!(r.stderr.contains("W023"), "stderr:\n{}", r.stderr);
    assert!(r.stderr.contains("foo") && r.stderr.contains("ADR-100"), "stderr:\n{}", r.stderr);
    assert!(r.stderr.contains("bar") && r.stderr.contains("ADR-200"), "stderr:\n{}", r.stderr);
}

#[test]
fn tc_597_migration_phase1_deprecation_tc_passes() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Deprecation", "accepted", &[], &["old-api"]);
    let script = write_exit_script(&s, "warning_observed", 0);
    write_tc(&s, "TC-930", "Phase1 Deprecation", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");
    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 0);
    assert_eq!(read_tc_status(&s, "docs/tests/TC-930-phase1-deprecation.md"), "passing");
}

#[test]
fn tc_598_migration_phase2_absence_tc_passes() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal", "accepted", &["old-dep"], &[]);
    let script = write_exit_script(&s, "absent", 0);
    write_tc(&s, "TC-940", "Phase2 Absence", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");
    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 0);
    assert_eq!(read_tc_status(&s, "docs/tests/TC-940-phase2-absence.md"), "passing");
}

#[test]
fn tc_599_migration_phase2_phase1_tc_unrunnable_no_block() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Removal", "accepted", &["old-dep"], &[]);
    let dummy = write_exit_script(&s, "dummy", 0);
    write_tc(&s, "TC-950", "Phase1 Superseded", "absence", &[], &["ADR-100"],
        Some(("bash", &dummy)), "unrunnable");
    let script = write_exit_script(&s, "absent2", 0);
    write_tc(&s, "TC-951", "Phase2 Absence", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "unimplemented");

    let r = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r.exit_code, 0, "exit={}; stderr:\n{}", r.exit_code, r.stderr);
    assert_eq!(read_tc_status(&s, "docs/tests/TC-951-phase2-absence.md"), "passing");
    let rg = Run::run(&s, &["graph", "check"]);
    assert!(rg.exit_code == 0 || rg.exit_code == 2);
}

#[test]
fn tc_600_removal_deprecation_exit() {
    let s = Session::new();
    write_adr_body(&s, "ADR-100", "Full Removal Deprecation", "accepted",
        &["AutoMapper"], &["old-field"]);
    let script = write_exit_script(&s, "ok", 0);
    write_tc(&s, "TC-999", "Absence Final", "absence", &[], &["ADR-100"],
        Some(("bash", &script)), "passing");

    let r_gap = Run::run(&s, &["gap", "check", "ADR-100", "--format", "json"]);
    assert!(!r_gap.stdout.contains("G009"), "stdout:\n{}", r_gap.stdout);

    let r_graph = Run::run(&s, &["graph", "check"]);
    assert!(!r_graph.stderr.contains("W022"), "stderr:\n{}", r_graph.stderr);
    assert!(r_graph.exit_code == 0 || r_graph.exit_code == 2);

    let r_pv = Run::run(&s, &["verify", "--platform"]);
    assert_eq!(r_pv.exit_code, 0, "stderr:\n{}", r_pv.stderr);
}
