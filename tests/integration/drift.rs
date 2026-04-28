//! Integration tests — drift.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_121_drift_check_d002_detected() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-consensus.md",
        "---\nid: FT-001\ntitle: Consensus\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nConsensus feature.\n",
    );
    h.write(
        "docs/adrs/ADR-002-consensus.md",
        "---\nid: ADR-002\ntitle: Use openraft for consensus\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n## Decision\n\nWe will use `openraft` as the consensus library for cluster coordination.\n\n**Rejected alternatives:**\n- Custom Raft implementation\n",
    );
    // Source file uses a custom Raft struct, not openraft
    h.write(
        "src/consensus/raft.rs",
        "// Custom consensus implementation\npub struct CustomRaft {\n    term: u64,\n    voted_for: Option<u64>,\n    log: Vec<Entry>,\n}\n\nimpl CustomRaft {\n    pub fn new() -> Self {\n        Self { term: 0, voted_for: None, log: vec![] }\n    }\n}\n",
    );
    let out = h.run(&["drift", "check", "ADR-002", "--files", "src/consensus/raft.rs"]);
    // Should find D002 — code overrides decision (uses custom instead of openraft)
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        combined.contains("D002"),
        "Expected D002 finding for overridden decision, got:\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
}

#[test]
fn tc_122_drift_check_d001_detected() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-iface.md",
        "---\nid: FT-001\ntitle: Interface\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-003]\ntests: []\n---\n\nInterface feature.\n",
    );
    h.write(
        "docs/adrs/ADR-003-interface.md",
        "---\nid: ADR-003\ntitle: Consensus Interface\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n## Decision\n\nImplement the `ConsensusInterface` trait for all cluster nodes.\n\n**Rejected alternatives:**\n- None\n",
    );
    // Source file is minimal — no ConsensusInterface implemented
    h.write(
        "src/nodes.rs",
        "// TODO: implement\n",
    );
    let out = h.run(&["drift", "check", "ADR-003", "--files", "src/nodes.rs"]);
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        combined.contains("D001"),
        "Expected D001 finding for unimplemented decision, got:\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
}

#[test]
fn tc_123_drift_scan_returns_adrs() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-consensus.md",
        "---\nid: FT-001\ntitle: Consensus\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nConsensus feature.\n",
    );
    h.write(
        "docs/adrs/ADR-002-consensus.md",
        "---\nid: ADR-002\ntitle: Use openraft for consensus\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nsource-files:\n  - src/consensus/raft.rs\n\n## Decision\n\nUse openraft.\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "src/consensus/raft.rs",
        "// Implements ADR-002 consensus\nuse openraft;\nfn leader() {}\n",
    );
    let out = h.run(&["drift", "scan", "src/consensus/raft.rs"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("ADR-002"),
        "Expected ADR-002 in scan output, got: {}",
        out.stdout
    );
}

#[test]
fn tc_124_drift_suppressed_passes() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-consensus.md",
        "---\nid: FT-001\ntitle: Consensus\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nConsensus feature.\n",
    );
    h.write(
        "docs/adrs/ADR-002-consensus.md",
        "---\nid: ADR-002\ntitle: Use openraft for consensus\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n## Decision\n\nWe will use `openraft` as the consensus library.\n\n**Rejected alternatives:**\n- Custom Raft\n",
    );
    h.write(
        "src/consensus/raft.rs",
        "// Custom consensus implementation\npub struct CustomRaft {\n    term: u64,\n    voted_for: Option<u64>,\n    log: Vec<Entry>,\n}\n\nimpl CustomRaft {\n    pub fn new() -> Self {\n        Self { term: 0, voted_for: None, log: vec![] }\n    }\n}\n",
    );

    // First, check that drift IS detected
    let out1 = h.run(&["drift", "check", "ADR-002", "--files", "src/consensus/raft.rs"]);
    let combined1 = format!("{}{}", out1.stdout, out1.stderr);
    assert!(combined1.contains("D002"), "Should detect D002 before suppression");

    // Extract the drift ID from the output
    let drift_id = out1.stdout.lines()
        .chain(out1.stderr.lines())
        .find(|l| l.contains("DRIFT-ADR-002-D002"))
        .and_then(|l| {
            l.split_whitespace()
                .find(|w| w.starts_with("DRIFT-ADR-002-D002"))
        })
        .unwrap_or("DRIFT-ADR-002-D002-unknown");

    // Suppress it
    let out2 = h.run(&["drift", "suppress", drift_id, "--reason", "Intentional for phase 2"]);
    out2.assert_exit(0);

    // Now drift check should exit 0 (suppressed findings don't trigger failure)
    let out3 = h.run(&["drift", "check", "ADR-002", "--files", "src/consensus/raft.rs"]);
    out3.assert_exit(0);
}

#[test]
fn tc_125_drift_source_files_frontmatter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-consensus.md",
        "---\nid: FT-001\ntitle: Consensus\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nConsensus feature.\n",
    );
    // ADR with source-files in body
    h.write(
        "docs/adrs/ADR-002-consensus.md",
        "---\nid: ADR-002\ntitle: Use openraft\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nsource-files:\n  - src/consensus/raft.rs\n  - src/consensus/leader.rs\n\n## Decision\n\nUse openraft.\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write("src/consensus/raft.rs", "// ADR-002 openraft impl\nuse openraft;\n");
    h.write("src/consensus/leader.rs", "// ADR-002 leader election\nuse openraft;\n");
    // This file should NOT be picked up since front-matter overrides pattern matching
    h.write("src/other/ADR-002-mention.rs", "// mentions ADR-002 but should not be used\n");

    let out = h.run(&["drift", "check", "ADR-002"]);
    out.assert_exit(0);
    // The source-files from front-matter should be used — no D004 since those files exist
    assert!(
        !out.stdout.contains("D004"),
        "Should not get D004 when source-files are specified in front-matter and exist"
    );
}

#[test]
fn tc_171_triage_confirm_converts_candidate_to_adr() {
    let h = Harness::new();

    // Write a single candidate
    let candidates_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "boundary",
                "title": "Database access exclusively through the repository layer",
                "observation": "All database queries are in src/repo/. No other module imports sqlx.",
                "evidence": [
                    {"file": "src/repo/users.rs", "line": 3, "snippet": "use sqlx;", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Adding queries outside src/repo/ would bypass transaction boundaries.",
                "confidence": "high",
                "warnings": []
            }
        ],
        "scan_metadata": {"files_scanned": 10, "prompt_version": "onboard-scan-v1"}
    }"#;

    let candidates_path = h.dir.path().join("candidates.json");
    std::fs::write(&candidates_path, candidates_json).expect("write candidates");

    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();

    // Triage: confirm the candidate
    let out = h.run_with_stdin(
        &[
            "onboard",
            "triage",
            &candidates_path.to_string_lossy(),
            "--interactive",
            "--output",
            &triaged_path,
        ],
        "c\n",
    );
    out.assert_exit(0);
    out.assert_stdout_contains("1 confirmed");

    // Seed the triaged output
    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    // Find the created ADR file
    let adrs_dir = h.dir.path().join("docs/adrs");
    let adr_files: Vec<_> = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("ADR-") && name.ends_with(".md")
        })
        .collect();

    assert!(
        !adr_files.is_empty(),
        "Expected at least one ADR file created"
    );

    // Read the ADR and verify content
    let adr_content = std::fs::read_to_string(adr_files[0].path()).expect("read ADR");
    assert!(
        adr_content.contains("status: proposed"),
        "ADR should have status: proposed"
    );
    assert!(
        adr_content.contains("database") || adr_content.contains("Database") || adr_content.contains("repository"),
        "ADR should contain observation text"
    );
    assert!(
        adr_content.contains("## Context") || adr_content.contains("## Decision"),
        "ADR should have Context/Decision sections"
    );
}

#[test]
fn tc_172_triage_reject_discards_candidate_permanently() {
    let h = Harness::new();

    let candidates_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "boundary",
                "title": "Rejected decision",
                "observation": "Observed pattern to reject",
                "evidence": [
                    {"file": "src/test.rs", "line": 1, "snippet": "test", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Bad things",
                "confidence": "low",
                "warnings": []
            },
            {
                "id": "DC-002",
                "signal_type": "consistency",
                "title": "Confirmed decision",
                "observation": "Observed pattern to confirm",
                "evidence": [
                    {"file": "src/other.rs", "line": 1, "snippet": "test", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Also bad",
                "confidence": "high",
                "warnings": []
            }
        ],
        "scan_metadata": {"files_scanned": 5, "prompt_version": "test"}
    }"#;

    let candidates_path = h.dir.path().join("candidates.json");
    std::fs::write(&candidates_path, candidates_json).expect("write");

    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();

    // Reject DC-001, confirm DC-002
    let out = h.run_with_stdin(
        &[
            "onboard",
            "triage",
            &candidates_path.to_string_lossy(),
            "--interactive",
            "--output",
            &triaged_path,
        ],
        "r\nc\n",
    );
    out.assert_exit(0);
    out.assert_stdout_contains("1 confirmed");
    out.assert_stdout_contains("1 rejected");

    // Verify triaged.json
    let triaged_content = std::fs::read_to_string(&triaged_path).expect("read triaged");
    let triaged: serde_json::Value = serde_json::from_str(&triaged_content).expect("parse");
    let candidates = triaged["candidates"].as_array().expect("candidates");

    // DC-001 should be rejected
    let dc001 = candidates.iter().find(|c| c["id"] == "DC-001").expect("DC-001");
    assert_eq!(dc001["triage_status"], "rejected");

    // DC-002 should be confirmed
    let dc002 = candidates.iter().find(|c| c["id"] == "DC-002").expect("DC-002");
    assert_eq!(dc002["triage_status"], "confirmed");

    // Seed — only DC-002 should become an ADR
    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    // Count ADR files
    let adrs_dir = h.dir.path().join("docs/adrs");
    let adr_count = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("ADR-") && name.ends_with(".md")
        })
        .count();

    assert_eq!(adr_count, 1, "Expected exactly 1 ADR file (rejected should not produce an ADR)");
}

#[test]
fn tc_173_triage_merge_combines_two_candidates_into_one_adr() {
    let h = Harness::new();

    let candidates_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "boundary",
                "title": "Database access exclusively through the repository layer",
                "observation": "All queries are in src/repo/.",
                "evidence": [
                    {"file": "src/repo/users.rs", "line": 3, "snippet": "use sqlx;", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Bypass transaction boundaries.",
                "confidence": "high",
                "warnings": []
            },
            {
                "id": "DC-002",
                "signal_type": "absence",
                "title": "No direct sqlx imports outside the repository module",
                "observation": "No file outside src/repo/ imports sqlx.",
                "evidence": [
                    {"file": "src/handlers/mod.rs", "line": 1, "snippet": "// no sqlx import here", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Adding sqlx outside repo breaks boundary.",
                "confidence": "high",
                "warnings": []
            }
        ],
        "scan_metadata": {"files_scanned": 10, "prompt_version": "test"}
    }"#;

    let candidates_path = h.dir.path().join("candidates.json");
    std::fs::write(&candidates_path, candidates_json).expect("write");

    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();

    // Merge DC-002 into DC-001, then confirm DC-001 (which has DC-002's merge already)
    let out = h.run_with_stdin(
        &[
            "onboard",
            "triage",
            &candidates_path.to_string_lossy(),
            "--interactive",
            "--output",
            &triaged_path,
        ],
        "m\nDC-002\n",
    );
    out.assert_exit(0);

    // Verify triaged output has one confirmed candidate with combined evidence
    let triaged_content = std::fs::read_to_string(&triaged_path).expect("read triaged");
    let triaged: serde_json::Value = serde_json::from_str(&triaged_content).expect("parse");
    let candidates = triaged["candidates"].as_array().expect("candidates");

    // Find confirmed candidates
    let confirmed: Vec<&serde_json::Value> = candidates
        .iter()
        .filter(|c| c["triage_status"] == "confirmed")
        .collect();

    assert_eq!(
        confirmed.len(),
        1,
        "Expected 1 confirmed candidate after merge, got {}",
        confirmed.len()
    );

    // The confirmed candidate should have evidence from both DC-001 and DC-002
    let evidence = confirmed[0]["evidence"].as_array().expect("evidence");
    assert!(
        evidence.len() >= 2,
        "Merged candidate should have evidence from both sources, got {}",
        evidence.len()
    );

    // Seed — should create exactly 1 ADR
    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    let adrs_dir = h.dir.path().join("docs/adrs");
    let adr_count = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("ADR-") && name.ends_with(".md")
        })
        .count();

    assert_eq!(adr_count, 1, "Expected exactly 1 ADR file after merge");

    // Verify evidence from both files appears in the ADR body
    let adr_file = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("ADR-") && name.ends_with(".md")
        })
        .expect("find ADR file");
    let adr_content = std::fs::read_to_string(adr_file.path()).expect("read ADR");
    assert!(
        adr_content.contains("src/repo/users.rs"),
        "ADR should reference src/repo/users.rs evidence"
    );
    assert!(
        adr_content.contains("src/handlers/mod.rs"),
        "ADR should reference src/handlers/mod.rs evidence from merged candidate"
    );
}

#[test]
fn tc_455_drift_check_feature_tag_based() {
    let h = Harness::new();
    h.write("src/foo.rs", "// initial content\nfn main() {}\n");
    git_init_with_commit(&h);

    // Create a completion tag at this commit
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete: 1/1 TCs passing (TC-001)"])
        .current_dir(h.dir.path()).output().expect("tag");

    // Feature must exist
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    // Modify a file after the tag — creating drift
    h.write("src/foo.rs", "// modified content\nfn main() { println!(\"changed\"); }\n");
    git_add_commit(&h, "modify foo.rs after completion");

    let out = h.run(&["drift", "check", "FT-001"]);
    // Under FT-045 the structural drift check exits 2 when changes are
    // detected (changes since completion tag).
    out.assert_exit(2);
    assert!(
        out.stdout.contains("src/foo.rs") || out.stdout.contains("Changed files"),
        "Should report drift on changed files.\nStdout: {}", out.stdout
    );

    // No-drift case: check a feature whose files haven't changed.
    h.write(
        "docs/features/FT-002-test.md",
        "---\nid: FT-002\ntitle: Other Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: []\n---\n\nFeature body.\n",
    );
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-002/complete", "-m", "FT-002 complete"])
        .current_dir(h.dir.path()).output().expect("tag FT-002");
    git_add_commit(&h, "add FT-002");

    let out2 = h.run(&["drift", "check", "FT-002"]);
    out2.assert_exit(0);
    assert!(
        out2.stdout.contains("No changes since completion") || out2.stdout.contains("No drift"),
        "Should report no drift.\nStdout: {}", out2.stdout
    );
}

#[test]
fn tc_456_drift_check_fallback_no_tag() {
    let h = Harness::new();
    git_init_with_commit(&h);

    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\nsource-files:\n  - src/main.rs\n---\n\n**Decision:** Use `openraft` for consensus.\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );
    h.write("src/main.rs", "fn main() {}\n");
    git_add_commit(&h, "add source");

    // No completion tag exists — under FT-045 we emit W020 and fall back
    // to structural ADR drift checks.
    let out = h.run(&["drift", "check", "FT-001"]);
    out.assert_stderr_contains("W020");
    // Should still work (no crash)
    assert!(out.exit_code == 0 || out.exit_code == 1 || out.exit_code == 2,
        "Should exit 0, 1 or 2, not crash. Exit: {}", out.exit_code);
}

#[test]
fn tc_457_drift_check_all_complete() {
    let h = Harness::new();
    h.write("src/a.rs", "fn a() {}\n");
    h.write("src/b.rs", "fn b() {}\n");
    git_init_with_commit(&h);

    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Feature One\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/features/FT-002-test.md",
        "---\nid: FT-002\ntitle: Feature Two\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/features/FT-003-test.md",
        "---\nid: FT-003\ntitle: Feature Three\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );
    git_add_commit(&h, "add features");

    // Create completion tags for FT-001 and FT-002 only
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"])
        .current_dir(h.dir.path()).output().expect("tag FT-001");
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-002/complete", "-m", "FT-002 complete"])
        .current_dir(h.dir.path()).output().expect("tag FT-002");

    let out = h.run(&["drift", "check", "--all-complete"]);
    out.assert_exit(0);

    // Should mention checking complete features
    // FT-003 (in-progress) should be skipped
    assert!(
        !out.stdout.contains("FT-003"),
        "FT-003 (in-progress) should not be checked.\nStdout: {}", out.stdout
    );
}

