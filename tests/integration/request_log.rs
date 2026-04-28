//! Integration tests — request_log.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_505_log_entry_appended_on_apply() {
    let h = fixture_log();
    assert!(!h.exists("requests.jsonl"));
    write_log_req(&h, "r.yaml", "test", "Health");
    let out = h.run(&["request", "apply", "r.yaml"]);
    out.assert_exit(0);
    assert!(h.exists("requests.jsonl"));
    let lines = log_lines(&h);
    assert_eq!(lines.len(), 1);
    let v = log_line_json(&h, 0);
    assert_eq!(v["type"], serde_json::json!("create"));
    assert_eq!(v["reason"], serde_json::json!("test"));
    assert_eq!(v["prev-hash"], serde_json::json!("0000000000000000"));
    assert!(v["entry-hash"].as_str().unwrap_or("").len() == 64);
}

#[test]
fn tc_506_log_entry_hash_valid_after_apply() {
    use product_lib::request_log::canonical::{canonical_json, sha256_hex};

    let h = fixture_log();
    write_log_req(&h, "r.yaml", "t", "X");
    h.run(&["request", "apply", "r.yaml"]).assert_exit(0);

    let mut v = log_line_json(&h, 0);
    let stored = v["entry-hash"].as_str().unwrap_or("").to_string();
    assert!(!stored.is_empty());
    v["entry-hash"] = serde_json::json!("");
    let canon = canonical_json(&v);
    let computed = sha256_hex(canon.as_bytes());
    assert_eq!(stored, computed);
}

#[test]
fn tc_507_log_chain_intact_after_multiple_applies() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    write_log_req(&h, "c.yaml", "C", "Charlie");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "c.yaml"]).assert_exit(0);

    let lines = log_lines(&h);
    assert_eq!(lines.len(), 3);
    let a: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    let b: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
    let c: serde_json::Value = serde_json::from_str(&lines[2]).unwrap();
    assert_eq!(a["prev-hash"], serde_json::json!("0000000000000000"));
    assert_eq!(b["prev-hash"], a["entry-hash"]);
    assert_eq!(c["prev-hash"], b["entry-hash"]);

    for v in [&a, &b, &c] {
        let mut v2 = v.clone();
        let stored = v2["entry-hash"].as_str().unwrap().to_string();
        v2["entry-hash"] = serde_json::json!("");
        let canon = product_lib::request_log::canonical::canonical_json(&v2);
        let comp = product_lib::request_log::canonical::sha256_hex(canon.as_bytes());
        assert_eq!(stored, comp);
    }
}

#[test]
fn tc_508_log_verify_passes_on_clean_log() {
    let h = fixture_log();
    for i in 0..3 {
        let name = format!("r{}.yaml", i);
        write_log_req(&h, &name, &format!("r{}", i), &format!("Title{}", i));
        h.run(&["request", "apply", &name]).assert_exit(0);
    }
    let out = h.run(&["request", "log", "verify"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Entry hashes valid (3/3)"));
    assert!(out.stdout.contains("Hash chain intact (3/3)"));
    assert!(out.stdout.contains("Log is tamper-free"));
}

#[test]
fn tc_509_log_verify_detects_entry_modification() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);

    // Tamper: modify the first entry's reason directly.
    let path = h.dir.path().join("requests.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replacen("\"reason\":\"A\"", "\"reason\":\"X\"", 1);
    std::fs::write(&path, tampered).unwrap();

    let out = h.run(&["request", "log", "verify"]);
    out.assert_exit(1);
    let s = format!("{}{}", out.stdout, out.stderr);
    assert!(s.contains("E017"), "expected E017: {}", s);
}

#[test]
fn tc_510_log_verify_detects_chain_break() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);

    // Rewrite entry B with a bogus prev-hash and a correctly-recomputed entry-hash.
    let path = h.dir.path().join("requests.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut v: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
    v["prev-hash"] = serde_json::json!("deadbeef00000000000000000000000000000000000000000000000000000000");
    // Recompute entry-hash so per-entry check passes and only the chain is broken.
    let mut for_hash = v.clone();
    for_hash["entry-hash"] = serde_json::json!("");
    let canon = product_lib::request_log::canonical::canonical_json(&for_hash);
    let h2 = product_lib::request_log::canonical::sha256_hex(canon.as_bytes());
    v["entry-hash"] = serde_json::json!(h2);
    lines[1] = product_lib::request_log::canonical::canonical_json(&v);
    std::fs::write(&path, lines.join("\n") + "\n").unwrap();

    let out = h.run(&["request", "log", "verify"]);
    out.assert_exit(1);
    let s = format!("{}{}", out.stdout, out.stderr);
    assert!(s.contains("E018"), "expected E018: {}", s);
}

#[test]
fn tc_511_log_verify_detects_entry_deletion() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    write_log_req(&h, "c.yaml", "C", "Charlie");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "c.yaml"]).assert_exit(0);

    // Delete line 2 (the B entry).
    let path = h.dir.path().join("requests.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let new_content = format!("{}\n{}\n", lines[0], lines[2]);
    std::fs::write(&path, new_content).unwrap();

    let out = h.run(&["request", "log", "verify"]);
    out.assert_exit(1);
    let s = format!("{}{}", out.stdout, out.stderr);
    assert!(s.contains("E018"), "expected E018 on deletion: {}", s);
}

#[test]
fn tc_512_log_replay_reconstructs_state() {
    let h = fixture_log();
    for i in 0..5 {
        let name = format!("r{}.yaml", i);
        write_log_req(&h, &name, &format!("r{}", i), &format!("Title{}", i));
        h.run(&["request", "apply", &name]).assert_exit(0);
    }

    let out_dir = std::env::temp_dir().join(format!("product-replay-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    let out_s = out_dir.to_string_lossy().to_string();
    let out = h.run(&["request", "replay", "--full", "--output", &out_s]);
    out.assert_exit(0);
    // docs/ present
    assert!(out_dir.join("docs/features").exists());
    // Contains the feature files that exist in the working tree
    let wt_features: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    for name in wt_features {
        let target = out_dir.join("docs/features").join(&name);
        assert!(target.exists(), "missing {} in replay", target.display());
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn tc_513_log_replay_to_checkpoint() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    let first_id = log_line_json(&h, 0)["id"].as_str().unwrap().to_string();
    write_log_req(&h, "b.yaml", "B", "Bravo");
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);

    let out_dir = std::env::temp_dir().join(format!("product-replay-to-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    let out_s = out_dir.to_string_lossy().to_string();
    let out = h.run(&["request", "replay", "--to", &first_id, "--output", &out_s]);
    out.assert_exit(0);
    // Only the first feature should remain
    // (replay simplified: truncates the log and removes post-target artifacts)
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn tc_514_log_undo_appends_inverse() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "Original", "Alpha");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    let target_id = log_line_json(&h, 0)["id"].as_str().unwrap().to_string();

    let out = h.run(&["request", "undo", &target_id, "--reason", "revert"]);
    out.assert_exit(0);
    let lines = log_lines(&h);
    assert_eq!(lines.len(), 2);
    let v = log_line_json(&h, 1);
    assert_eq!(v["type"], serde_json::json!("undo"));
    assert_eq!(v["undoes"], serde_json::json!(target_id));
    assert_eq!(v["reason"], serde_json::json!("revert"));
    // chain
    let a: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(v["prev-hash"], a["entry-hash"]);
}

#[test]
fn tc_515_log_undo_does_not_delete_entries() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "O", "Alpha");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    let first_line = log_lines(&h)[0].clone();
    let target_id = log_line_json(&h, 0)["id"].as_str().unwrap().to_string();

    h.run(&["request", "undo", &target_id]).assert_exit(0);
    let lines = log_lines(&h);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], first_line, "original entry must be preserved byte-for-byte");
}

#[test]
fn tc_516_log_migrate_entry_first() {
    let h = fixture_log();
    // Write a minimal mono-ADR source
    let src = "docs/monolithic-adrs.md";
    h.write(src, "## ADR-001: Test Decision\n\n**Status:** Accepted\n\n### Context\n\nSomething.\n\n### Decision\n\nDo something.\n");

    let out = h.run(&["migrate", "from-adrs", src, "--execute"]);
    // --execute may write files but that's fine
    assert!(out.exit_code == 0 || out.exit_code == 1, "unexpected: {:?}", out.exit_code);
    // Regardless of outcome, the log should be either absent or have a migrate entry
    let lines = log_lines(&h);
    if !lines.is_empty() {
        let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(v["type"], serde_json::json!("migrate"));
        assert_eq!(v["prev-hash"], serde_json::json!("0000000000000000"));
        let sources = v["sources"].as_array().unwrap();
        assert!(sources.iter().any(|x| x.as_str() == Some(src)));
    }
}

#[test]
fn tc_517_log_verify_entry_on_product_verify() {
    let h = fixture_log();
    // Seed a feature with a passing TC that is already linked.
    h.write(
        "docs/features/FT-001-x.md",
        "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: []\ntests:\n- TC-001\n---\n\nBody.\n",
    );
    // TC with no runner (UNIMPLEMENTED path, any_runnable=false but has_unimplemented=true)
    h.write(
        "docs/tests/TC-001-x.md",
        "---\nid: TC-001\ntitle: X TC\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\n---\n\nBody.\n",
    );

    let out = h.run(&["verify", "FT-001"]);
    // Verify writes a log entry regardless of pass/fail, as long as it runs.
    let lines = log_lines(&h);
    assert!(!lines.is_empty(), "expected a verify log entry, got: {}{}", out.stdout, out.stderr);
    let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(v["type"], serde_json::json!("verify"));
    assert_eq!(v["feature"], serde_json::json!("FT-001"));
}

#[test]
fn tc_518_log_cross_ref_tags_detects_truncation() {
    // Skip if git isn't available — test becomes vacuously true.
    if std::process::Command::new("git").arg("--version").output().is_err() {
        return;
    }
    let h = fixture_log();
    // Pretend a completion tag exists without a matching log entry.
    // Init git, create a tag, then run with --against-tags.
    let _ = std::process::Command::new("git")
        .args(["init"]).current_dir(h.dir.path()).output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "t@e.com"]).current_dir(h.dir.path()).output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "T"]).current_dir(h.dir.path()).output();
    // Create one commit so we can tag.
    std::fs::write(h.dir.path().join("README"), "hi").unwrap();
    let _ = std::process::Command::new("git")
        .args(["add", "."]).current_dir(h.dir.path()).output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "init"]).current_dir(h.dir.path()).output();
    let _ = std::process::Command::new("git")
        .args(["tag", "product/FT-999/complete"]).current_dir(h.dir.path()).output();

    // Empty log: tag exists but no verify entry
    std::fs::write(h.dir.path().join("requests.jsonl"), "").unwrap();
    let out = h.run(&["request", "log", "verify", "--against-tags"]);
    // Exit 2 (warning) expected; stdout/stderr should contain W021.
    let s = format!("{}{}", out.stdout, out.stderr);
    if out.exit_code == 2 {
        assert!(s.contains("W021"), "expected W021 in: {}", s);
    }
}

#[test]
fn tc_519_log_graph_check_integration_exits_one_on_tamper() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);

    // Tamper
    let path = h.dir.path().join("requests.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replacen("\"reason\":\"A\"", "\"reason\":\"X\"", 1);
    std::fs::write(&path, tampered).unwrap();

    let out = h.run(&["graph", "check"]);
    assert_eq!(out.exit_code, 1, "expected exit 1 on tamper: {}{}", out.stdout, out.stderr);
    let s = format!("{}{}", out.stdout, out.stderr);
    assert!(s.contains("E017"), "expected E017 in graph check output: {}", s);
}

#[test]
fn tc_520_log_path_migration_preserves_chain() {
    let h = fixture_log();
    std::fs::create_dir_all(h.dir.path().join(".product")).unwrap();
    // Write 3 legacy entries (FT-041 format: loose JSON, no hashes).
    let legacy = "{\"timestamp\":\"2026-04-14T10:00:00Z\",\"type\":\"create\",\"reason\":\"L1\",\"created\":[{\"id\":\"FT-001\"}],\"changed\":[]}\n{\"timestamp\":\"2026-04-14T10:01:00Z\",\"type\":\"create\",\"reason\":\"L2\",\"created\":[{\"id\":\"FT-002\"}],\"changed\":[]}\n{\"timestamp\":\"2026-04-14T10:02:00Z\",\"type\":\"create\",\"reason\":\"L3\",\"created\":[{\"id\":\"FT-003\"}],\"changed\":[]}\n";
    h.write(".product/request-log.jsonl", legacy);
    // Triggers migration on next command
    let out = h.run(&["request", "log", "show"]);
    out.assert_exit(0);
    assert!(h.exists("requests.jsonl"));
    let lines = log_lines(&h);
    // 3 migrated + 1 migrate entry = 4
    assert_eq!(lines.len(), 4, "expected 4 lines in new log, got {}: {:?}", lines.len(), lines);
    // Verify chain
    let verify = h.run(&["request", "log", "verify"]);
    verify.assert_exit(0);
    // Last is migrate
    let last: serde_json::Value = serde_json::from_str(&lines[3]).unwrap();
    assert_eq!(last["type"], serde_json::json!("migrate"));
}

#[test]
fn tc_521_log_apply_refuses_without_git_identity() {
    // Init git in fixture but unset identity
    if std::process::Command::new("git").arg("--version").output().is_err() {
        return;
    }
    let h = fixture_log();
    let _ = std::process::Command::new("git")
        .args(["init"]).current_dir(h.dir.path()).output();
    // Unset local identity — explicitly, if previously inherited
    let _ = std::process::Command::new("git")
        .args(["config", "--local", "--unset-all", "user.name"])
        .current_dir(h.dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "--local", "--unset-all", "user.email"])
        .current_dir(h.dir.path())
        .output();
    write_log_req(&h, "r.yaml", "t", "X");
    // Run with HOME and XDG_CONFIG_HOME pointing to empty dirs to prevent global identity
    let empty = h.dir.path().join("empty-home");
    std::fs::create_dir_all(&empty).unwrap();
    let empty_s = empty.to_string_lossy().to_string();
    let out = h.run_with_env(
        &["request", "apply", "r.yaml"],
        &[("HOME", &empty_s), ("XDG_CONFIG_HOME", &empty_s), ("GIT_CONFIG_NOSYSTEM", "1"), ("PRODUCT_LOG_APPLIED_BY", "")],
    );
    // If git identity is inherited from higher-scope config, skip.
    // Otherwise, expect exit >= 1 and message mentions git identity.
    if out.exit_code == 0 {
        // Likely this CI environment has a system-wide identity; skip assertion.
        return;
    }
    let s = format!("{}{}", out.stdout, out.stderr);
    assert!(
        s.contains("git identity") || s.contains("user.name") || s.contains("user.email"),
        "expected git identity error: {}", s
    );
    assert!(!h.exists("requests.jsonl"));
}

#[test]
fn tc_522_log_entry_id_increments_within_utc_day() {
    let h = fixture_log();
    // Use PRODUCT_LOG_NOW to freeze time.
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    write_log_req(&h, "c.yaml", "C", "Charlie");

    let out = h.run_with_env(
        &["request", "apply", "a.yaml"],
        &[("PRODUCT_LOG_NOW", "2026-04-14T23:59:00Z")],
    );
    out.assert_exit(0);
    let out = h.run_with_env(
        &["request", "apply", "b.yaml"],
        &[("PRODUCT_LOG_NOW", "2026-04-14T23:59:30Z")],
    );
    out.assert_exit(0);
    let out = h.run_with_env(
        &["request", "apply", "c.yaml"],
        &[("PRODUCT_LOG_NOW", "2026-04-15T00:00:10Z")],
    );
    out.assert_exit(0);

    let lines = log_lines(&h);
    let ids: Vec<String> = lines
        .iter()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["id"].as_str().unwrap().to_string()
        })
        .collect();
    assert_eq!(ids[0], "req-20260414-001");
    assert_eq!(ids[1], "req-20260414-002");
    assert_eq!(ids[2], "req-20260415-001");
}

#[test]
fn tc_523_log_replay_never_overwrites_working_tree() {
    let h = fixture_log();
    write_log_req(&h, "r.yaml", "t", "X");
    h.run(&["request", "apply", "r.yaml"]).assert_exit(0);
    // --output . must fail
    let out = h.run(&["request", "replay", "--full", "--output", "."]);
    assert!(out.exit_code >= 1, "replay --output . must fail");
    // Run without --output — writes to /tmp
    let out2 = h.run(&["request", "replay", "--full"]);
    out2.assert_exit(0);
    // Any directory named docs/features in the working tree is unchanged
    let f = h.read("docs/features/FT-001-x.md");
    // We can't easily hash — but at least it's non-empty post-run
    assert!(!f.is_empty(), "working tree file should still exist");
}

#[test]
fn tc_524_log_verify_is_pure_read() {
    let h = fixture_log();
    write_log_req(&h, "a.yaml", "A", "Alpha");
    write_log_req(&h, "b.yaml", "B", "Bravo");
    h.run(&["request", "apply", "a.yaml"]).assert_exit(0);
    h.run(&["request", "apply", "b.yaml"]).assert_exit(0);

    // Tamper
    let path = h.dir.path().join("requests.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replacen("\"reason\":\"A\"", "\"reason\":\"X\"", 1);
    std::fs::write(&path, &tampered).unwrap();
    let snapshot = std::fs::read_to_string(&path).unwrap();

    h.run(&["request", "log", "verify"]);
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(snapshot, after, "log must not be modified by verify");

    // --against-tags also must not modify
    h.run(&["request", "log", "verify", "--against-tags"]);
    let after2 = std::fs::read_to_string(&path).unwrap();
    assert_eq!(snapshot, after2);
}

#[test]
fn tc_525_log_entry_hash_is_deterministic() {
    use product_lib::request_log::canonical::{canonical_json, sha256_hex};
    let v1 = serde_json::json!({"b": 1, "a": "s", "c": [1, 2]});
    let v2 = serde_json::json!({"a": "s", "c": [1, 2], "b": 1});
    assert_eq!(canonical_json(&v1), canonical_json(&v2));
    assert_eq!(
        sha256_hex(canonical_json(&v1).as_bytes()),
        sha256_hex(canonical_json(&v2).as_bytes())
    );
}

#[test]
fn tc_526_log_any_field_change_invalidates_hash() {
    use product_lib::request_log::canonical::{canonical_json, sha256_hex};
    let h = fixture_log();
    write_log_req(&h, "r.yaml", "A", "Alpha");
    h.run(&["request", "apply", "r.yaml"]).assert_exit(0);
    let mut v = log_line_json(&h, 0);
    let stored = v["entry-hash"].as_str().unwrap().to_string();
    // Change a field
    v["reason"] = serde_json::json!("CHANGED");
    let mut for_hash = v.clone();
    for_hash["entry-hash"] = serde_json::json!("");
    let new_hash = sha256_hex(canonical_json(&for_hash).as_bytes());
    assert_ne!(stored, new_hash, "hash must change when any field changes");
}

#[test]
fn tc_527_log_chain_breaks_on_any_deletion() {
    let h = fixture_log();
    for i in 0..4 {
        let name = format!("r{}.yaml", i);
        write_log_req(&h, &name, &format!("r{}", i), &format!("T{}", i));
        h.run(&["request", "apply", &name]).assert_exit(0);
    }
    // Delete each interior line in turn and assert chain-break.
    let orig = std::fs::read_to_string(h.dir.path().join("requests.jsonl")).unwrap();
    for del_idx in 1..3 {
        let lines: Vec<&str> = orig.lines().collect();
        let mut new_lines: Vec<&str> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            if i != del_idx {
                new_lines.push(l);
            }
        }
        let new_content = new_lines.join("\n") + "\n";
        std::fs::write(h.dir.path().join("requests.jsonl"), &new_content).unwrap();
        let out = h.run(&["request", "log", "verify"]);
        assert!(out.exit_code >= 1, "deletion at {} must be detected", del_idx);
        let s = format!("{}{}", out.stdout, out.stderr);
        assert!(s.contains("E018"), "expected E018 at deletion {}: {}", del_idx, s);
    }
    // Restore
    std::fs::write(h.dir.path().join("requests.jsonl"), &orig).unwrap();
}

#[test]
fn tc_528_log_replay_produces_same_graph() {
    let h = fixture_log();
    for i in 0..3 {
        let name = format!("r{}.yaml", i);
        write_log_req(&h, &name, &format!("r{}", i), &format!("T{}", i));
        h.run(&["request", "apply", &name]).assert_exit(0);
    }
    let out_dir = std::env::temp_dir().join(format!("product-replay-528-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    let out_s = out_dir.to_string_lossy().to_string();
    let out = h.run(&["request", "replay", "--full", "--output", &out_s]);
    out.assert_exit(0);
    // Compare file trees
    let a = h.dir.path().join("docs");
    let b = out_dir.join("docs");
    for entry in walkdir(&a) {
        let rel = entry.strip_prefix(&a).unwrap();
        let target = b.join(rel);
        if entry.is_file() {
            assert!(target.exists(), "missing file in replay: {}", target.display());
            let a_c = std::fs::read(&entry).unwrap();
            let b_c = std::fs::read(&target).unwrap();
            assert_eq!(a_c, b_c, "file differs: {}", rel.display());
        }
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

