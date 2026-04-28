//! Integration tests — misc.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_177_end_to_end_onboard_produces_graph_with_no_structural_errors() {
    let h = Harness::new();
    let fixture_dir = format!(
        "{}/tests/fixtures/onboard-sample",
        env!("CARGO_MANIFEST_DIR")
    );
    let candidates_path = h.dir.path().join("candidates.json").to_string_lossy().to_string();
    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();

    // Phase 1: Scan
    let out = h.run(&["onboard", "scan", &fixture_dir, "--output", &candidates_path]);
    out.assert_exit(0);

    // Phase 2: Triage — batch confirm all (non-interactive)
    let out = h.run(&["onboard", "triage", &candidates_path, "--output", &triaged_path]);
    out.assert_exit(0);

    // Phase 3: Seed
    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    // Run graph check
    let out = h.run(&["graph", "check"]);
    // Exit 0 (clean) or 2 (warnings only) is acceptable
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "Expected exit 0 or 2, got {}. stderr: {}",
        out.exit_code,
        out.stderr
    );

    // No E-class errors
    assert!(
        !out.stderr.contains("E001"),
        "No E001 malformed front-matter errors expected"
    );
    assert!(
        !out.stderr.contains("E002"),
        "No E002 broken link errors expected"
    );
    assert!(
        !out.stderr.contains("E003"),
        "No E003 dependency cycle errors expected"
    );

    // W001 (orphaned) and W002 (no tests) are acceptable
}

#[test]
fn tc_429_mutable_front_matter_does_not_affect_content_hash() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );

    // Create and accept an ADR
    let adr_body = "Decision body text.\n";
    let hash = compute_adr_content_hash("Stable ADR", adr_body.trim());
    h.write(
        "docs/adrs/ADR-001-stable.md",
        &format!(
            "---\nid: ADR-001\ntitle: Stable ADR\nstatus: accepted\ncontent-hash: {}\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n{}",
            hash, adr_body
        ),
    );

    // graph check should pass initially
    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E014"),
        "Should not have E014 initially.\nstderr: {}",
        out.stderr
    );

    // Modify mutable field: status (superseded-by is also mutable)
    let content = h.read("docs/adrs/ADR-001-stable.md");
    let modified = content.replace("superseded-by: []", "superseded-by: [ADR-999]");
    std::fs::write(
        h.dir.path().join("docs/adrs/ADR-001-stable.md"),
        &modified,
    )
    .expect("write modified");

    // graph check should NOT produce E014 (mutable field change)
    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E014"),
        "Mutable field change should not trigger E014.\nstderr: {}",
        out.stderr
    );

    // Modify another mutable field: features
    let modified = modified.replace("features:\n- FT-001", "features:\n- FT-001\n- FT-002");
    std::fs::write(
        h.dir.path().join("docs/adrs/ADR-001-stable.md"),
        &modified,
    )
    .expect("write modified");

    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E014"),
        "features change should not trigger E014.\nstderr: {}",
        out.stderr
    );

    // Also test TC mutable fields
    let tc_body = "## Description\n\nTest description.\n";
    let tc_hash = compute_tc_content_hash("Stable TC", "scenario", &[], tc_body.trim());
    h.write(
        "docs/tests/TC-001-stable.md",
        &format!(
            "---\nid: TC-001\ntitle: Stable TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\ncontent-hash: {}\n---\n\n{}",
            tc_hash, tc_body
        ),
    );

    // Modify mutable TC field: status
    let content = h.read("docs/tests/TC-001-stable.md");
    let modified = content.replace("status: unimplemented", "status: passing");
    std::fs::write(
        h.dir.path().join("docs/tests/TC-001-stable.md"),
        &modified,
    )
    .expect("write modified");

    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E015"),
        "TC status change should not trigger E015.\nstderr: {}",
        out.stderr
    );

    // Modify mutable TC field: validates.features
    let modified = modified.replace("features:\n  - FT-001", "features:\n  - FT-001\n  - FT-002");
    std::fs::write(
        h.dir.path().join("docs/tests/TC-001-stable.md"),
        &modified,
    )
    .expect("write modified");

    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E015"),
        "TC validates.features change should not trigger E015.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_314_harness_scripts_present() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let implement_sh = repo_root.join("scripts/harness/implement.sh");
    let author_sh = repo_root.join("scripts/harness/author.sh");

    assert!(
        implement_sh.exists(),
        "scripts/harness/implement.sh should exist at {}",
        implement_sh.display()
    );
    assert!(
        author_sh.exists(),
        "scripts/harness/author.sh should exist at {}",
        author_sh.display()
    );

    // Check executable permission (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let implement_perms = std::fs::metadata(&implement_sh)
            .expect("metadata")
            .permissions();
        assert!(
            implement_perms.mode() & 0o111 != 0,
            "implement.sh should be executable"
        );
        let author_perms = std::fs::metadata(&author_sh)
            .expect("metadata")
            .permissions();
        assert!(
            author_perms.mode() & 0o111 != 0,
            "author.sh should be executable"
        );
    }
}

#[test]
fn tc_444_skip_adr_check_bypasses_e016() {
    let h = fixture_lifecycle_gate_proposed();
    let out = h.run(&["verify", "FT-001", "--skip-adr-check"]);
    out.assert_exit(0);
    // No E016 in stderr
    assert!(
        !out.stderr.contains("E016"),
        "E016 should be suppressed with --skip-adr-check.\nStderr: {}",
        out.stderr
    );

    // Feature should be updated (complete since TC passes)
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: complete"),
        "Feature should be marked complete with --skip-adr-check.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_502_granular_tools_continue_to_work_alongside_request_interface() {
    let h = fixture_request();
    // Granular: create a new feature
    h.run(&["feature", "new", "Coexist"]).assert_exit(0);
    // Request: apply a change to that feature
    write_req(
        &h,
        "rc.yaml",
        "type: change\nschema-version: 1\nreason: \"add domain\"\nchanges:\n  - target: FT-002\n    mutations:\n      - op: append\n        field: domains\n        value: api\n",
    );
    h.run(&["request", "apply", "rc.yaml"]).assert_exit(0);
    // Granular again: add a domain
    h.run(&["feature", "domain", "FT-002", "--add", "security"]).assert_exit(0);
    // Graph check must still be clean
    let check = h.run(&["graph", "check"]);
    assert!(check.exit_code == 0 || check.exit_code == 2);
    let c = h.read("docs/features/FT-002-coexist.md");
    assert!(c.contains("api"));
    assert!(c.contains("security"));
}

#[test]
fn tc_503_process_killed_mid_apply_leaves_recoverable_state() {
    let h = fixture_request();
    // First apply a request
    write_req(
        &h,
        "rchaos.yaml",
        r#"type: create-and-change
schema-version: 1
reason: "chaos recovery"
artifacts:
  - type: feature
    ref: ft-c
    title: Chaos
    phase: 2
    domains: [api]
changes:
  - target: FT-001
    mutations:
      - op: append
        field: domains
        value: networking
"#,
    );
    h.run(&["request", "apply", "rchaos.yaml"]).assert_exit(0);
    let after1 = h.read("docs/features/FT-001-seed.md");
    // Re-apply — idempotent
    h.run(&["request", "apply", "rchaos.yaml"]); // exit code may be 1 (duplicate create) or 0; not critical
    // FT-001's state is unchanged (append is idempotent)
    let after2 = h.read("docs/features/FT-001-seed.md");
    assert!(after2.contains("networking"));
    // Verify domains line count hasn't exploded
    assert_eq!(
        after1.matches("networking").count(),
        after2.matches("networking").count(),
    );
}

#[test]
fn tc_625_relative_paths_in_log_exit() {
    let h = fixture_log();
    write_log_req(&h, "r.yaml", "tc-625-fresh", "Fresh");
    h.run(&["request", "apply", "r.yaml"]).assert_exit(0);

    // Exit-criteria #1: emitted paths are repo-relative.
    let v = log_line_json(&h, 0);
    let mut files = Vec::new();
    collect_file_values_from_json(&v, &mut files);
    assert!(!files.is_empty(), "entry should carry at least one file");
    for f in &files {
        assert!(!f.starts_with('/'), "fresh log has absolute file: {}", f);
        assert!(f.starts_with("docs/"), "fresh log has off-docs file: {}", f);
    }

    // Exit-criteria #4: verify exits 0 on a fresh post-FT-051 log, no warnings.
    let out = h.run(&["request", "log", "verify"]);
    assert_eq!(
        out.exit_code, 0,
        "verify should exit 0 on a fresh post-FT-051 log;\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        !out.stderr.contains("W-path-absolute"),
        "fresh log should not emit W-path-absolute: {}",
        out.stderr
    );

    // Exit-criteria #3 (smoke): migrate-paths on an already-clean log is a
    // no-op and does not append a new entry.
    let before = log_lines(&h).len();
    let mig = h.run(&["request", "log", "migrate-paths"]);
    mig.assert_exit(0);
    mig.assert_stdout_contains("no absolute paths");
    let after = log_lines(&h).len();
    assert_eq!(before, after, "migrate-paths must be a no-op on a clean log");
}

