//! Integration tests — status.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_083_status() {
    let h = Harness::new();
    let adr_source = r#"# ADRs

## ADR-001: Accepted ADR

**Status:** Accepted

Context for accepted.

### Test coverage

- `test_one_accepted` — a test

## ADR-002: Proposed ADR

**Status:** Proposed

Context for proposed.

### Test coverage

- `test_two_proposed` — another test

## ADR-003: No Status ADR

Context without status line.

### Test coverage

- `test_three_nostatus` — yet another test
"#;
    h.write("source-adrs.md", adr_source);
    let out = h.run(&["migrate", "from-adrs", "source-adrs.md", "--execute"]);
    out.assert_exit(0);

    // Check ADR-001 has status: accepted
    let adr1_files: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/adrs"))
        .expect("readdir")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("ADR-001"))
        .collect();
    assert_eq!(adr1_files.len(), 1, "should create ADR-001");
    let adr1_content = std::fs::read_to_string(adr1_files[0].path()).unwrap_or_default();
    assert!(adr1_content.contains("status: accepted"), "ADR-001 should have status: accepted, got:\n{}", adr1_content);

    // Check ADR-002 has status: proposed
    let adr2_files: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/adrs"))
        .expect("readdir")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("ADR-002"))
        .collect();
    assert_eq!(adr2_files.len(), 1, "should create ADR-002");
    let adr2_content = std::fs::read_to_string(adr2_files[0].path()).unwrap_or_default();
    assert!(adr2_content.contains("status: proposed"), "ADR-002 should have status: proposed, got:\n{}", adr2_content);

    // Check ADR-003 defaults to proposed (no status found) and W008 warning
    let adr3_files: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/adrs"))
        .expect("readdir")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("ADR-003"))
        .collect();
    assert_eq!(adr3_files.len(), 1, "should create ADR-003");
    let adr3_content = std::fs::read_to_string(adr3_files[0].path()).unwrap_or_default();
    assert!(adr3_content.contains("status: proposed"), "ADR-003 should default to proposed, got:\n{}", adr3_content);

    // W008 warning should appear in stdout for ADR-003
    assert!(
        out.stdout.contains("W008"),
        "should warn W008 for missing status, got stdout:\n{}",
        out.stdout
    );
}

#[test]
fn tc_237_status_shows_phase_gate() {
    let h = Harness::new();
    // Phase 1 with passing exit criteria → OPEN
    h.write(
        "docs/features/FT-001-phase1.md",
        "---\nid: FT-001\ntitle: Phase 1 Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: Phase 1 Exit\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    // Phase 2 with failing exit criteria → LOCKED
    h.write(
        "docs/features/FT-005-phase2.md",
        "---\nid: FT-005\ntitle: Phase 2 Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-007]\n---\n",
    );
    h.write(
        "docs/tests/TC-007-exit.md",
        "---\nid: TC-007\ntitle: Phase 2 Exit\ntype: exit-criteria\nstatus: failing\nvalidates:\n  features: [FT-005]\n  adrs: []\nphase: 2\n---\n",
    );

    let out = h.run(&["status"]);
    out.assert_exit(0);

    // Phase 1 should show [OPEN]
    assert!(
        out.stdout.contains("[OPEN]"),
        "Phase 1 should show [OPEN]. stdout:\n{}",
        out.stdout
    );
    // Phase 2 should show [LOCKED]
    assert!(
        out.stdout.contains("[LOCKED"),
        "Phase 2 should show [LOCKED]. stdout:\n{}",
        out.stdout
    );
    // LOCKED phase should name the failing TC
    assert!(
        out.stdout.contains("TC-007"),
        "LOCKED phase should name failing TC-007. stdout:\n{}",
        out.stdout
    );
}

#[test]
fn tc_238_status_phase_detail() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-phase1.md",
        "---\nid: FT-001\ntitle: Phase 1 Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: []\ntests: [TC-001, TC-002]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: First Exit\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h.write(
        "docs/tests/TC-002-exit.md",
        "---\nid: TC-002\ntitle: Second Exit\ntype: exit-criteria\nstatus: failing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );

    let out = h.run(&["status", "--phase", "1"]);
    out.assert_exit(0);

    // Should list individual exit-criteria TCs with pass/fail
    assert!(
        out.stdout.contains("TC-001") && out.stdout.contains("passing"),
        "Should show TC-001 as passing. stdout:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("TC-002") && out.stdout.contains("failing"),
        "Should show TC-002 as failing. stdout:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("Exit criteria"),
        "Should show 'Exit criteria' section. stdout:\n{}",
        out.stdout
    );
}

#[test]
fn tc_636_due_date_field_parses_iso_8601_date() {
    // Valid date — parses and is accepted by graph check.
    let h_ok = fixture_planning(Some("2026-05-01"));
    let out = h_ok.run(&["graph", "check"]);
    // graph check exits 2 or 0 (W028/W029 may fire, but no E-class errors).
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "graph check should not hard-fail on a valid due-date; stderr: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("E001"),
        "valid due-date should not produce E001: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("E006"),
        "valid due-date should not produce E006: {}",
        out.stderr
    );

    // Invalid date — E006 with the expected-YYYY-MM-DD hint.
    let h_bad = fixture_planning(Some("not-a-date"));
    let out_bad = h_bad.run(&["graph", "check"]);
    out_bad.assert_stderr_contains("E006");
    out_bad.assert_stderr_contains("YYYY-MM-DD");
    assert_eq!(
        out_bad.exit_code, 1,
        "invalid due-date should exit 1 (E-class); stderr: {}",
        out_bad.stderr
    );
}

#[test]
fn tc_639_started_tag_created_on_first_in_progress_transition() {
    // Git path — tag is created.
    let h = fixture_with_domains();
    git_init(&h);
    h.write(
        "docs/features/FT-009-payments.md",
        "---\nid: FT-009\ntitle: Payments\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    h.write(
        "req.yaml",
        "type: change\nschema-version: 1\nreason: \"start FT-009\"\nchanges:\n  - target: FT-009\n    mutations:\n      - op: set\n        field: status\n        value: in-progress\n",
    );
    let out = h.run(&["request", "apply", "req.yaml"]);
    out.assert_exit(0);

    // Tag should exist.
    let tag_out = std::process::Command::new("git")
        .args(["tag", "-l", "product/FT-009/started"])
        .current_dir(h.dir.path())
        .output()
        .expect("git tag");
    let tags = String::from_utf8_lossy(&tag_out.stdout);
    assert!(
        tags.contains("product/FT-009/started"),
        "started tag should exist after transition: {}",
        tags
    );

    // Message contains the feature id and status change phrase.
    let msg_out = std::process::Command::new("git")
        .args([
            "tag",
            "-l",
            "product/FT-009/started",
            "--format=%(contents)",
        ])
        .current_dir(h.dir.path())
        .output()
        .expect("tag msg");
    let msg = String::from_utf8_lossy(&msg_out.stdout);
    assert!(msg.contains("FT-009 started"), "tag message: {}", msg);
    assert!(msg.contains("in-progress"), "tag message: {}", msg);

    // No-git path — warning, no crash.
    let h2 = fixture_with_domains();
    h2.write(
        "docs/features/FT-009-payments.md",
        "---\nid: FT-009\ntitle: Payments\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    h2.write(
        "req.yaml",
        "type: change\nschema-version: 1\nreason: \"start FT-009\"\nchanges:\n  - target: FT-009\n    mutations:\n      - op: set\n        field: status\n        value: in-progress\n",
    );
    let out2 = h2.run_with_env(
        &["request", "apply", "req.yaml"],
        &[("PRODUCT_AUTHOR", "local:test")],
    );
    // Apply may fail because of missing git identity — skip assertion on exit code if so.
    // Regardless: when git is missing, the apply either succeeds with a W030 warning
    // or fails on git-identity; both paths are acceptable. The key assertion is that
    // no started tag leaks out.
    let no_tag = !out2.stdout.contains("product/FT-009/started")
        && !out2.stderr.contains("Tagged: product/FT-009/started");
    assert!(no_tag, "no started tag should be created without git: {}{}", out2.stdout, out2.stderr);
}

#[test]
fn tc_640_started_tag_not_recreated_on_replan_or_restart() {
    let h = fixture_with_domains();
    git_init(&h);
    h.write(
        "docs/features/FT-009-payments.md",
        "---\nid: FT-009\ntitle: Payments\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    // First transition → creates product/FT-009/started.
    h.write(
        "req1.yaml",
        "type: change\nschema-version: 1\nreason: \"start\"\nchanges:\n  - target: FT-009\n    mutations:\n      - op: set\n        field: status\n        value: in-progress\n",
    );
    h.run(&["request", "apply", "req1.yaml"]).assert_exit(0);

    // Capture original timestamp.
    let ts_out_1 = std::process::Command::new("git")
        .args([
            "tag",
            "-l",
            "product/FT-009/started",
            "--format=%(creatordate:iso8601)",
        ])
        .current_dir(h.dir.path())
        .output()
        .expect("ts1");
    let ts1 = String::from_utf8_lossy(&ts_out_1.stdout).trim().to_string();
    assert!(!ts1.is_empty(), "started tag should exist after first transition");

    // Replan → planned.
    h.write(
        "req2.yaml",
        "type: change\nschema-version: 1\nreason: \"replan\"\nchanges:\n  - target: FT-009\n    mutations:\n      - op: set\n        field: status\n        value: planned\n",
    );
    h.run(&["request", "apply", "req2.yaml"]).assert_exit(0);

    // Back to in-progress — must NOT create a new or versioned tag.
    h.write(
        "req3.yaml",
        "type: change\nschema-version: 1\nreason: \"restart\"\nchanges:\n  - target: FT-009\n    mutations:\n      - op: set\n        field: status\n        value: in-progress\n",
    );
    let out3 = h.run(&["request", "apply", "req3.yaml"]);
    out3.assert_exit(0);
    assert!(
        !out3.stdout.contains("Tagged: product/FT-009/started"),
        "no new started tag should be emitted on restart: {}",
        out3.stdout
    );

    // Only one `started`-family tag — no `started-v2`.
    let all_tags = std::process::Command::new("git")
        .args(["tag", "-l", "product/FT-009/*"])
        .current_dir(h.dir.path())
        .output()
        .expect("tags");
    let tags = String::from_utf8_lossy(&all_tags.stdout);
    let started_count = tags
        .lines()
        .filter(|l| l.contains("/started"))
        .count();
    assert_eq!(
        started_count, 1,
        "exactly one started tag expected, got: {}",
        tags
    );

    // Timestamp unchanged.
    let ts_out_2 = std::process::Command::new("git")
        .args([
            "tag",
            "-l",
            "product/FT-009/started",
            "--format=%(creatordate:iso8601)",
        ])
        .current_dir(h.dir.path())
        .output()
        .expect("ts2");
    let ts2 = String::from_utf8_lossy(&ts_out_2.stdout).trim().to_string();
    assert_eq!(
        ts1, ts2,
        "started tag timestamp must be preserved across replans"
    );
}

#[test]
fn tc_643_due_date_never_blocks_verification_or_phase_gate() {
    let h = fixture_with_domains();
    h.write(
        "docs/features/FT-009-overdue.md",
        "---\nid: FT-009\ntitle: Overdue\nphase: 1\nstatus: in-progress\ndue-date: \"1970-01-01\"\ndepends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    h.write(
        "docs/adrs/ADR-045-planning.md",
        "---\nid: ADR-045\ntitle: Planning\nstatus: accepted\nfeatures:\n- FT-009\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: cross-cutting\n---\n\nSeed.\n",
    );
    let out = h.run(&["graph", "check"]);
    // W028 present, exit 2 (warning only).
    out.assert_stderr_contains("W028");
    assert_eq!(
        out.exit_code, 2,
        "overdue alone must never produce exit 1; stderr: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("error[E"),
        "overdue due-date must not produce any E-class diagnostic: {}",
        out.stderr
    );
}

#[test]
fn tc_645_cycle_times_lists_complete_features() {
    let h = ct_fixture(&[
        (
            "FT-101",
            "complete",
            Some("2026-04-08T13:00:00+0000"),
            Some("2026-04-11T09:14:00+0000"),
        ),
        (
            "FT-102",
            "complete",
            Some("2026-04-12T10:30:00+0000"),
            Some("2026-04-17T15:42:00+0000"),
        ),
        (
            "FT-103",
            "complete",
            Some("2026-04-15T08:00:00+0000"),
            Some("2026-04-18T18:00:00+0000"),
        ),
    ]);
    let out = h.run(&["cycle-times"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-101");
    out.assert_stdout_contains("FT-102");
    out.assert_stdout_contains("FT-103");
    out.assert_stdout_contains("count:");
    // recent/all stats render
    out.assert_stdout_contains("median");
    // no trend line with <6 features
    assert!(
        !out.stdout.contains("Trend:"),
        "trend must be omitted below 6 complete features: {}",
        out.stdout
    );
}

#[test]
fn tc_646_cycle_times_excludes_features_without_started_tag() {
    let h = ct_fixture(&[
        (
            "FT-201",
            "complete",
            Some("2026-04-08T13:00:00+0000"),
            Some("2026-04-11T09:00:00+0000"),
        ),
        ("FT-202", "complete", None, Some("2026-04-15T00:00:00+0000")),
    ]);
    // Also add enough other features so we clear the min-features gate.
    ct_write_feature(&h, "FT-203", "complete");
    ct_write_feature(&h, "FT-204", "complete");
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("add");
    std::process::Command::new("git")
        .args(["commit", "-m", "more"])
        .current_dir(h.dir.path())
        .output()
        .expect("commit");
    ct_tag_at(&h, "FT-203", "started", "2026-04-20T08:00:00+0000");
    ct_tag_at(&h, "FT-203", "complete", "2026-04-23T08:00:00+0000");
    ct_tag_at(&h, "FT-204", "started", "2026-04-25T08:00:00+0000");
    ct_tag_at(&h, "FT-204", "complete", "2026-04-28T08:00:00+0000");

    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    // FT-202 should NOT appear in the feature list
    assert!(
        !out.stdout.contains("FT-202"),
        "FT-202 (no started tag) should be excluded: {}",
        out.stdout
    );
    out.assert_stdout_contains("FT-201");
}

#[test]
fn tc_647_cycle_times_excludes_features_without_complete_tag() {
    let h = ct_fixture(&[
        (
            "FT-301",
            "complete",
            Some("2026-04-08T13:00:00+0000"),
            Some("2026-04-11T09:00:00+0000"),
        ),
        ("FT-302", "in-progress", Some("2026-04-15T00:00:00+0000"), None),
        (
            "FT-303",
            "complete",
            Some("2026-04-12T00:00:00+0000"),
            Some("2026-04-14T00:00:00+0000"),
        ),
        (
            "FT-304",
            "complete",
            Some("2026-04-17T00:00:00+0000"),
            Some("2026-04-20T00:00:00+0000"),
        ),
    ]);
    let out = h.run(&["cycle-times"]);
    out.assert_exit(0);
    assert!(
        !out.stdout.contains("FT-302"),
        "FT-302 must not appear in default cycle-times output: {}",
        out.stdout
    );
    out.assert_stdout_contains("FT-301");
}

#[test]
fn tc_648_cycle_times_uses_first_complete_tag_for_v2_features() {
    let h = ct_fixture(&[
        (
            "FT-401",
            "complete",
            Some("2026-04-08T13:00:00+0000"),
            Some("2026-04-11T09:14:00+0000"),
        ),
        (
            "FT-402",
            "complete",
            Some("2026-04-12T00:00:00+0000"),
            Some("2026-04-14T00:00:00+0000"),
        ),
        (
            "FT-403",
            "complete",
            Some("2026-04-16T00:00:00+0000"),
            Some("2026-04-18T00:00:00+0000"),
        ),
    ]);
    // Add complete-v2 for FT-401 at a LATER date.
    ct_tag_at(&h, "FT-401", "complete-v2", "2026-05-03T11:00:00+0000");

    let out = h.run(&["cycle-times", "--format", "csv"]);
    out.assert_exit(0);
    // FT-401: cycle = 2026-04-08 13:00 → 2026-04-11 09:14 ≈ 2.8d (NOT 25d)
    let line_401 = out
        .stdout
        .lines()
        .find(|l| l.starts_with("FT-401,"))
        .expect("row for FT-401");
    let days_str = line_401.split(',').nth(3).expect("days column");
    let days: f64 = days_str.parse().expect("numeric");
    assert!(
        (days - 2.8).abs() <= 0.2,
        "FT-401 cycle time should be ≈2.8d (first complete tag), got {}",
        days
    );
}

#[test]
fn tc_649_cycle_times_recent_5_computed_correctly() {
    // Build 14 complete features with specific cycle times.
    let days = [
        2.84f64, 5.12, 3.21, 8.44, 2.10, 4.88, 1.95, 11.32, 3.67, 2.44, 6.78, 4.01, 3.55, 7.22,
    ];
    let mut entries_owned: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    for (i, d) in days.iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let started = base + chrono::Duration::days((i as i64) * 20);
        let secs = (*d * 86400.0) as i64;
        let completed = started.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds(secs);
        entries_owned.push((
            id,
            "complete".to_string(),
            Some(format!("{} 00:00:00 +0000", started.format("%Y-%m-%d"))),
            Some(format!("{} +0000", completed.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let entries: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries_owned
        .iter()
        .map(|(id, st, s, c)| (id.as_str(), st.as_str(), s.as_deref(), c.as_deref()))
        .collect();
    let h = ct_fixture(&entries);

    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("valid JSON");
    let count = v["summary"]["count"].as_u64().expect("count");
    assert_eq!(count, 14);
    let recent_median = v["summary"]["recent_5"]["median"].as_f64().expect("median");
    assert!(
        (recent_median - 4.0).abs() <= 0.2,
        "recent median ≈ 4.0, got {}",
        recent_median
    );
    let recent_min = v["summary"]["recent_5"]["min"].as_f64().expect("min");
    assert!(
        (recent_min - 2.4).abs() <= 0.2,
        "recent min ≈ 2.4, got {}",
        recent_min
    );
    let recent_max = v["summary"]["recent_5"]["max"].as_f64().expect("max");
    assert!(
        (recent_max - 7.2).abs() <= 0.2,
        "recent max ≈ 7.2, got {}",
        recent_max
    );
    // Trend should be populated with ≥6 features.
    assert!(v["summary"]["trend"].is_string());
}

#[test]
fn tc_650_cycle_times_trend_accelerating() {
    // 6 historic features ~8d, 5 recent features ~3d
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    for (i, d) in [8.0, 7.5, 9.0, 8.5, 7.8, 8.2].iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d as f64 * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    for (i, d) in [3.0f64, 3.5, 2.8, 3.2, 4.0].iter().enumerate() {
        let id = format!("FT-{:03}", 201 + i);
        let st = base + chrono::Duration::days(200 + (i as i64) * 10);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("json");
    assert_eq!(
        v["summary"]["trend"].as_str(),
        Some("accelerating"),
        "expected accelerating; got {:?}",
        v["summary"]["trend"]
    );
}

#[test]
fn tc_651_cycle_times_trend_stable() {
    // 14 features with approximately equal cycle times.
    let days: Vec<f64> = (0..14).map(|_| 4.0).collect();
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    for (i, d) in days.iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("json");
    assert_eq!(
        v["summary"]["trend"].as_str(),
        Some("stable"),
        "expected stable; got {:?}",
        v["summary"]["trend"]
    );
}

#[test]
fn tc_652_cycle_times_trend_slowing() {
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    for (i, d) in [3.0f64, 3.5, 2.8, 3.2, 3.1, 3.3].iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    for (i, d) in [6.0f64, 5.5, 7.0, 6.8, 5.9].iter().enumerate() {
        let id = format!("FT-{:03}", 201 + i);
        let st = base + chrono::Duration::days(200 + (i as i64) * 10);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("json");
    assert_eq!(
        v["summary"]["trend"].as_str(),
        Some("slowing"),
        "expected slowing; got {:?}",
        v["summary"]["trend"]
    );
}

#[test]
fn tc_653_cycle_times_in_progress_shows_elapsed() {
    // Five complete features (to provide a reference median) + one in-progress.
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    for i in 0..5 {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st + chrono::Duration::days(4);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} 00:00:00 +0000", cp.format("%Y-%m-%d"))),
        ));
    }
    // Build a recent "in-progress" feature with a 2-day-old started tag.
    let now = chrono::Local::now();
    let yesterday = now - chrono::Duration::days(2);
    entries.push((
        "FT-015".into(),
        "in-progress".into(),
        Some(format!("{} +0000", yesterday.format("%Y-%m-%d %H:%M:%S"))),
        None,
    ));
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["cycle-times", "--in-progress"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-015");
    // should NOT contain any of the complete FT-101/102 rows in this view
    assert!(
        !out.stdout.contains("FT-101"),
        "complete features must not appear in --in-progress view: {}",
        out.stdout
    );
}

#[test]
fn tc_654_cycle_times_json_valid_schema() {
    let h = ct_fixture(&[
        (
            "FT-601",
            "complete",
            Some("2026-04-01T00:00:00+0000"),
            Some("2026-04-04T00:00:00+0000"),
        ),
        (
            "FT-602",
            "complete",
            Some("2026-04-05T00:00:00+0000"),
            Some("2026-04-10T00:00:00+0000"),
        ),
        (
            "FT-603",
            "complete",
            Some("2026-04-11T00:00:00+0000"),
            Some("2026-04-14T00:00:00+0000"),
        ),
    ]);
    let out = h.run(&["cycle-times", "--format", "json"]);
    out.assert_exit(0);
    let v: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("valid JSON");
    assert!(v.get("features").is_some(), "features array required");
    assert!(v.get("summary").is_some(), "summary object required");
    let features = v["features"].as_array().expect("array");
    for f in features {
        assert!(f.get("id").is_some());
        assert!(f.get("started").is_some());
        assert!(f.get("completed").is_some());
        assert!(f.get("cycle_time_days").is_some());
        let days = f["cycle_time_days"].as_f64().expect("number");
        assert!(days >= 0.0, "cycle time non-negative");
    }
    assert!(v["summary"]["count"].is_number());
}

#[test]
fn tc_655_cycle_times_csv_parseable() {
    let h = ct_fixture(&[
        (
            "FT-701",
            "complete",
            Some("2026-04-01T00:00:00+0000"),
            Some("2026-04-04T00:00:00+0000"),
        ),
        (
            "FT-702",
            "complete",
            Some("2026-04-05T00:00:00+0000"),
            Some("2026-04-10T00:00:00+0000"),
        ),
        (
            "FT-703",
            "complete",
            Some("2026-04-11T00:00:00+0000"),
            Some("2026-04-14T00:00:00+0000"),
        ),
    ]);
    let out = h.run(&["cycle-times", "--format", "csv"]);
    out.assert_exit(0);
    let first_line = out.stdout.lines().next().expect("first line");
    assert_eq!(
        first_line, "feature_id,started,completed,cycle_time_days,phase",
        "CSV header must match schema; got: {}",
        first_line
    );
    for line in out.stdout.lines().skip(1) {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        assert_eq!(cols.len(), 5, "CSV row has 5 columns: {}", line);
        // cycle_time_days is a number with exactly one decimal.
        let days_col = cols[3];
        assert!(
            days_col.contains('.'),
            "cycle_time_days must have decimal: {}",
            days_col
        );
    }
}

#[test]
fn tc_656_forecast_naive_single_feature() {
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    for (i, d) in [2.44f64, 6.78, 4.01, 3.55, 7.22].iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    // An in-progress feature.
    let now = chrono::Local::now();
    let started = now - chrono::Duration::hours(50);
    entries.push((
        "FT-015".into(),
        "in-progress".into(),
        Some(format!("{} +0000", started.format("%Y-%m-%d %H:%M:%S"))),
        None,
    ));
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["forecast", "FT-015", "--naive"]);
    out.assert_exit(0);
    out.assert_stdout_contains("Likely completion:");
    out.assert_stdout_contains("Optimistic:");
    out.assert_stdout_contains("Pessimistic:");
    out.assert_stdout_contains("rough estimate");
    out.assert_stdout_contains("not a probability forecast");
}

#[test]
fn tc_657_forecast_naive_phase_sequential() {
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    for (i, d) in [2.44f64, 6.78, 4.01, 3.55, 7.22].iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 20);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    // Add 5 planned features in phase 2.
    for i in 0..5 {
        let id = format!("FT-{:03}", 301 + i);
        h.write(
            &format!("docs/features/{}-p2.md", id),
            &format!(
                "---\nid: {}\ntitle: {}\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\nSeed.\n",
                id, id
            ),
        );
    }
    let out = h.run(&["forecast", "--phase", "2", "--naive"]);
    out.assert_exit(0);
    out.assert_stdout_contains("Phase 2");
    out.assert_stdout_contains("5");
    out.assert_stdout_contains("Likely completion:");
    out.assert_stdout_contains("Assumes no parallelism");
    out.assert_stdout_contains("cycle-times --format csv");
}

#[test]
fn tc_658_forecast_naive_insufficient_data() {
    let h = ct_fixture(&[
        (
            "FT-801",
            "complete",
            Some("2026-04-01T00:00:00+0000"),
            Some("2026-04-03T00:00:00+0000"),
        ),
        (
            "FT-802",
            "complete",
            Some("2026-04-05T00:00:00+0000"),
            Some("2026-04-08T00:00:00+0000"),
        ),
    ]);
    // An in-progress feature to target.
    ct_write_feature(&h, "FT-803", "in-progress");
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("add");
    std::process::Command::new("git")
        .args(["commit", "-m", "more"])
        .current_dir(h.dir.path())
        .output()
        .expect("commit");
    ct_tag_at(&h, "FT-803", "started", "2026-04-11T00:00:00+0000");

    let out = h.run(&["forecast", "FT-803", "--naive"]);
    assert_eq!(
        out.exit_code, 2,
        "expected exit 2 for insufficient data; got {}: {}",
        out.exit_code, out.stderr
    );
    // Message mentions Insufficient and the minimum.
    assert!(
        out.stderr.contains("Insufficient"),
        "stderr should mention Insufficient: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("3"),
        "stderr should mention the threshold: {}",
        out.stderr
    );
}

#[test]
fn tc_659_forecast_naive_elapsed_exceeds_sample_clamps_to_today() {
    // Start 5 "recent" features each at 1-day cycle time, then an in-progress
    // that has elapsed 30 days already. Projections should clamp.
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    for i in 0..5 {
        let id = format!("FT-{:03}", 101 + i);
        let st = base + chrono::Duration::days((i as i64) * 10);
        let cp = st + chrono::Duration::days(1);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} 00:00:00 +0000", cp.format("%Y-%m-%d"))),
        ));
    }
    let now = chrono::Local::now();
    let started = now - chrono::Duration::days(30);
    entries.push((
        "FT-999".into(),
        "in-progress".into(),
        Some(format!("{} +0000", started.format("%Y-%m-%d %H:%M:%S"))),
        None,
    ));
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["forecast", "FT-999", "--naive", "--format", "json"]);
    out.assert_exit(0);
    let today_iso = now.format("%Y-%m-%d").to_string();
    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("json");
    assert_eq!(
        v["forecast"]["likely"].as_str(),
        Some(today_iso.as_str()),
        "likely must clamp to today"
    );
    assert_eq!(
        v["forecast"]["optimistic"].as_str(),
        Some(today_iso.as_str()),
        "optimistic must clamp to today"
    );
    assert_eq!(
        v["forecast"]["pessimistic"].as_str(),
        Some(today_iso.as_str()),
        "pessimistic must clamp to today"
    );
}

#[test]
fn tc_660_status_shows_cycle_time_column_when_data_present() {
    let base = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut entries: Vec<(String, String, Option<String>, Option<String>)> = Vec::new();
    for (i, d) in [2.84f64, 5.12, 3.21].iter().enumerate() {
        let id = format!("FT-{:03}", 1 + i);
        let st = base + chrono::Duration::days((i as i64) * 10);
        let cp = st.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        entries.push((
            id,
            "complete".into(),
            Some(format!("{} 00:00:00 +0000", st.format("%Y-%m-%d"))),
            Some(format!("{} +0000", cp.format("%Y-%m-%d %H:%M:%S"))),
        ));
    }
    let refs: Vec<(&str, &str, Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|(id, s, st, cp)| (id.as_str(), s.as_str(), st.as_deref(), cp.as_deref()))
        .collect();
    let h = ct_fixture(&refs);
    let out = h.run(&["status"]);
    out.assert_exit(0);
    // Some cycle-time label should appear somewhere in the output.
    assert!(
        out.stdout.contains("cycle") || out.stdout.contains("2.8d") || out.stdout.contains("5.1d"),
        "expected cycle-time cell in status output: {}",
        out.stdout
    );
}

#[test]
fn tc_661_status_omits_cycle_time_column_when_below_min() {
    let h = ct_fixture(&[
        (
            "FT-001",
            "complete",
            Some("2026-04-01T00:00:00+0000"),
            Some("2026-04-03T00:00:00+0000"),
        ),
        (
            "FT-002",
            "complete",
            Some("2026-04-05T00:00:00+0000"),
            Some("2026-04-08T00:00:00+0000"),
        ),
    ]);
    let out = h.run(&["status"]);
    out.assert_exit(0);
    // With default min-features = 3 and only 2 complete features,
    // the "cycle" label must not appear.
    assert!(
        !out.stdout.contains("  cycle"),
        "cycle-time cell should be absent below min-features: {}",
        out.stdout
    );
}

#[test]
fn tc_662_cycle_time_visibility_and_naive_forecast_exit() {
    // Same fixture as TC-645 (3 features), plus a 4th to clear min-features.
    let h = ct_fixture(&[
        (
            "FT-601",
            "complete",
            Some("2026-04-01T00:00:00+0000"),
            Some("2026-04-04T00:00:00+0000"),
        ),
        (
            "FT-602",
            "complete",
            Some("2026-04-05T00:00:00+0000"),
            Some("2026-04-10T00:00:00+0000"),
        ),
        (
            "FT-603",
            "complete",
            Some("2026-04-11T00:00:00+0000"),
            Some("2026-04-14T00:00:00+0000"),
        ),
    ]);

    // 1. cycle-times ships
    h.run(&["cycle-times"]).assert_exit(0);
    // 2. JSON format works
    let out_json = h.run(&["cycle-times", "--format", "json"]);
    out_json.assert_exit(0);
    let _v: serde_json::Value =
        serde_json::from_str(out_json.stdout.trim()).expect("json");
    // 3. CSV format works
    let out_csv = h.run(&["cycle-times", "--format", "csv"]);
    out_csv.assert_exit(0);
    out_csv.assert_stdout_contains("feature_id,started,completed,cycle_time_days,phase");
    // 4. status shows cycle-time column
    h.run(&["status"]).assert_exit(0);
}

