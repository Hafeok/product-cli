//! ST-110..ST-119 — unified verify pipeline (FT-044, ADR-040).
//!
//! Each test composes a temp repository, drives `product verify` (with or
//! without scope flags) through a controlled state, and asserts on the JSON
//! output + exit code. Runner configuration uses the `bash` runner with short
//! inline scripts so we don't spawn nested `cargo test` processes.

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct VerifyRun {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl VerifyRun {
    fn run(s: &Session, args: &[&str]) -> Self {
        let out = Command::new(&s.bin)
            .args(args)
            .current_dir(s.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("spawn product verify");
        VerifyRun {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        }
    }

    fn ci(s: &Session, extra: &[&str]) -> Self {
        let mut args: Vec<&str> = vec!["verify", "--ci"];
        args.extend(extra);
        Self::run(s, &args)
    }

    fn parse_json(&self) -> serde_json::Value {
        let start = self.stdout.find('{').unwrap_or(0);
        serde_json::from_str(&self.stdout[start..]).unwrap_or_else(|e| {
            panic!(
                "failed to parse CI JSON (exit={}): {}\nstdout: {}\nstderr: {}",
                self.exit_code, e, self.stdout, self.stderr
            )
        })
    }
}

fn stage(v: &serde_json::Value, stage_num: u64) -> &serde_json::Value {
    v.get("stages")
        .and_then(|a| a.as_array())
        .and_then(|a| a.iter().find(|s| s.get("stage").and_then(|x| x.as_u64()) == Some(stage_num)))
        .unwrap_or_else(|| panic!("stage {} not in CI JSON: {}", stage_num, v))
}

fn stage_status(v: &serde_json::Value, stage_num: u64) -> String {
    stage(v, stage_num)
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string()
}

fn stage_findings_codes(v: &serde_json::Value, stage_num: u64) -> Vec<String> {
    stage(v, stage_num)
        .get("findings")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Write a minimal feature file directly to disk (no request pipeline).
fn write_feature(s: &Session, id: &str, title: &str, phase: u32, status: &str, adrs: &[&str], tests: &[&str]) {
    let adrs_str = if adrs.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", adrs.join(", "))
    };
    let tests_str = if tests.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", tests.join(", "))
    };
    let content = format!(
        r#"---
id: {id}
title: {title}
phase: {phase}
status: {status}
adrs: {adrs_str}
tests: {tests_str}
---

Test feature.
"#
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/features/{}-{}.md", id, slug), &content);
}

/// Write a minimal ADR file.
fn write_adr(s: &Session, id: &str, title: &str, status: &str) {
    let content = format!(
        r#"---
id: {id}
title: {title}
status: {status}
domains: [api]
scope: domain
---

**Context:** Test context.

**Decision:** Test decision.

**Rationale:** Test rationale.

**Rejected alternatives:** None.

**Test coverage:** None.
"#
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

/// Write a minimal TC with the given runner (or no runner if empty).
fn write_tc(
    s: &Session,
    id: &str,
    title: &str,
    feature: &str,
    runner: &str,
    runner_args: &str,
    status: &str,
) {
    let mut fm = format!(
        r#"---
id: {id}
title: {title}
type: scenario
status: {status}
validates:
  features: [{feature}]
phase: 1
"#
    );
    if !runner.is_empty() {
        fm.push_str(&format!("runner: {runner}\n"));
        if !runner_args.is_empty() {
            fm.push_str(&format!("runner-args: \"{runner_args}\"\n"));
        }
    }
    fm.push_str("---\n\nTest criterion.\n");
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/tests/{}-{}.md", id, slug), &fm);
}

/// Make a shell script that exits with the given code. Returns its path
/// relative to the session root.
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

// ---------------------------------------------------------------------------
// TC-552 — all stages pass on a clean repo
// ---------------------------------------------------------------------------

#[test]
fn tc_552_verify_all_pass_clean_repo() {
    let s = Session::new();

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(
        run.exit_code, 0,
        "expected exit 0 on clean repo; stdout:\n{}\nstderr:\n{}",
        run.stdout, run.stderr
    );

    let v = run.parse_json();
    assert_eq!(v.get("passed").and_then(|x| x.as_bool()), Some(true));
    assert_eq!(v.get("exit").and_then(|x| x.as_i64()), Some(0));

    // Every stage is pass
    let stages = v.get("stages").and_then(|a| a.as_array()).expect("stages array");
    assert_eq!(stages.len(), 6);
    for (i, st) in stages.iter().enumerate() {
        assert_eq!(
            st.get("status").and_then(|s| s.as_str()),
            Some("pass"),
            "stage {} expected pass, got {:?}",
            i + 1,
            st
        );
    }
}

// ---------------------------------------------------------------------------
// TC-553 — E-class graph error fails the pipeline
// ---------------------------------------------------------------------------

#[test]
fn tc_553_verify_fails_on_e_class_graph_error() {
    let s = Session::new();

    // Feature references a non-existent ADR — E002 broken link.
    write_feature(&s, "FT-001", "Broken Link Feature", 1, "in-progress", &["ADR-999"], &[]);

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(
        run.exit_code, 1,
        "expected exit 1 on E-class error; stdout:\n{}",
        run.stdout
    );
    let v = run.parse_json();
    assert_eq!(v.get("exit").and_then(|x| x.as_i64()), Some(1));
    assert_eq!(v.get("passed").and_then(|x| x.as_bool()), Some(false));

    // Stage 2 fails with E002
    assert_eq!(stage_status(&v, 2), "fail");
    let codes = stage_findings_codes(&v, 2);
    assert!(
        codes.iter().any(|c| c == "E002"),
        "expected E002 in stage 2 findings, got {:?}",
        codes
    );

    // All 6 stages still run — report is complete.
    assert_eq!(
        v.get("stages").and_then(|a| a.as_array()).map(Vec::len),
        Some(6)
    );
}

// ---------------------------------------------------------------------------
// TC-554 — W-class warning only yields exit 2
// ---------------------------------------------------------------------------

#[test]
fn tc_554_verify_warns_on_w_class_only() {
    let s = Session::new();

    // An orphaned ADR (no feature links) triggers W001.
    write_adr(&s, "ADR-001", "Orphan", "accepted");

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(
        run.exit_code, 2,
        "expected exit 2 on W-class only; stdout:\n{}",
        run.stdout
    );
    let v = run.parse_json();
    assert_eq!(v.get("exit").and_then(|x| x.as_i64()), Some(2));

    // Stage 2 is warning
    assert_eq!(stage_status(&v, 2), "warning");
    // All stages present
    assert_eq!(
        v.get("stages").and_then(|a| a.as_array()).map(Vec::len),
        Some(6)
    );
}

// ---------------------------------------------------------------------------
// TC-555 — failing TC fails the pipeline (stage 5)
// ---------------------------------------------------------------------------

#[test]
fn tc_555_verify_fails_on_failing_tc() {
    let s = Session::new();

    // Accepted ADR so the feature can verify without E016.
    write_adr(&s, "ADR-001", "Decision", "accepted");

    // A TC that runs a bash script exiting 1.
    let script = write_exit_script(&s, "fail", 1);
    write_feature(
        &s,
        "FT-001",
        "Failing Feature",
        1,
        "in-progress",
        &["ADR-001"],
        &["TC-001"],
    );
    write_tc(&s, "TC-001", "Failing Test", "FT-001", "bash", &script, "failing");

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(run.exit_code, 1, "expected exit 1; stdout:\n{}", run.stdout);
    let v = run.parse_json();

    assert_eq!(stage_status(&v, 5), "fail");
    let findings = stage(&v, 5).get("findings").and_then(|a| a.as_array()).unwrap();
    let has_failing_tc = findings.iter().any(|f| {
        f.get("tc").and_then(|t| t.as_str()) == Some("TC-001")
            && f.get("feature").and_then(|x| x.as_str()) == Some("FT-001")
            && f.get("status").and_then(|x| x.as_str()) == Some("failing")
    });
    assert!(
        has_failing_tc,
        "expected a {{tc, feature, status: 'failing'}} finding in stage 5, got {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// TC-556 — features in a locked phase are skipped with a reason
// ---------------------------------------------------------------------------

#[test]
fn tc_556_verify_skips_locked_phase_features() {
    let s = Session::new();

    // Accepted ADR so that an in-progress feature linked to it doesn't
    // drag its graph check to proposed-ADR state.
    write_adr(&s, "ADR-001", "Decision", "accepted");

    // Phase 1 feature (in-progress) with a TC. Phase 2 feature is complete
    // → phase 1 is locked per ADR-040.
    write_feature(&s, "FT-001", "Phase One", 1, "in-progress", &["ADR-001"], &["TC-001"]);
    write_tc(&s, "TC-001", "Some Test", "FT-001", "", "", "unimplemented");
    write_feature(&s, "FT-002", "Phase Two", 2, "complete", &["ADR-001"], &[]);

    let run = VerifyRun::ci(&s, &[]);
    let v = run.parse_json();

    let findings = stage(&v, 5)
        .get("findings")
        .and_then(|a| a.as_array())
        .unwrap_or_else(|| panic!("no findings array; stdout:\n{}", run.stdout));

    let phase1_skipped = findings.iter().any(|f| {
        f.get("tc").and_then(|t| t.as_str()) == Some("TC-001")
            && f.get("status").and_then(|s| s.as_str()) == Some("skipped")
            && f.get("reason")
                .and_then(|s| s.as_str())
                .map(|r| r.contains("phase-1-locked"))
                .unwrap_or(false)
    });

    assert!(
        phase1_skipped,
        "expected TC-001 to be skipped with reason containing 'phase-1-locked', got {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// TC-557 — --phase N scopes stage 5 to that phase
// ---------------------------------------------------------------------------

#[test]
fn tc_557_verify_phase_scope_flag() {
    let s = Session::new();

    write_adr(&s, "ADR-001", "Decision", "accepted");
    // FT-001 phase 1 with TC-001, FT-002 phase 2 with TC-002.
    write_feature(&s, "FT-001", "Phase One", 1, "in-progress", &["ADR-001"], &["TC-001"]);
    write_tc(&s, "TC-001", "P1 Test", "FT-001", "", "", "unimplemented");
    write_feature(&s, "FT-002", "Phase Two", 2, "in-progress", &["ADR-001"], &["TC-002"]);
    write_tc(&s, "TC-002", "P2 Test", "FT-002", "", "", "unimplemented");

    let run = VerifyRun::ci(&s, &["--phase", "1"]);
    let v = run.parse_json();

    let findings = stage(&v, 5)
        .get("findings")
        .and_then(|a| a.as_array())
        .unwrap_or_else(|| panic!("no findings array; stdout:\n{}", run.stdout));

    let tcs: Vec<String> = findings
        .iter()
        .filter_map(|f| f.get("tc").and_then(|t| t.as_str()).map(String::from))
        .collect();

    assert!(
        tcs.iter().any(|t| t == "TC-001"),
        "expected TC-001 in phase-1 stage 5 findings, got {:?}",
        tcs
    );
    assert!(
        !tcs.iter().any(|t| t == "TC-002"),
        "TC-002 (phase 2) must not appear in --phase 1 stage 5 findings, got {:?}",
        tcs
    );
}

// ---------------------------------------------------------------------------
// TC-558 — --ci produces valid single-document JSON matching the schema
// ---------------------------------------------------------------------------

#[test]
fn tc_558_verify_ci_json_output() {
    let s = Session::new();

    // A mix of pass/warning/fail: orphan ADR (W), broken link on a feature
    // (E); also complete feature with no TC (W).
    write_adr(&s, "ADR-001", "Orphan One", "accepted");
    write_feature(&s, "FT-001", "Broken", 1, "in-progress", &["ADR-999"], &[]);

    let run = VerifyRun::ci(&s, &[]);
    let v = run.parse_json();

    // Top-level schema
    assert!(v.get("passed").and_then(|b| b.as_bool()).is_some());
    assert!(v.get("exit").and_then(|x| x.as_i64()).is_some());
    let stages = v
        .get("stages")
        .and_then(|a| a.as_array())
        .expect("stages array");
    assert_eq!(stages.len(), 6);

    let names: Vec<&str> = stages
        .iter()
        .filter_map(|s| s.get("name").and_then(|x| x.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![
            "log-integrity",
            "graph-structure",
            "schema-validation",
            "metrics",
            "feature-tcs",
            "platform-tcs"
        ]
    );

    for st in stages {
        assert!(st.get("stage").and_then(|x| x.as_i64()).is_some());
        let status = st.get("status").and_then(|x| x.as_str()).unwrap_or("");
        assert!(
            matches!(status, "pass" | "warning" | "fail"),
            "bad status: {}",
            status
        );
        assert!(st.get("findings").and_then(|a| a.as_array()).is_some());
    }

    // No ANSI colour codes in CI output
    assert!(
        !run.stdout.contains("\x1b["),
        "CI output must contain no ANSI colour codes"
    );
}

// ---------------------------------------------------------------------------
// TC-559 — per-feature behaviour unchanged (positional arg dispatches to ADR-021 path)
// ---------------------------------------------------------------------------

#[test]
fn tc_559_verify_feature_scope_unchanged() {
    let s = Session::new();

    write_adr(&s, "ADR-001", "Decision", "accepted");
    let script = write_exit_script(&s, "pass", 0);
    write_feature(
        &s,
        "FT-001",
        "OneShot",
        1,
        "in-progress",
        &["ADR-001"],
        &["TC-001"],
    );
    write_tc(&s, "TC-001", "OneShot Test", "FT-001", "bash", &script, "unimplemented");

    let run = VerifyRun::run(&s, &["verify", "FT-001"]);
    assert_eq!(
        run.exit_code, 0,
        "per-feature verify with passing TC should exit 0; stdout:\n{}\nstderr:\n{}",
        run.stdout, run.stderr
    );

    // Output is the per-feature format, NOT the pipeline report.
    assert!(
        !run.stdout.contains("log-integrity"),
        "per-feature output must not include pipeline stage names"
    );
    assert!(
        !run.stdout.contains("[1/6]"),
        "per-feature output must not include pipeline stage counters"
    );

    // Feature transitioned to complete.
    let ft_content = s.read("docs/features/FT-001-oneshot.md");
    assert!(
        ft_content.contains("status: complete"),
        "feature should be status: complete after passing per-feature verify;\ncontent:\n{}",
        ft_content
    );
}

// ---------------------------------------------------------------------------
// TC-560 — stage 1 detects a tampered log
// ---------------------------------------------------------------------------

#[test]
fn tc_560_verify_log_integrity_stage_1() {
    let mut s = Session::new();

    // Apply one valid request so the log has at least one entry.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed for log-integrity test"
artifacts:
  - type: feature
    title: Seed
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    let log_path = s.dir.path().join("requests.jsonl");
    assert!(log_path.exists(), "requests.jsonl must exist after apply");

    // Tamper: rewrite the reason inside the last entry so its stored hash
    // no longer matches.
    let raw = std::fs::read_to_string(&log_path).expect("read log");
    let tampered = raw.replacen(
        "seed for log-integrity test",
        "T4MPERED reason string aaaa",
        1,
    );
    assert_ne!(raw, tampered, "tamper substitution must apply");
    std::fs::write(&log_path, tampered).expect("write tampered log");

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(
        run.exit_code, 1,
        "tampered log must exit 1; stdout:\n{}\nstderr:\n{}",
        run.stdout, run.stderr
    );
    let v = run.parse_json();
    assert_eq!(stage_status(&v, 1), "fail");
    let codes = stage_findings_codes(&v, 1);
    let has_e_code = codes
        .iter()
        .any(|c| c == "E015" || c == "E016" || c == "E017" || c == "E018");
    assert!(
        has_e_code,
        "expected E015/E016 (or E017/E018 per ADR-039) in stage 1 findings, got {:?}",
        codes
    );

    // All 6 stages ran.
    assert_eq!(
        v.get("stages").and_then(|a| a.as_array()).map(Vec::len),
        Some(6)
    );
}

// ---------------------------------------------------------------------------
// TC-561 — metrics threshold breach is reported in stage 4
// ---------------------------------------------------------------------------

#[test]
fn tc_561_verify_metrics_threshold_stage_4() {
    let s = Session::new();

    // Write a feature so spec_coverage is computable — a feature with no
    // linked ADRs has spec_coverage 0, which trivially trips a min threshold.
    write_feature(&s, "FT-001", "Uncovered", 1, "in-progress", &[], &[]);

    // Append a warning-severity min-threshold of 0.9 for spec_coverage.
    let cfg_path = s.dir.path().join("product.toml");
    let existing = std::fs::read_to_string(&cfg_path).expect("read config");
    let added = format!(
        "{existing}\n[metrics.thresholds]\nspec_coverage = {{ min = 0.9, severity = \"warning\" }}\n"
    );
    std::fs::write(&cfg_path, added).expect("write config");

    let run = VerifyRun::ci(&s, &[]);
    let v = run.parse_json();

    let st4 = stage_status(&v, 4);
    assert_eq!(
        st4, "warning",
        "stage 4 should warn on breach; stdout:\n{}",
        run.stdout
    );
    let codes = stage_findings_codes(&v, 4);
    assert!(
        codes.iter().any(|c| c.contains("spec_coverage")),
        "expected spec_coverage in stage 4 findings, got {:?}",
        codes
    );

    // Overall exit is at least 2 (could be higher if other stages fail;
    // for this fixture it should be 2).
    assert!(
        v.get("exit").and_then(|x| x.as_i64()) == Some(2)
            || v.get("exit").and_then(|x| x.as_i64()) == Some(1),
        "exit should be 2 (warning) or 1 (error), got {:?}",
        v.get("exit")
    );
}

// ---------------------------------------------------------------------------
// TC-562 — exit-criteria: the pipeline contract holds end-to-end.
// ---------------------------------------------------------------------------

#[test]
fn tc_562_unified_verify_pipeline_exit() {
    // Composite smoke-check: one run on a clean repo must match the
    // invariants of every TC above.
    let s = Session::new();

    let run = VerifyRun::ci(&s, &[]);
    assert_eq!(run.exit_code, 0, "clean repo must exit 0");

    let v = run.parse_json();
    assert_eq!(v.get("passed").and_then(|b| b.as_bool()), Some(true));
    assert_eq!(v.get("exit").and_then(|x| x.as_i64()), Some(0));

    // Stage count invariant.
    let stages = v.get("stages").and_then(|a| a.as_array()).unwrap();
    assert_eq!(stages.len(), 6);

    // Pretty output is available too and includes the stage counters.
    let pretty = VerifyRun::run(&s, &["verify"]);
    assert_eq!(pretty.exit_code, 0);
    assert!(
        pretty.stdout.contains("[1/6]") && pretty.stdout.contains("[6/6]"),
        "pretty output should enumerate all six stages; got:\n{}",
        pretty.stdout
    );
    assert!(
        pretty.stdout.contains("Exit:    0"),
        "pretty output should record exit 0; got:\n{}",
        pretty.stdout
    );
}
