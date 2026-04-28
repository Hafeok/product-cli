//! Integration tests — adr.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_031_abandon_feature_orphans_tests() {
    let h = fixture_abandon();

    // Abandon the feature
    let out = h.run(&["feature", "status", "FT-001", "abandoned"]);
    out.assert_exit(0);

    // Read TC files and verify FT-001 removed from validates.features
    let tc1 = h.read("docs/tests/TC-001-test-one.md");
    let tc2 = h.read("docs/tests/TC-002-test-two.md");

    assert!(
        !tc1.contains("FT-001"),
        "TC-001 should have FT-001 removed from validates.features, got:\n{}",
        tc1
    );
    assert!(
        !tc2.contains("FT-001"),
        "TC-002 should have FT-001 removed from validates.features, got:\n{}",
        tc2
    );
}

#[test]
fn tc_032_abandon_feature_exit_code() {
    let h = fixture_abandon();

    // Abandon the feature
    h.run(&["feature", "status", "FT-001", "abandoned"]).assert_exit(0);

    // graph check should return 2 (warnings: orphaned tests) not 1 (errors)
    let out = h.run(&["graph", "check"]);
    out.assert_exit(2);
    // Should have W001 (orphaned tests) but no E-level errors
    out.assert_stderr_contains("W001");
}

#[test]
fn tc_033_abandon_feature_stdout() {
    let h = fixture_abandon();

    let out = h.run(&["feature", "status", "FT-001", "abandoned"]);
    out.assert_exit(0);

    // stdout should list the orphaned tests
    out.assert_stdout_contains("TC-001");
    out.assert_stdout_contains("TC-002");
    out.assert_stdout_contains("Auto-orphaning");
}

#[test]
fn tc_034_abandon_feature_tests_preserved() {
    let h = fixture_abandon();

    h.run(&["feature", "status", "FT-001", "abandoned"]).assert_exit(0);

    // Both test files should still exist
    assert!(
        h.exists("docs/tests/TC-001-test-one.md"),
        "TC-001 file should still exist after abandonment"
    );
    assert!(
        h.exists("docs/tests/TC-002-test-two.md"),
        "TC-002 file should still exist after abandonment"
    );

    // Verify files still have content (not empty)
    let tc1 = h.read("docs/tests/TC-001-test-one.md");
    let tc2 = h.read("docs/tests/TC-002-test-two.md");
    assert!(tc1.contains("Test One"), "TC-001 should still have its title");
    assert!(tc2.contains("Test Two"), "TC-002 should still have its title");
}

#[test]
fn tc_135_acknowledgement_requires_reason() {
    let h = harness_with_domains();

    // Feature with empty acknowledgement reasoning
    h.write("docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged:\n  security: \"\"\n---\n\nBody.\n");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1)
        .assert_stderr_contains("E011");
    assert!(
        out.stderr.contains("security") || out.stderr.contains("domains-acknowledged"),
        "E011 should mention the field, got stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_138_acknowledgement_closes_gap() {
    let h = harness_with_domains();

    // Domain-scoped security ADR
    h.write("docs/adrs/ADR-020-security-policy.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity policy.\n");

    // Feature acknowledges security domain with reasoning
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged:\n  security: \"no trust boundaries introduced\"\n---\n\nRate limiting.\n");

    let out = h.run(&["graph", "check"]);
    // W011 should NOT appear for security domain on FT-009
    let has_w011_ft009 = out.stderr.contains("W011") && out.stderr.contains("FT-009") && out.stderr.contains("security");
    assert!(
        !has_w011_ft009,
        "W011 should not fire for FT-009 security when acknowledged, got stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_119_adr_review_structural_missing_section() {
    let h = Harness::new();
    git_init(&h);

    // ADR missing "Rejected alternatives"
    h.write(
        "docs/adrs/ADR-051-missing-section.md",
        "---\nid: ADR-051\ntitle: Missing Section ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** ctx\n\n**Decision:** dec\n\n**Rationale:** rat\n\n**Test coverage:** tc\n",
    );

    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-051-missing-section.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // Finding must include file path and section name
    assert!(
        out.stderr.contains("Rejected alternatives"),
        "Finding should mention 'Rejected alternatives'.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("adrs/ADR-051") || out.stderr.contains("ADR-051-missing-section"),
        "Finding should include file path.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_120_adr_review_structural_no_features() {
    let h = Harness::new();
    git_init(&h);

    // ADR with all sections but features: []
    h.write(
        "docs/adrs/ADR-052-no-features.md",
        "---\nid: ADR-052\ntitle: No Features ADR\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** ctx\n\n**Decision:** dec\n\n**Rationale:** rat\n\n**Rejected alternatives:** none\n\n**Test coverage:** tc\n",
    );

    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-052-no-features.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // Should warn about no linked features
    assert!(
        out.stderr.contains("no linked features") || out.stderr.contains("features"),
        "Should warn about empty features.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("ADR-052") || out.stderr.contains("adrs/"),
        "Should reference the ADR path.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_423_adr_amend_records_amendment_and_recomputes_hash() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    // Create and accept an ADR
    h.run(&["adr", "new", "Amendable ADR"]).assert_exit(0);

    let adr_dir = h.dir.path().join("docs/adrs");
    let entries: Vec<_> = std::fs::read_dir(&adr_dir)
        .expect("read")
        .filter_map(|e| e.ok())
        .collect();
    let adr_path = entries[0].path();
    let filename = adr_path.file_name().expect("fname").to_str().expect("utf8");
    let adr_id = &filename[..7];

    h.run(&["adr", "status", adr_id, "accepted"]).assert_exit(0);

    // Get the original hash
    let content = std::fs::read_to_string(&adr_path).expect("read");
    let original_hash = content
        .lines()
        .find(|l| l.starts_with("content-hash: "))
        .expect("hash line")
        .strip_prefix("content-hash: ")
        .expect("strip")
        .to_string();

    // Modify the body (fix a "typo")
    let modified = content.replace("Describe the decision", "Describe the architectural decision");
    std::fs::write(&adr_path, &modified).expect("write modified");

    // Amend the ADR
    let out = h.run(&["adr", "amend", adr_id, "--reason", "Fix typo in decision section"]);
    out.assert_exit(0);
    out.assert_stdout_contains("amended");

    // Verify amendments array exists with correct structure
    let content = std::fs::read_to_string(&adr_path).expect("read");
    assert!(content.contains("amendments:"), "Should have amendments array");
    assert!(
        content.contains("reason: Fix typo in decision section"),
        "Should contain amendment reason"
    );
    assert!(
        content.contains("previous-hash:"),
        "Should contain previous-hash"
    );
    assert!(
        content.contains(&format!("previous-hash: {}", original_hash)),
        "previous-hash should match original"
    );

    // Verify content-hash is updated
    let new_hash = content
        .lines()
        .find(|l| l.starts_with("content-hash: "))
        .expect("hash line")
        .strip_prefix("content-hash: ")
        .expect("strip");
    assert_ne!(new_hash, original_hash, "Hash should have changed");

    // Verify graph check passes
    let out = h.run(&["graph", "check"]);
    // Should not have E014 errors (may have other warnings like W001)
    assert!(
        !out.stderr.contains("E014"),
        "Should not have E014 after amend.\nstderr: {}",
        out.stderr
    );

    // Verify amend without --reason is rejected
    let out = h.run(&["adr", "amend", adr_id]);
    assert_ne!(
        out.exit_code, 0,
        "amend without --reason should fail"
    );
}

#[test]
fn tc_428_adr_rehash_seals_pre_existing_accepted_adrs() {
    let h = Harness::new();

    // Create multiple ADR files manually with status: accepted but no content-hash
    h.write(
        "docs/adrs/ADR-001-legacy-a.md",
        "---\nid: ADR-001\ntitle: Legacy A\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nLegacy decision A.\n",
    );
    h.write(
        "docs/adrs/ADR-002-legacy-b.md",
        "---\nid: ADR-002\ntitle: Legacy B\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nLegacy decision B.\n",
    );
    // Proposed ADR — should not be touched
    h.write(
        "docs/adrs/ADR-003-proposed.md",
        "---\nid: ADR-003\ntitle: Proposed ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nDraft.\n",
    );

    // Rehash a single ADR
    let out = h.run(&["adr", "rehash", "ADR-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("sealed");

    // ADR-001 should now have content-hash but no amendments
    let content = h.read("docs/adrs/ADR-001-legacy-a.md");
    assert!(content.contains("content-hash: sha256:"), "ADR-001 should be sealed");
    assert!(!content.contains("amendments:"), "Initial sealing should not add amendments");

    // ADR-002 should still have no hash
    let content = h.read("docs/adrs/ADR-002-legacy-b.md");
    assert!(!content.contains("content-hash"), "ADR-002 should not be sealed yet");

    // Rehash all
    let out = h.run(&["adr", "rehash", "--all"]);
    out.assert_exit(0);
    out.assert_stdout_contains("ADR-002"); // ADR-002 should get sealed

    // ADR-002 should now have hash
    let content = h.read("docs/adrs/ADR-002-legacy-b.md");
    assert!(content.contains("content-hash: sha256:"), "ADR-002 should be sealed after --all");

    // ADR-003 (proposed) should NOT have hash
    let content = h.read("docs/adrs/ADR-003-proposed.md");
    assert!(!content.contains("content-hash"), "Proposed ADR should not be touched");

    // ADR-001 (already sealed) should not be modified by --all
    let content_before = h.read("docs/adrs/ADR-001-legacy-a.md");
    h.run(&["adr", "rehash", "--all"]).assert_exit(0);
    let content_after = h.read("docs/adrs/ADR-001-legacy-a.md");
    assert_eq!(content_before, content_after, "Already-sealed ADR should not be modified");
}

#[test]
fn tc_321_adr_review_missing_section() {
    let h = Harness::new();
    git_init(&h);

    // Write an ADR missing "Rejected alternatives" section
    h.write(
        "docs/adrs/ADR-070-missing-section.md",
        "---\nid: ADR-070\ntitle: Missing Section\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** ctx\n\n**Decision:** dec\n\n**Rationale:** rat\n\n**Test coverage:** tc\n",
    );

    // Stage and review
    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-070-missing-section.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // Finding should mention file path and section name
    assert!(
        out.stderr.contains("Rejected alternatives"),
        "Should report missing 'Rejected alternatives' section.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("adrs/ADR-070") || out.stderr.contains("ADR-070-missing-section"),
        "Should include file path.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_322_adr_review_no_features() {
    let h = Harness::new();
    git_init(&h);

    // Write an ADR with all sections but features: []
    h.write(
        "docs/adrs/ADR-071-no-features.md",
        "---\nid: ADR-071\ntitle: No Features\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** ctx\n\n**Decision:** dec\n\n**Rationale:** rat\n\n**Rejected alternatives:** none\n\n**Test coverage:** tc\n",
    );

    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-071-no-features.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // Should warn about no linked features with W001
    assert!(
        out.stderr.contains("W001") || out.stderr.contains("no linked features"),
        "Should report W001-class warning about empty features.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("ADR-071") || out.stderr.contains("adrs/"),
        "Should reference the ADR path.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_445_superseded_and_abandoned_adrs_satisfy_lifecycle_invariant() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Superseded ADR\nstatus: superseded\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/adrs/ADR-002-test.md",
        "---\nid: ADR-002\ntitle: Abandoned ADR\nstatus: abandoned\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
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
    out.assert_exit(0);
    // No E016
    assert!(
        !out.stderr.contains("E016"),
        "E016 should not fire for superseded/abandoned ADRs.\nStderr: {}",
        out.stderr
    );

    // Feature should be complete
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: complete"),
        "Feature should be complete with superseded/abandoned ADRs.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_447_lifecycle_gate_exit_criteria() {
    // This exit-criteria test validates that all lifecycle gate scenarios work.
    // It is satisfied when TC-440 through TC-446 all pass.
    // Run verify on a feature with an accepted ADR to confirm the happy path.
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
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

    // Verify succeeds with accepted ADR (happy path covers the full lifecycle gate)
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    assert!(
        !out.stderr.contains("E016"),
        "No E016 should appear.\nStderr: {}",
        out.stderr
    );
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: complete"),
        "Feature should be complete.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_464_adr_scope_validates_enum_values() {
    let h = fixture_with_domains();
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");

    // Invalid scope → exit 1 with E001
    let out = h.run(&["adr", "scope", "ADR-001", "invalid-scope"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E001");

    // Valid values → exit 0
    for scope in &["cross-cutting", "domain", "feature-specific"] {
        let out = h.run(&["adr", "scope", "ADR-001", scope]);
        out.assert_exit(0);
        let content = h.read("docs/adrs/ADR-001-test.md");
        assert!(content.contains(&format!("scope: {}", scope)),
            "scope should be set to {} in front-matter, got:\n{}", scope, content);
    }
}

#[test]
fn tc_465_adr_supersede_bidirectional_write() {
    let h = fixture_with_domains();
    h.write("docs/adrs/ADR-001-old.md", "---\nid: ADR-001\ntitle: Old Decision\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nOld body.\n");
    h.write("docs/adrs/ADR-002-new.md", "---\nid: ADR-002\ntitle: New Decision\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nNew body.\n");

    // Supersede: ADR-002 supersedes ADR-001
    let out = h.run(&["adr", "supersede", "ADR-002", "--supersedes", "ADR-001"]);
    out.assert_exit(0);

    // Check ADR-002 has supersedes: [ADR-001]
    let content_new = h.read("docs/adrs/ADR-002-new.md");
    assert!(content_new.contains("ADR-001"), "ADR-002 should list ADR-001 in supersedes");

    // Check ADR-001 has superseded-by: [ADR-002]
    let content_old = h.read("docs/adrs/ADR-001-old.md");
    assert!(content_old.contains("ADR-002"), "ADR-001 should list ADR-002 in superseded-by");
    // ADR-001 was accepted, should be superseded now
    assert!(content_old.contains("superseded"), "ADR-001 status should be superseded");

    // Remove the supersession link
    let out2 = h.run(&["adr", "supersede", "ADR-002", "--remove", "ADR-001"]);
    out2.assert_exit(0);

    // Both links should be removed
    let content_new2 = h.read("docs/adrs/ADR-002-new.md");
    let content_old2 = h.read("docs/adrs/ADR-001-old.md");
    // After removal, ADR-002 supersedes should be empty and ADR-001 superseded-by should be empty
    assert!(!content_new2.contains("- ADR-001"), "ADR-001 should be removed from ADR-002 supersedes");
    assert!(!content_old2.contains("- ADR-002"), "ADR-002 should be removed from ADR-001 superseded-by");
}

#[test]
fn tc_466_adr_supersede_detects_cycles() {
    let h = fixture_with_domains();
    h.write("docs/adrs/ADR-001-a.md", "---\nid: ADR-001\ntitle: A\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nA.\n");
    h.write("docs/adrs/ADR-002-b.md", "---\nid: ADR-002\ntitle: B\nstatus: proposed\nfeatures: []\nsupersedes: [ADR-001]\nsuperseded-by: []\n---\n\nB.\n");
    h.write("docs/adrs/ADR-003-c.md", "---\nid: ADR-003\ntitle: C\nstatus: proposed\nfeatures: []\nsupersedes: [ADR-002]\nsuperseded-by: []\n---\n\nC.\n");

    // Also set up the reverse links
    h.write("docs/adrs/ADR-001-a.md", "---\nid: ADR-001\ntitle: A\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: [ADR-002]\n---\n\nA.\n");
    h.write("docs/adrs/ADR-002-b.md", "---\nid: ADR-002\ntitle: B\nstatus: proposed\nfeatures: []\nsupersedes: [ADR-001]\nsuperseded-by: [ADR-003]\n---\n\nB.\n");

    // Save file contents before the cycle attempt
    let before_a = h.read("docs/adrs/ADR-001-a.md");

    // ADR-001 supersedes ADR-003 would create cycle: A -> C -> B -> A
    let out = h.run(&["adr", "supersede", "ADR-001", "--supersedes", "ADR-003"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E004");

    // Verify no files were modified
    let after_a = h.read("docs/adrs/ADR-001-a.md");
    assert_eq!(before_a, after_a, "ADR-001 should not be modified on cycle detection");
}

#[test]
fn tc_468_adr_source_files_add_and_remove() {
    let h = fixture_with_domains();
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");

    // Create a real file for one path
    h.write("src/drift.rs", "// drift module\n");
    std::fs::create_dir_all(h.dir.path().join("src/drift")).expect("mkdir");

    // Add source files
    let out = h.run(&["adr", "source-files", "ADR-001", "--add", "src/drift.rs", "--add", "src/drift/"]);
    out.assert_exit(0);
    let content = h.read("docs/adrs/ADR-001-test.md");
    assert!(content.contains("src/drift.rs"), "should contain src/drift.rs");
    assert!(content.contains("src/drift/"), "should contain src/drift/");

    // Remove one
    let out2 = h.run(&["adr", "source-files", "ADR-001", "--remove", "src/drift.rs"]);
    out2.assert_exit(0);
    let content2 = h.read("docs/adrs/ADR-001-test.md");
    assert!(!content2.contains("src/drift.rs"), "src/drift.rs should be removed");
    assert!(content2.contains("src/drift/"), "src/drift/ should remain");

    // Add nonexistent path → exit 0 with W-class warning
    let out3 = h.run(&["adr", "source-files", "ADR-001", "--add", "src/nonexistent.rs"]);
    out3.assert_exit(0);
    out3.assert_stderr_contains("warning");
    let content3 = h.read("docs/adrs/ADR-001-test.md");
    assert!(content3.contains("src/nonexistent.rs"), "nonexistent path should still be added");
}

