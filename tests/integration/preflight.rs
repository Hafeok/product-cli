//! Integration tests — preflight.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_140_preflight_clean_exits_0() {
    let h = harness_with_domains();

    // Cross-cutting ADR linked by FT-001
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nError model.\n");

    // Domain ADR for security, linked by FT-001
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");

    // Feature that links all cross-cutting and domain ADRs
    h.write("docs/features/FT-001-cluster.md",
        "---\nid: FT-001\ntitle: Cluster\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-013, ADR-020]\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nCluster feature.\n");

    let out = h.run(&["preflight", "FT-001"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("CLEAN"),
        "Preflight should print 'CLEAN' when all coverage present, got:\n{}",
        out.stdout
    );
}

#[test]
fn tc_141_preflight_cross_cutting_gap() {
    let h = harness_with_domains();

    // Cross-cutting ADR NOT linked by FT-009
    h.write("docs/adrs/ADR-038-observability.md",
        "---\nid: ADR-038\ntitle: Observability Requirements\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [networking]\nscope: cross-cutting\n---\n\nObservability.\n");

    // Feature with no ADR links
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    let out = h.run(&["preflight", "FT-009"]);
    assert_eq!(out.exit_code, 1, "Preflight should exit 1 with gaps, got {}", out.exit_code);
    assert!(
        out.stdout.contains("ADR-038"),
        "Preflight should name ADR-038 in the report, got:\n{}",
        out.stdout
    );
}

#[test]
fn tc_142_preflight_domain_gap() {
    let h = harness_with_domains();

    // Security domain ADRs (not linked by FT-009)
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");
    h.write("docs/adrs/ADR-021-trust.md",
        "---\nid: ADR-021\ntitle: Trust Boundaries\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nTrust.\n");

    // Feature declares security domain but doesn't link any security ADRs
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    let out = h.run(&["preflight", "FT-009"]);
    assert_eq!(out.exit_code, 1, "Preflight should exit 1 with domain gap");
    // Should report security gap and name top ADRs
    assert!(
        out.stdout.contains("security"),
        "Should report security domain gap, got:\n{}",
        out.stdout
    );
    // Should name at least one of the top security ADRs
    assert!(
        out.stdout.contains("ADR-020") || out.stdout.contains("ADR-021"),
        "Should name top security ADRs by centrality, got:\n{}",
        out.stdout
    );
}

#[test]
fn tc_143_preflight_acknowledgement_closes_gap() {
    let h = harness_with_domains();

    // Security domain ADR
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");

    // Feature with security domain gap
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    // Verify gap exists first
    let out_before = h.run(&["preflight", "FT-009"]);
    assert_eq!(out_before.exit_code, 1, "Should have gap before acknowledge");

    // Acknowledge the domain
    let ack = h.run(&["feature", "acknowledge", "FT-009", "--domain", "security", "--reason", "no trust boundaries"]);
    assert_eq!(ack.exit_code, 0, "Acknowledge should succeed, stderr: {}", ack.stderr);

    // Re-run preflight — gap should be closed
    let out_after = h.run(&["preflight", "FT-009"]);
    out_after.assert_exit(0);
    assert!(
        out_after.stdout.contains("CLEAN"),
        "Preflight should be clean after acknowledgement, got:\n{}",
        out_after.stdout
    );
}

#[test]
fn tc_144_preflight_acknowledgement_without_reason_fails() {
    let h = harness_with_domains();

    // Feature
    let feature_content = "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting.\n";
    h.write("docs/features/FT-009-rate-limiting.md", feature_content);

    // Acknowledge with empty reason
    let out = h.run(&["feature", "acknowledge", "FT-009", "--domain", "security", "--reason", ""]);
    assert!(
        out.exit_code != 0,
        "Acknowledge with empty reason should fail, got exit {}",
        out.exit_code
    );
    assert!(
        out.stderr.contains("E011"),
        "Should produce E011 error, got stderr:\n{}",
        out.stderr
    );

    // Verify front-matter was not mutated: re-read and check domains-acknowledged is still empty
    let after = h.read("docs/features/FT-009-rate-limiting.md");
    assert!(
        after.contains("domains-acknowledged: {}"),
        "Front-matter should not be mutated after failed acknowledge, got:\n{}",
        after
    );
}

#[test]
fn tc_116_pre_commit_hook_installed() {
    let h = Harness::new();
    git_init(&h);

    let out = h.run(&["install-hooks"]);
    out.assert_exit(0);

    let hook_path = h.dir.path().join(".git/hooks/pre-commit");
    assert!(hook_path.exists(), "pre-commit hook should exist");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&hook_path)
            .expect("metadata")
            .permissions();
        assert!(
            perms.mode() & 0o111 != 0,
            "pre-commit hook should be executable, mode={:o}",
            perms.mode()
        );
    }
}

#[test]
fn tc_117_pre_commit_hook_runs_on_staged_adr() {
    let h = Harness::new();
    git_init(&h);

    // Write an ADR missing the "Rejected alternatives" section
    h.write(
        "docs/adrs/ADR-050-incomplete.md",
        "---\nid: ADR-050\ntitle: Incomplete ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** Some context.\n\n**Decision:** Some decision.\n\n**Rationale:** Some rationale.\n\n**Test coverage:** Some tests.\n",
    );

    // Stage the ADR
    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-050-incomplete.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    // Run adr review --staged
    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // The finding should mention the missing section and the file path
    assert!(
        out.stderr.contains("Rejected alternatives"),
        "Should report missing 'Rejected alternatives' section.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("ADR-050") || out.stderr.contains("adrs/"),
        "Should mention the file path.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_118_pre_commit_hook_skips_non_adr() {
    let h = Harness::new();
    git_init(&h);

    // Stage only a feature file (no ADR)
    h.write(
        "docs/features/FT-050-test.md",
        "---\nid: FT-050\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );
    std::process::Command::new("git")
        .args(["add", "docs/features/FT-050-test.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);

    // Should report "No staged ADR files found" — no review warnings
    assert!(
        out.stderr.contains("No staged ADR files"),
        "Should skip review when no ADR files staged.\nstderr: {}",
        out.stderr
    );
    // Should NOT contain structural warnings
    assert!(
        !out.stderr.contains("missing required section"),
        "Should not report structural findings for non-ADR files.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_315_prompts_init_creates_files() {
    let h = Harness::new();

    // Ensure no benchmarks/prompts/ directory exists
    assert!(
        !h.exists("benchmarks/prompts"),
        "benchmarks/prompts/ should not exist before init"
    );

    let out = h.run(&["prompts", "init"]);
    out.assert_exit(0);

    // Assert all four default prompt files exist
    assert!(
        h.exists("benchmarks/prompts/author-feature-v1.md"),
        "author-feature-v1.md should be created"
    );
    assert!(
        h.exists("benchmarks/prompts/author-adr-v1.md"),
        "author-adr-v1.md should be created"
    );
    assert!(
        h.exists("benchmarks/prompts/author-review-v1.md"),
        "author-review-v1.md should be created"
    );
    assert!(
        h.exists("benchmarks/prompts/implement-v1.md"),
        "implement-v1.md should be created"
    );

    // Output should mention created files
    out.assert_stdout_contains("created");
}

#[test]
fn tc_316_prompts_list_output() {
    let h = Harness::new();

    let out = h.run(&["prompts", "list"]);
    out.assert_exit(0);

    // Should list all prompt names
    out.assert_stdout_contains("author-feature");
    out.assert_stdout_contains("author-adr");
    out.assert_stdout_contains("author-review");
    out.assert_stdout_contains("implement");

    // Should include version numbers
    out.assert_stdout_contains("v1");
}

#[test]
fn tc_317_prompts_get_stdout() {
    let h = Harness::new();

    let out = h.run(&["prompts", "get", "author-feature"]);
    out.assert_exit(0);

    // stdout should contain the prompt content
    assert!(
        out.stdout.contains("product_feature_list") || out.stdout.contains("feature"),
        "stdout should contain prompt content.\nstdout: {}",
        out.stdout
    );

    // stderr should be empty (no warnings/errors)
    assert!(
        out.stderr.is_empty(),
        "stderr should be empty.\nstderr: {}",
        out.stderr
    );
}

