//! Integration tests — gap.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_086_gap_check_single_adr() {
    let h = fixture_gap_g001();
    let out = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(
        out.exit_code, 1,
        "Expected exit 1 for ADR with uncovered testable claim.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("gap check output is not valid JSON: {}\nstdout: {}", e, out.stdout));
    let findings = reports[0]["findings"].as_array().expect("findings should be array");
    assert!(
        findings.iter().any(|f| f["code"].as_str() == Some("G001")),
        "Expected G001 finding in output. Got: {}",
        out.stdout
    );
}

#[test]
fn tc_089_gap_check_resolved() {
    let h = fixture_gap_g001();

    // Step 1: Run gap check to get findings
    let out = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(out.exit_code, 1);
    let reports: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let findings = reports[0]["findings"].as_array().expect("findings");
    let g001_finding = findings.iter().find(|f| f["code"].as_str() == Some("G001")).expect("G001 finding");
    let gap_id = g001_finding["id"].as_str().expect("gap id").to_string();

    // Step 2: Suppress the gap
    let out2 = h.run(&["gap", "suppress", &gap_id, "--reason", "testing resolved"]);
    assert_eq!(out2.exit_code, 0, "suppress should succeed: {}", out2.stderr);

    // Step 3: Fix the gap by adding a linked TC
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test Coverage\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    // Step 4: Run gap check again — gap should not appear in findings
    let out3 = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(out3.exit_code, 0, "Expected exit 0 after fix.\nstdout: {}\nstderr: {}", out3.stdout, out3.stderr);
    let reports3: serde_json::Value = serde_json::from_str(&out3.stdout).expect("valid JSON");
    let findings3 = reports3[0]["findings"].as_array().expect("findings");
    assert!(
        !findings3.iter().any(|f| f["id"].as_str() == Some(gap_id.as_str())),
        "Resolved gap should not appear in findings"
    );

    // Step 5: Verify gaps.json has the resolved entry
    let baseline_content = h.read("gaps.json");
    let baseline: serde_json::Value = serde_json::from_str(&baseline_content)
        .unwrap_or_else(|e| panic!("gaps.json not valid JSON: {}\ncontent: {}", e, baseline_content));
    let resolved = baseline["resolved"].as_array().expect("resolved array");
    assert!(
        resolved.iter().any(|r| r["id"].as_str() == Some(gap_id.as_str())),
        "gaps.json resolved list should contain the fixed gap. Got: {}",
        baseline_content
    );
}

#[test]
fn tc_090_gap_check_changed_scoping() {
    let h = Harness::new();
    git_init(&h);

    // Create fixtures: ADR-002 shares FT-001 with ADR-005. ADR-007 is isolated.
    h.write("docs/features/FT-001-shared.md", "---\nid: FT-001\ntitle: Shared Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002, ADR-005]\ntests: []\n---\n\nShared feature body.\n");
    h.write("docs/features/FT-002-isolated.md", "---\nid: FT-002\ntitle: Isolated Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-007]\ntests: []\n---\n\nIsolated feature body.\n");
    h.write("docs/adrs/ADR-002-test.md", "---\nid: ADR-002\ntitle: ADR Two\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n");
    h.write("docs/adrs/ADR-005-test.md", "---\nid: ADR-005\ntitle: ADR Five\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n");
    h.write("docs/adrs/ADR-007-test.md", "---\nid: ADR-007\ntitle: ADR Seven\nstatus: accepted\nfeatures: [FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n");

    // Initial commit
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

    // Modify ADR-002
    h.write("docs/adrs/ADR-002-test.md", "---\nid: ADR-002\ntitle: ADR Two Updated\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\nUpdated content.\n");
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "modify ADR-002"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    // Run --changed
    let out = h.run(&["gap", "check", "--changed"]);
    assert_eq!(out.exit_code, 0, "Expected exit 0.\nstdout: {}\nstderr: {}", out.stdout, out.stderr);

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("gap check --changed output not valid JSON: {}\nstdout: {}", e, out.stdout));
    let report_arr = reports.as_array().expect("reports should be array");

    // ADR-002 and ADR-005 (1-hop neighbour) should be analysed
    let analysed_adrs: Vec<&str> = report_arr.iter().filter_map(|r| r["adr"].as_str()).collect();
    assert!(
        analysed_adrs.contains(&"ADR-002"),
        "ADR-002 (changed) should be analysed. Got: {:?}",
        analysed_adrs
    );
    assert!(
        analysed_adrs.contains(&"ADR-005"),
        "ADR-005 (1-hop neighbour) should be analysed. Got: {:?}",
        analysed_adrs
    );
    // ADR-007 (no shared features) should NOT be analysed
    assert!(
        !analysed_adrs.contains(&"ADR-007"),
        "ADR-007 (isolated) should NOT be analysed. Got: {:?}",
        analysed_adrs
    );
}

#[test]
fn tc_091_gap_check_model_error_exits_2() {
    let h = fixture_gap_clean();
    let out = h.run_with_env(
        &["gap", "check", "ADR-001"],
        &[("PRODUCT_GAP_INJECT_ERROR", "simulated network failure")],
    );
    assert_eq!(
        out.exit_code, 0,
        "Under FT-045 the gap check is structural only and never exits 2 for a removed LLM path.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );
    assert!(
        !out.stderr.contains("model failure"),
        "Under FT-045 there is no LLM model call; stderr must not reference 'model failure'. Got: {}",
        out.stderr
    );
}

#[test]
fn tc_092_gap_check_invalid_json_discarded() {
    let h = fixture_gap_clean();

    // Inject a response with one valid and one malformed finding — FT-045
    // requires these to be fully ignored.
    let mock_response = r#"[
        {
            "id": "GAP-ADR-001-G004-abcd",
            "code": "G004",
            "severity": "medium",
            "description": "Undocumented constraint found",
            "affected_artifacts": ["ADR-001"],
            "suggested_action": "Document the constraint"
        },
        {
            "id": "GAP-ADR-001-G005-bad",
            "code": "G005",
            "severity": "invalid_severity"
        }
    ]"#;

    let out = h.run_with_env(
        &["gap", "check", "ADR-001"],
        &[("PRODUCT_GAP_INJECT_RESPONSE", mock_response)],
    );

    assert_eq!(
        out.exit_code, 0,
        "Expected exit 0.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("output not valid JSON: {}\nstdout: {}", e, out.stdout));

    // Injected findings must NOT appear — Product no longer invokes an LLM.
    for report in reports.as_array().expect("reports array") {
        for finding in report["findings"].as_array().expect("findings array") {
            assert_ne!(
                finding["id"].as_str(),
                Some("GAP-ADR-001-G004-abcd"),
                "Injected model finding must be absent under FT-045"
            );
        }
    }
}

#[test]
fn tc_095_gap_changed_expansion() {
    let h = Harness::new();
    git_init(&h);

    // FT-001 links ADR-002 and ADR-005
    h.write("docs/features/FT-001-shared.md", "---\nid: FT-001\ntitle: Shared\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002, ADR-005]\ntests: []\n---\n\nBody.\n");
    h.write("docs/adrs/ADR-002-two.md", "---\nid: ADR-002\ntitle: Two\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n");
    h.write("docs/adrs/ADR-005-five.md", "---\nid: ADR-005\ntitle: Five\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n");

    // Initial commit
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

    // Modify ADR-002
    h.write("docs/adrs/ADR-002-two.md", "---\nid: ADR-002\ntitle: Two Updated\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\nUpdated.\n");
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "update ADR-002"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    let out = h.run(&["gap", "check", "--changed"]);
    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("not valid JSON: {}\nstdout: {}", e, out.stdout));
    let report_arr = reports.as_array().expect("reports array");
    let analysed_adrs: Vec<&str> = report_arr.iter().filter_map(|r| r["adr"].as_str()).collect();

    assert!(
        analysed_adrs.contains(&"ADR-005"),
        "ADR-005 should be included via 1-hop expansion. Analysed: {:?}",
        analysed_adrs
    );
}

#[test]
fn tc_097_gap_stdout_stderr_separation() {
    // Test 1: normal run — stdout is valid JSON
    let h = fixture_gap_g001();
    let out = h.run(&["gap", "check", "ADR-001"]);
    // stdout should be valid JSON regardless of exit code
    let _parsed: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {}\nstdout: {}", e, out.stdout));

    // Test 2: under FT-045 / ADR-040 there is no LLM path. Injected env vars
    // are ignored — stdout stays valid JSON and there is no model error.
    let h2 = fixture_gap_clean();
    let out2 = h2.run_with_env(
        &["gap", "check", "ADR-001"],
        &[("PRODUCT_GAP_INJECT_ERROR", "test error")],
    );
    assert_eq!(out2.exit_code, 0);
    let _parsed2: serde_json::Value = serde_json::from_str(&out2.stdout)
        .unwrap_or_else(|e| panic!("stdout should be valid JSON: {}\nstdout: {}", e, out2.stdout));
    assert!(
        !out2.stderr.contains("model failure"),
        "Under FT-045 there is no LLM model call. Got stderr: {}",
        out2.stderr
    );
}

#[test]
fn tc_098_gap_json_schema() {
    let h = fixture_gap_g001();
    let out = h.run(&["gap", "check", "ADR-001"]);

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout not valid JSON: {}\nstdout: {}", e, out.stdout));

    let required_fields = ["id", "code", "severity", "description", "affected_artifacts", "suggested_action"];

    for report in reports.as_array().expect("reports array") {
        for finding in report["findings"].as_array().expect("findings array") {
            for field in &required_fields {
                assert!(
                    !finding[field].is_null(),
                    "Finding missing required field '{}': {}",
                    field,
                    finding
                );
            }
            // Verify types
            assert!(finding["id"].is_string(), "id should be string");
            assert!(finding["code"].is_string(), "code should be string");
            assert!(finding["severity"].is_string(), "severity should be string");
            assert!(finding["description"].is_string(), "description should be string");
            assert!(finding["affected_artifacts"].is_array(), "affected_artifacts should be array");
            assert!(finding["suggested_action"].is_string(), "suggested_action should be string");
        }
    }
}

#[test]
fn tc_087_gap_check_no_gaps() {
    let h = fixture_gap_clean();
    let out = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(
        out.exit_code, 0,
        "Expected exit 0 for ADR with full coverage.\nstdout: {}\nstderr: {}",
        out.stdout, out.stderr
    );

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("gap check output is not valid JSON: {}\nstdout: {}", e, out.stdout));
    let findings = reports[0]["findings"].as_array().expect("findings should be array");
    assert!(
        findings.is_empty(),
        "Expected empty findings array for clean ADR. Got: {}",
        out.stdout
    );
}

#[test]
fn tc_088_gap_check_suppressed() {
    let h = fixture_gap_g001();

    // Step 1: Run gap check to get findings
    let out = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(out.exit_code, 1, "Expected exit 1 initially.\nstdout: {}\nstderr: {}", out.stdout, out.stderr);
    let reports: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let findings = reports[0]["findings"].as_array().expect("findings");
    let g001_finding = findings.iter().find(|f| f["code"].as_str() == Some("G001")).expect("G001 finding");
    let gap_id = g001_finding["id"].as_str().expect("gap id").to_string();

    // Step 2: Suppress the gap
    let out2 = h.run(&["gap", "suppress", &gap_id, "--reason", "deferred to phase 2"]);
    assert_eq!(out2.exit_code, 0, "suppress should succeed: {}", out2.stderr);

    // Step 3: Run gap check again — should exit 0 and finding should be suppressed
    let out3 = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(
        out3.exit_code, 0,
        "Expected exit 0 after suppression.\nstdout: {}\nstderr: {}",
        out3.stdout, out3.stderr
    );
    let reports3: serde_json::Value = serde_json::from_str(&out3.stdout).expect("valid JSON");
    let findings3 = reports3[0]["findings"].as_array().expect("findings");
    let suppressed_finding = findings3.iter().find(|f| f["id"].as_str() == Some(gap_id.as_str()));
    assert!(
        suppressed_finding.is_some(),
        "Suppressed finding should still appear in output. Got: {}",
        out3.stdout
    );
    assert_eq!(
        suppressed_finding.expect("finding")["suppressed"].as_bool(),
        Some(true),
        "Finding should have suppressed=true. Got: {}",
        out3.stdout
    );
}

#[test]
fn tc_093_gap_id_deterministic() {
    let h = fixture_gap_g001();

    // Run gap analysis twice
    let out1 = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(out1.exit_code, 1);
    let reports1: serde_json::Value = serde_json::from_str(&out1.stdout).expect("valid JSON run 1");
    let findings1 = reports1[0]["findings"].as_array().expect("findings run 1");

    let out2 = h.run(&["gap", "check", "ADR-001"]);
    assert_eq!(out2.exit_code, 1);
    let reports2: serde_json::Value = serde_json::from_str(&out2.stdout).expect("valid JSON run 2");
    let findings2 = reports2[0]["findings"].as_array().expect("findings run 2");

    // All high-severity findings should have identical IDs between runs
    let high1: Vec<&str> = findings1
        .iter()
        .filter(|f| f["severity"].as_str() == Some("high"))
        .filter_map(|f| f["id"].as_str())
        .collect();
    let high2: Vec<&str> = findings2
        .iter()
        .filter(|f| f["severity"].as_str() == Some("high"))
        .filter_map(|f| f["id"].as_str())
        .collect();

    assert!(!high1.is_empty(), "Expected at least one high-severity finding");
    assert_eq!(
        high1, high2,
        "High-severity finding IDs should be identical between runs.\nRun 1: {:?}\nRun 2: {:?}",
        high1, high2
    );
}

#[test]
fn tc_094_gap_suppress_mutates_baseline() {
    let h = fixture_gap_clean();
    git_init(&h);

    // Make an initial commit so git rev-parse works
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

    let gap_id = "GAP-ADR002-G001-a3f9";
    let out = h.run(&["gap", "suppress", gap_id, "--reason", "deferred"]);
    assert_eq!(out.exit_code, 0, "suppress should succeed: {}", out.stderr);

    // Read and verify gaps.json
    let baseline_content = h.read("gaps.json");
    assert!(!baseline_content.is_empty(), "gaps.json should exist after suppress");

    let baseline: serde_json::Value = serde_json::from_str(&baseline_content)
        .unwrap_or_else(|e| panic!("gaps.json not valid JSON: {}\ncontent: {}", e, baseline_content));

    let suppressions = baseline["suppressions"].as_array().expect("suppressions array");
    let entry = suppressions
        .iter()
        .find(|s| s["id"].as_str() == Some(gap_id))
        .expect("suppression entry for gap ID should exist");

    // Verify reason
    assert_eq!(
        entry["reason"].as_str(),
        Some("deferred"),
        "Reason should match. Got: {}",
        entry
    );

    // Verify timestamp exists and is non-empty
    let suppressed_at = entry["suppressed_at"].as_str().expect("suppressed_at field");
    assert!(!suppressed_at.is_empty(), "suppressed_at should be non-empty");

    // Verify commit hash exists and starts with "git:"
    let suppressed_by = entry["suppressed_by"].as_str().expect("suppressed_by field");
    assert!(
        suppressed_by.starts_with("git:"),
        "suppressed_by should start with 'git:'. Got: {}",
        suppressed_by
    );
}

#[test]
fn tc_096_gap_id_format() {
    let h = fixture_gap_g001();
    let out = h.run(&["gap", "check", "ADR-001"]);

    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout not valid JSON: {}\nstdout: {}", e, out.stdout));

    let re = regex::Regex::new(r"^GAP-[A-Z]+-[0-9]+-[A-Z0-9]+-[a-f0-9]{4,8}$").expect("valid regex");

    for report in reports.as_array().expect("reports array") {
        for finding in report["findings"].as_array().expect("findings array") {
            let id = finding["id"].as_str().expect("finding id should be string");
            assert!(
                re.is_match(id),
                "Gap ID '{}' does not match expected format GAP-[A-Z]+-[A-Z0-9]+-[A-Z0-9]{{4,8}}",
                id
            );
        }
    }
}

