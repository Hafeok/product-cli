//! Integration tests — verify.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_145_implement_blocked_by_preflight() {
    let h = harness_with_domains();

    // Cross-cutting ADR not linked by FT-009
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nCross-cutting error model.\n");

    // Feature with gaps: no link to cross-cutting ADR-013
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nRate limiting feature.\n");

    let out = h.run(&["implement", "FT-009", "--dry-run"]);
    assert!(
        out.exit_code != 0,
        "implement should fail when preflight has gaps, got exit {}",
        out.exit_code
    );
    assert!(
        out.stderr.contains("preflight") || out.stderr.contains("Pre-flight") || out.stderr.contains("BLOCKED"),
        "Should mention preflight in error, got stderr:\n{}",
        out.stderr
    );
    // No agent should have been invoked (no Step 3/4 output)
    assert!(
        !out.stdout.contains("Step 3") && !out.stdout.contains("Step 4"),
        "Agent should not be invoked when preflight blocks, got stdout:\n{}",
        out.stdout
    );
}

#[test]
fn tc_108_implement_gap_gate_blocks() {
    let h = fixture_implement_gap();
    let out = h.run(&["implement", "FT-001", "--dry-run"]);
    // Should exit 1 due to gap gate
    out.assert_exit(1);
    out.assert_stderr_contains("E009");
    out.assert_stderr_contains("implementation blocked by specification gaps");
    out.assert_stderr_contains("gap[G001]");
}

#[test]
fn tc_109_implement_gap_gate_suppressed() {
    let h = fixture_implement_gap();

    // First, get the gap ID by running gap check
    let out = h.run(&["gap", "check", "ADR-001"]);
    let reports: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("gap check output not valid JSON: {}\nstdout: {}", e, out.stdout));
    let findings = reports[0]["findings"].as_array().expect("findings array");
    let g001_finding = findings.iter().find(|f| f["code"].as_str() == Some("G001"))
        .expect("G001 finding should exist");
    let gap_id = g001_finding["id"].as_str().expect("gap id").to_string();

    // Suppress the gap
    let suppress_out = h.run(&["gap", "suppress", &gap_id, "--reason", "testing suppression"]);
    assert_eq!(suppress_out.exit_code, 0, "suppress should succeed: {}", suppress_out.stderr);

    // Now implement --dry-run should get past the gap gate
    let out2 = h.run(&["implement", "FT-001", "--dry-run"]);
    // Should succeed (dry-run stops at step 3, not blocked by gaps)
    out2.assert_exit(0);
    out2.assert_stdout_contains("Gap gate");
    out2.assert_stdout_contains("OK");
    out2.assert_stdout_contains("dry-run");
}

#[test]
fn tc_110_implement_dry_run() {
    let h = fixture_gap_clean();
    let out = h.run(&["implement", "FT-001", "--dry-run"]);
    out.assert_exit(0);
    // Should print context file path
    out.assert_stdout_contains("Context file:");
    out.assert_stdout_contains("product-impl-FT-001");
    // Should indicate dry-run stopped
    out.assert_stdout_contains("dry-run");
    // The context file path should be a temp file
    // Extract path from output and verify it exists
    let path_line = out.stdout.lines()
        .find(|l| l.contains("Context file:"))
        .expect("should have context file line");
    let path_str = path_line.split("Context file:").nth(1).expect("path after colon").trim();
    assert!(
        std::path::Path::new(path_str).exists(),
        "Context temp file should exist at: {}",
        path_str
    );
}

#[test]
fn tc_111_verify_all_pass_completes_feature() {
    let h = fixture_verify_passing();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");

    // Check feature status is now complete
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: complete"),
        "Feature should be marked complete.\nContent: {}",
        feature_content
    );

    // Check TC statuses are passing
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(tc1.contains("status: passing"), "TC-001 should be passing.\nContent: {}", tc1);
    let tc2 = h.read("docs/tests/TC-002-test.md");
    assert!(tc2.contains("status: passing"), "TC-002 should be passing.\nContent: {}", tc2);
}

#[test]
fn tc_112_verify_one_fail_keeps_in_progress() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Pass Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Fail Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("fail.sh", "#!/bin/bash\necho 'assertion failed' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "fail.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");
    out.assert_stdout_contains("FAIL");

    // Feature should stay in-progress
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: in-progress"),
        "Feature should remain in-progress when a TC fails.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_113_verify_unimplemented_blocks() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body with no runner.\n",
    );

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("UNIMPLEMENTED");

    // Feature status should be in-progress (unimplemented TCs block completion)
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: in-progress"),
        "Feature should be in-progress when TCs are unimplemented.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_113b_verify_unrunnable_no_block() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unrunnable\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body acknowledged as unrunnable.\n",
    );

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("UNRUNNABLE");

    // Should emit W016 warning for unrunnable TCs
    out.assert_stderr_contains("warning[W016]");
}

#[test]
fn tc_114_verify_updates_frontmatter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Pass Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Fail Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("fail.sh", "#!/bin/bash\necho 'assertion failed: expected 42' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "fail.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // TC-001 (passing) should have last-run
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("last-run:"),
        "Passing TC should have last-run timestamp.\nContent: {}",
        tc1
    );

    // TC-002 (failing) should have last-run and failure-message
    let tc2 = h.read("docs/tests/TC-002-test.md");
    assert!(
        tc2.contains("last-run:"),
        "Failing TC should have last-run timestamp.\nContent: {}",
        tc2
    );
    assert!(
        tc2.contains("failure-message:"),
        "Failing TC should have failure-message.\nContent: {}",
        tc2
    );
}

#[test]
fn tc_115_verify_regenerates_checklist() {
    let h = fixture_verify_passing();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // Checklist should exist and contain the feature
    assert!(h.exists("docs/checklist.md"), "checklist.md should be generated");
    let checklist = h.read("docs/checklist.md");
    assert!(
        checklist.contains("FT-001"),
        "Checklist should contain FT-001.\nContent: {}",
        checklist
    );
    // Feature should be marked complete with [x]
    assert!(
        checklist.contains("[x]") && checklist.contains("FT-001"),
        "Checklist should show FT-001 as complete.\nContent: {}",
        checklist
    );
}

#[test]
fn tc_304_verify_one_fail_in_progress() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Pass Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Fail Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("fail.sh", "#!/bin/bash\necho 'test assertion failed' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "fail.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");
    out.assert_stdout_contains("FAIL");

    // Feature should stay in-progress
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: in-progress"),
        "Feature should remain in-progress when a TC fails.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_305_verify_unimplemented_no_runner_blocks() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body with no runner configured.\n",
    );

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("UNIMPLEMENTED");

    // Feature status should be in-progress (unimplemented TCs block completion)
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: in-progress"),
        "Feature should be in-progress when TCs have no runner.\nContent: {}",
        feature_content
    );
}

#[test]
fn tc_306_verify_updates_tc_frontmatter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Pass Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Fail Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("fail.sh", "#!/bin/bash\necho 'expected 42 got 0' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "fail.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // TC-001 (passing) should have last-run and last-run-duration
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("last-run:"),
        "Passing TC should have last-run timestamp.\nContent: {}",
        tc1
    );
    assert!(
        tc1.contains("last-run-duration:"),
        "Passing TC should have last-run-duration.\nContent: {}",
        tc1
    );

    // TC-002 (failing) should have last-run and last-run-duration
    let tc2 = h.read("docs/tests/TC-002-test.md");
    assert!(
        tc2.contains("last-run:"),
        "Failing TC should have last-run timestamp.\nContent: {}",
        tc2
    );
    assert!(
        tc2.contains("last-run-duration:"),
        "Failing TC should have last-run-duration.\nContent: {}",
        tc2
    );
}

#[test]
fn tc_307_verify_failure_message_written() {
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
        "---\nid: TC-001\ntitle: Fail Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h.write("fail.sh", "#!/bin/bash\necho 'thread panicked at assertion failed: expected 42' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "fail.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FAIL");

    // TC should have failure-message with test output
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("failure-message:"),
        "Failing TC should have failure-message.\nContent: {}",
        tc1
    );
    assert!(
        tc1.contains("assertion failed"),
        "failure-message should contain test output.\nContent: {}",
        tc1
    );
}

#[test]
fn tc_309_verify_platform_runs_cross_cutting() {
    let h = Harness::new();
    // Feature-specific ADR with a TC — should NOT be run by --platform
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-feature.md",
        "---\nid: ADR-001\ntitle: Feature ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\nscope: feature-specific\n---\n\nFeature-specific decision.\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Feature Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nFeature test — should NOT run under --platform.\n",
    );

    // Cross-cutting ADR with a TC — should be run by --platform
    h.write(
        "docs/adrs/ADR-002-cross.md",
        "---\nid: ADR-002\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\nscope: cross-cutting\n---\n\nCross-cutting ADR.\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Cross-Cutting Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: []\n  adrs: [ADR-002]\nphase: 1\nrunner: bash\nrunner-args: cross_pass.sh\n---\n\nCross-cutting test — should run under --platform.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("cross_pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "cross_pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "--platform"]);
    out.assert_exit(0);

    // Cross-cutting TC should have been run and marked passing
    let tc2 = h.read("docs/tests/TC-002-test.md");
    assert!(
        tc2.contains("status: passing"),
        "Cross-cutting TC should be marked passing.\nContent: {}",
        tc2
    );

    // Feature-specific TC should NOT have been run (status unchanged)
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("status: unimplemented"),
        "Feature-specific TC should NOT be run by --platform.\nContent: {}",
        tc1
    );
}

#[test]
fn tc_310_verify_requires_satisfied() {
    let h = Harness::new();
    // Override product.toml with prerequisites
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
[verify.prerequisites]
binary-compiled = "true"
"#,
    );
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
        "---\nid: TC-001\ntitle: Test With Prereq\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\nrequires: [binary-compiled]\n---\n\nTest with satisfied prerequisite.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");

    // TC should be passing (prerequisite was satisfied, test ran)
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("status: passing"),
        "TC with satisfied prereq should pass.\nContent: {}",
        tc1
    );
}

#[test]
fn tc_311_verify_requires_not_satisfied() {
    let h = Harness::new();
    // Override product.toml with prerequisite that fails
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
[verify.prerequisites]
two-node-cluster = "false"
"#,
    );
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
        "---\nid: TC-001\ntitle: Cluster Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\nrequires: [two-node-cluster]\n---\n\nTest requiring cluster.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("UNRUNNABLE");

    // TC should become unrunnable with failure-message containing the prereq name
    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("status: unrunnable"),
        "TC with unsatisfied prereq should be unrunnable.\nContent: {}",
        tc1
    );
    assert!(
        tc1.contains("two-node-cluster"),
        "failure-message should contain prerequisite name.\nContent: {}",
        tc1
    );

    // Feature status should remain unchanged (in-progress) — unrunnable doesn't change status
    // Since no runnable TCs and no unimplemented TCs, the W001 warning fires and status is unchanged
    let feature = h.read("docs/features/FT-001-test.md");
    assert!(
        feature.contains("status: in-progress"),
        "Feature should remain in-progress when all TCs are unrunnable.\nContent: {}",
        feature
    );
}

#[test]
fn tc_312_verify_requires_missing_prereq_def() {
    let h = Harness::new();
    // No [verify.prerequisites] section — prerequisite not defined
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
        "---\nid: TC-001\ntitle: Cluster Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\nrequires: [nonexistent-prereq]\n---\n\nTest requiring undefined prereq.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E011");
    out.assert_stderr_contains("nonexistent-prereq");
    out.assert_stderr_contains("[verify.prerequisites]");
}

#[test]
fn tc_313_verify_wrapper_script() {
    // Test 1: Script exits 0 → TC passing
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
        "---\nid: TC-001\ntitle: Wrapper Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: scripts/test-harness/raft.sh\n---\n\nWrapper script test.\n",
    );
    std::fs::create_dir_all(h.dir.path().join("scripts/test-harness")).expect("mkdir");
    h.write("scripts/test-harness/raft.sh", "#!/usr/bin/env bash\nset -euo pipefail\n# Setup, test, teardown — entirely this script's responsibility.\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "scripts/test-harness/raft.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");

    let tc1 = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc1.contains("status: passing"),
        "Wrapper script exiting 0 should set TC to passing.\nContent: {}",
        tc1
    );

    // Test 2: Script exits 1 → TC failing
    let h2 = Harness::new();
    h2.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h2.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h2.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Wrapper Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: scripts/test-harness/raft.sh\n---\n\nWrapper script test.\n",
    );
    std::fs::create_dir_all(h2.dir.path().join("scripts/test-harness")).expect("mkdir");
    h2.write("scripts/test-harness/raft.sh", "#!/usr/bin/env bash\nset -euo pipefail\necho 'raft election timeout' >&2\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "scripts/test-harness/raft.sh"])
        .current_dir(h2.dir.path())
        .output()
        .expect("chmod");

    let out2 = h2.run(&["verify", "FT-001"]);
    out2.assert_exit(0);
    out2.assert_stdout_contains("FAIL");

    let tc1_fail = h2.read("docs/tests/TC-001-test.md");
    assert!(
        tc1_fail.contains("status: failing"),
        "Wrapper script exiting 1 should set TC to failing.\nContent: {}",
        tc1_fail
    );
}

#[test]
fn tc_440_verify_exits_e016_when_linked_adr_is_proposed() {
    let h = fixture_lifecycle_gate_proposed();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E016");
    out.assert_stderr_contains("ADR-001");
    out.assert_stderr_contains("proposed");

    // Feature status should be unchanged (still in-progress, not promoted)
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: in-progress"),
        "Feature should remain in-progress when E016 blocks.\nContent: {}",
        feature_content
    );

    // TC should not have been executed (no status update, no last-run)
    let tc_content = h.read("docs/tests/TC-001-test.md");
    assert!(
        !tc_content.contains("status: passing"),
        "TC should not have been executed.\nContent: {}",
        tc_content
    );
    assert!(
        !tc_content.contains("last-run:"),
        "TC should not have last-run timestamp.\nContent: {}",
        tc_content
    );
}

#[test]
fn tc_441_verify_succeeds_when_all_linked_adrs_are_accepted() {
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

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    // No E016 in stderr
    assert!(
        !out.stderr.contains("E016"),
        "Should not contain E016 when ADR is accepted.\nStderr: {}",
        out.stderr
    );

    // Feature should be complete
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(
        feature_content.contains("status: complete"),
        "Feature should be marked complete.\nContent: {}",
        feature_content
    );

    // TC should be passing with last-run
    let tc_content = h.read("docs/tests/TC-001-test.md");
    assert!(
        tc_content.contains("status: passing"),
        "TC should be passing.\nContent: {}",
        tc_content
    );
    assert!(
        tc_content.contains("last-run:"),
        "TC should have last-run timestamp.\nContent: {}",
        tc_content
    );
}

#[test]
fn tc_448_verify_creates_completion_tag() {
    let h = fixture_verify_with_git();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("PASS");

    // Feature should be complete
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(feature_content.contains("status: complete"), "Feature should be complete.\nContent: {}", feature_content);

    // Tag should exist
    let tag_out = std::process::Command::new("git")
        .args(["tag", "-l", "product/FT-001/complete"])
        .current_dir(h.dir.path())
        .output()
        .expect("git tag -l");
    let tag_stdout = String::from_utf8_lossy(&tag_out.stdout);
    assert!(tag_stdout.contains("product/FT-001/complete"), "Tag should exist.\nTag output: {}", tag_stdout);

    // Tag should be annotated (has a message)
    let msg_out = std::process::Command::new("git")
        .args(["tag", "-l", "product/FT-001/complete", "--format=%(contents)"])
        .current_dir(h.dir.path())
        .output()
        .expect("git tag message");
    let msg = String::from_utf8_lossy(&msg_out.stdout);
    assert!(msg.contains("FT-001 complete"), "Tag message should contain 'FT-001 complete'.\nMessage: {}", msg);
    assert!(msg.contains("TC-001"), "Tag message should list TC IDs.\nMessage: {}", msg);
    assert!(msg.contains("TC-002"), "Tag message should list TC IDs.\nMessage: {}", msg);

    // Stdout should mention the tag
    out.assert_stdout_contains("Tagged: product/FT-001/complete");
    out.assert_stdout_contains("git push --tags");
}

#[test]
fn tc_449_verify_tag_version_increments() {
    let h = fixture_verify_with_git();

    // First verify → complete tag
    let out1 = h.run(&["verify", "FT-001"]);
    out1.assert_exit(0);
    out1.assert_stdout_contains("Tagged: product/FT-001/complete");

    // Reset feature to in-progress for re-verification
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    git_add_commit(&h, "reset feature status");

    // Second verify → complete-v2
    let out2 = h.run(&["verify", "FT-001"]);
    out2.assert_exit(0);
    out2.assert_stdout_contains("Tagged: product/FT-001/complete-v2");

    // Both tags should exist
    let tag_out = std::process::Command::new("git")
        .args(["tag", "-l", "product/FT-001/*"])
        .current_dir(h.dir.path())
        .output()
        .expect("git tag -l");
    let tags = String::from_utf8_lossy(&tag_out.stdout);
    assert!(tags.contains("product/FT-001/complete"), "Original tag should exist");
    assert!(tags.contains("product/FT-001/complete-v2"), "v2 tag should exist");
}

#[test]
fn tc_450_verify_skips_tag_outside_git() {
    // Use standard fixture (no git init)
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

    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // Feature completes normally
    let feature_content = h.read("docs/features/FT-001-test.md");
    assert!(feature_content.contains("status: complete"), "Feature should be complete");

    // W018 warning about not being a git repo
    out.assert_stderr_contains("W018");
    out.assert_stderr_contains("not a git repository");
}

#[test]
fn tc_698_implement_pipeline_honors_per_repo_implement_prompt() {
    let h = fixture_gap_clean();

    // --- Override path -------------------------------------------------
    let sentinel = "# CUSTOM IMPLEMENT PROMPT — sentinel-9f3b2a";
    h.write("benchmarks/prompts/implement-v1.md", sentinel);

    let out = h.run(&["implement", "FT-001", "--dry-run"]);
    out.assert_exit(0);
    out.assert_stdout_contains("Context file:");

    let path_line = out
        .stdout
        .lines()
        .find(|l| l.contains("Context file:"))
        .expect("should have context file line");
    let path_str = path_line
        .split("Context file:")
        .nth(1)
        .expect("path after colon")
        .trim();
    let content =
        std::fs::read_to_string(path_str).expect("context file should be readable");

    assert!(
        content.starts_with(sentinel),
        "Override prompt should appear at top of context file.\nfile starts with:\n{}",
        &content[..content.len().min(200)]
    );
    // Dynamic suffix is appended below the sentinel.
    assert!(
        content.contains("# Implementation Task: FT-001"),
        "Dynamic suffix should include the feature header. file:\n{}",
        content
    );
    assert!(
        content.contains("## Current test status"),
        "Dynamic suffix should include the TC status table. file:\n{}",
        content
    );
    assert!(
        content.contains("product verify FT-001"),
        "Dynamic suffix should include the verify hard constraint. file:\n{}",
        content
    );
    assert!(
        content.contains("## Context Bundle"),
        "Dynamic suffix should include the context bundle. file:\n{}",
        content
    );

    // --- Fallback path -------------------------------------------------
    std::fs::remove_file(h.dir.path().join("benchmarks/prompts/implement-v1.md"))
        .expect("remove override");

    let out2 = h.run(&["implement", "FT-001", "--dry-run"]);
    out2.assert_exit(0);

    let path_line2 = out2
        .stdout
        .lines()
        .find(|l| l.contains("Context file:"))
        .expect("should have context file line");
    let path_str2 = path_line2
        .split("Context file:")
        .nth(1)
        .expect("path after colon")
        .trim();
    let content2 =
        std::fs::read_to_string(path_str2).expect("context file should be readable");

    // Embedded default begins with the title from src/author/prompts/implement.txt
    assert!(
        content2.starts_with("# Product Implementation Session"),
        "Fallback prompt should use the embedded default body.\nfile starts with:\n{}",
        &content2[..content2.len().min(200)]
    );
    // Dynamic suffix still appended.
    assert!(
        content2.contains("# Implementation Task: FT-001"),
        "Dynamic suffix should still be appended in fallback path."
    );
    assert!(
        content2.contains("product verify FT-001"),
        "Dynamic suffix should still be appended in fallback path."
    );

    // --- Negative case (empty override file) ---------------------------
    h.write("benchmarks/prompts/implement-v1.md", "");

    let out3 = h.run(&["implement", "FT-001", "--dry-run"]);
    out3.assert_exit(0);

    let path_line3 = out3
        .stdout
        .lines()
        .find(|l| l.contains("Context file:"))
        .expect("should have context file line");
    let path_str3 = path_line3
        .split("Context file:")
        .nth(1)
        .expect("path after colon")
        .trim();
    let content3 =
        std::fs::read_to_string(path_str3).expect("context file should be readable");

    // Empty override: file still produced, dynamic suffix still present.
    assert!(
        content3.contains("# Implementation Task: FT-001"),
        "Empty override should still produce the dynamic suffix."
    );
    assert!(
        content3.contains("product verify FT-001"),
        "Empty override should still produce the dynamic suffix."
    );
}

