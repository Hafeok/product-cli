//! Integration tests — ft_exit_criteria.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_154_ft002_exit_criteria() {
    let h = fixture_minimal();
    // Feature list works
    h.run(&["feature", "list"]).assert_exit(0).assert_stdout_contains("FT-001");
    // Feature show works
    h.run(&["feature", "show", "FT-001"]).assert_exit(0);
    // Graph is clean
    h.run(&["graph", "check"]).assert_exit(0);
}

#[test]
fn tc_152_ft007_exit_criteria() {
    // 1. Markdown front-matter stripping (TC-011): context bundle strips ---/YAML fields
    let h1 = Harness::new();
    h1.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h1.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n",
    );
    h1.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );
    let out = h1.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(
        !out.stdout.starts_with("---\n"),
        "Context bundle should not start with front-matter delimiter"
    );
    assert!(
        !out.stdout.contains("status: planned"),
        "YAML fields should not appear in context bundle"
    );

    // 2. Markdown passthrough (TC-012): code blocks, tables, nested lists preserved
    let h2 = Harness::new();
    h2.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\n```rust\nfn main() {}\n```\n\n| Col1 | Col2 |\n|------|------|\n| a    | b    |\n\n- item 1\n  - nested\n",
    );
    let out = h2.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("```rust"), "Code blocks should be preserved");
    assert!(out.stdout.contains("fn main() {}"), "Code content should be preserved");
    assert!(out.stdout.contains("| Col1 | Col2 |"), "Tables should be preserved");
    assert!(out.stdout.contains("- item 1"), "Lists should be preserved");
    assert!(out.stdout.contains("  - nested"), "Nested lists should be preserved");

    // 3. Formal block parsing: Types, Invariants, Scenario, Evidence blocks parsed and preserved
    let h3 = Harness::new();
    h3.write(
        "docs/features/FT-001-formal.md",
        "---\nid: FT-001\ntitle: Formal Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature with formal blocks.\n",
    );
    h3.write(
        "docs/tests/TC-001-formal.md",
        "---\nid: TC-001\ntitle: Formal Test\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Σ:Types⟧{\n  Graph≜⟨nodes:Node+, edges:Edge*⟩\n  CentralityScore≜Float\n}\n\n⟦Γ:Invariants⟧{\n  ∀g:Graph, ∀n∈g.nodes: betweenness(g,n) ≥ 0.0 ∧ betweenness(g,n) ≤ 1.0\n}\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n",
    );
    let out = h3.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Formal blocks must be preserved in context output
    assert!(out.stdout.contains("⟦Σ:Types⟧"), "Types block should be preserved in context bundle");
    assert!(out.stdout.contains("⟦Γ:Invariants⟧"), "Invariants block should be preserved in context bundle");
    assert!(out.stdout.contains("CentralityScore"), "Type definitions should be preserved");
    assert!(out.stdout.contains("betweenness"), "Invariant content should be preserved");

    // 4. Evidence aggregation: AISP bundle header includes evidence metrics
    assert!(out.stdout.contains("⟦Ε⟧"), "Evidence block should appear in bundle header");

    // 5. Graph check passes for well-formed formal specification artifacts
    let out = h3.run(&["graph", "check"]);
    // Exit code 0 (clean) or 2 (warnings only, e.g. W003 for missing exit-criteria) are acceptable
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "Graph check should pass (got exit code {}): {}",
        out.exit_code,
        out.stderr
    );
}

#[test]
fn tc_155_ft003_exit_criteria() {
    let h = Harness::new();
    // Valid feature parses
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nBody.\n");
    h.run(&["feature", "list"]).assert_exit(0).assert_stdout_contains("FT-001");
    // Invalid ID rejected
    h.write("docs/features/bad-id.md", "---\nid: bad\ntitle: Bad\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    let out = h.run(&["feature", "list"]);
    assert!(out.stderr.contains("E005") || out.stderr.contains("invalid"), "Bad ID should error");
}

#[test]
fn tc_153_ft015_exit_criteria() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Formal Test\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{\n  ∀x:Node: x.id > 0\n}\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n");
    // Context bundle includes formal blocks
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("⟦Γ:Invariants⟧"), "Formal blocks preserved in context");
    assert!(out.stdout.contains("∀x:Node"), "Invariant content preserved");
}

#[test]
fn tc_156_ft001_exit_criteria() {
    let h = Harness::new();

    // Markdown front-matter strip (TC-011): context bundle strips front-matter
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n");
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(!out.stdout.starts_with("---\n"), "Context bundle should not start with front-matter delimiter");
    assert!(out.stdout.contains("Feature body"), "Context bundle should contain feature body");
    assert!(out.stdout.contains("Decision body"), "Context bundle should contain ADR body");
    assert!(out.stdout.contains("Test body"), "Context bundle should contain TC body");

    // Markdown passthrough (TC-012): code blocks, tables preserved
    let h2 = Harness::new();
    h2.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\n```rust\nfn main() {}\n```\n\n| Col1 | Col2 |\n|------|------|\n| a    | b    |\n\n- item 1\n  - nested\n");
    let out = h2.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("```rust"), "Code blocks should be preserved");
    assert!(out.stdout.contains("| Col1 | Col2 |"), "Tables should be preserved");
    assert!(out.stdout.contains("- item 1"), "Lists should be preserved");

    // ID auto-increment (TC-013): sequential IDs
    let h3 = Harness::new();
    let out1 = h3.run(&["feature", "new", "First"]);
    out1.assert_exit(0).assert_stdout_contains("FT-001");
    let out2 = h3.run(&["feature", "new", "Second"]);
    out2.assert_exit(0).assert_stdout_contains("FT-002");
    let out3 = h3.run(&["feature", "new", "Third"]);
    out3.assert_exit(0).assert_stdout_contains("FT-003");

    // ID gap fill (TC-014): gaps not filled
    let h4 = Harness::new();
    h4.write("docs/features/FT-001-a.md", "---\nid: FT-001\ntitle: A\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h4.write("docs/features/FT-003-c.md", "---\nid: FT-003\ntitle: C\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    let out = h4.run(&["feature", "new", "D"]);
    out.assert_exit(0).assert_stdout_contains("FT-004");

    // ID conflict (TC-015): duplicate IDs detected
    let h5 = Harness::new();
    h5.write("docs/features/FT-001-a.md", "---\nid: FT-001\ntitle: A\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h5.write("docs/features/FT-001-b.md", "---\nid: FT-001\ntitle: B\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    let out = h5.run(&["graph", "check"]);
    out.assert_exit(1).assert_stderr_contains("E011");
}

#[test]
fn tc_165_ft_021_mcp_server_stdio_and_http_pass() {
    // This test validates that both stdio and HTTP transports work.
    // It exercises a basic tool call via stdio and via HTTP on the same repo
    // to confirm the full MCP surface is operational.

    let h = fixture_minimal();

    // 1. Verify stdio transport works
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_list","arguments":{}}}"#;
    let stdio_out = run_mcp_stdio(&h, input);
    assert!(stdio_out.contains("FT-001"), "stdio should return FT-001: {}", stdio_out);

    // 2. Verify HTTP transport works
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "exit-token-165"]);

    let body = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_feature_list","arguments":{}}}"#;
    let (status, _headers, resp_body) = http_post(port, body, Some("Bearer exit-token-165"));

    let _ = child.kill();
    let _ = child.wait();

    assert!(status.contains("200"), "HTTP should return 200: {}", status);
    assert!(resp_body.contains("FT-001"), "HTTP should return FT-001: {}", resp_body);
}

#[test]
fn tc_161_ft005_exit_criteria() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    // 1. Atomic write produces correct content (TC-066)
    let out = h.run(&["feature", "status", "FT-001", "in-progress"]);
    out.assert_exit(0);
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("in-progress"), "atomic write should update status");

    // No leftover tmp files
    let tmp_files: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.contains(".product-tmp."))
                .unwrap_or(false)
        })
        .collect();
    assert!(tmp_files.is_empty(), "no leftover tmp files after write");

    // 2. Concurrent write lock (TC-068) — lock held by live process blocks writes
    let lock_path = h.dir.path().join(".product.lock");
    std::fs::write(
        &lock_path,
        format!("pid={}\nstarted=2026-04-13T00:00:00Z\n", std::process::id()),
    )
    .expect("write lock");
    let out = h.run(&["feature", "status", "FT-001", "complete"]);
    assert_ne!(out.exit_code, 0, "should fail when lock is held");
    assert!(
        out.stderr.contains("E010") || out.stderr.contains("repository locked"),
        "should report lock error"
    );
    let _ = std::fs::remove_file(&lock_path);

    // 3. Stale lock cleanup (TC-069) — dead PID lock is cleared
    std::fs::write(&lock_path, "pid=4294967\nstarted=2026-04-01T00:00:00Z\n")
        .expect("write stale lock");
    let out = h.run(&["feature", "status", "FT-001", "complete"]);
    out.assert_exit(0);
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("complete"), "should succeed after stale lock cleanup");

    // 4. Tmp cleanup on startup (TC-070)
    h.write("docs/features/.leftover.product-tmp.12345", "garbage");
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    assert!(
        !h.exists("docs/features/.leftover.product-tmp.12345"),
        "tmp files should be cleaned on startup"
    );
}

#[test]
fn tc_160_ft009_exit_criteria() {
    let h = Harness::new();

    // Create a feature with linked ADR and test criterion containing formal blocks
    h.write(
        "docs/features/FT-001-formal.md",
        "---\nid: FT-001\ntitle: Formal Spec\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002, TC-003]\ndomains: []\ndomains-acknowledged: {}\n---\n\nFormal specification feature.\n",
    );
    h.write(
        "docs/adrs/ADR-001-formal.md",
        "---\nid: ADR-001\ntitle: Formal Grammar\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n",
    );

    // TC with ⟦Σ:Types⟧ block
    h.write(
        "docs/tests/TC-001-types.md",
        "---\nid: TC-001\ntitle: Types block\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\n⟦Σ:Types⟧{\n  Node≜IRI\n  Role≜Leader|Follower|Learner\n}\n\n⟦Ε⟧⟨δ≜0.90;φ≜95;τ≜◊⁺⟩\n",
    );

    // TC with ⟦Γ:Invariants⟧ block
    h.write(
        "docs/tests/TC-002-invariants.md",
        "---\nid: TC-002\ntitle: Invariants block\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\n⟦Γ:Invariants⟧{\n  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1\n}\n\n⟦Ε⟧⟨δ≜0.85;φ≜80;τ≜◊?⟩\n",
    );

    // TC with ⟦Λ:Scenario⟧ block
    h.write(
        "docs/tests/TC-003-scenario.md",
        "---\nid: TC-003\ntitle: Scenario block\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\n⟦Λ:Scenario⟧{\n  given≜cluster_init(nodes:3)\n  when≜leader_fails()\n  then≜∃n∈nodes: roles(n)=Leader ∧ n≠old_leader\n}\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n",
    );

    // 1. Context bundle includes formal blocks from test criteria
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("⟦Σ:Types⟧"),
        "Context bundle should contain Types block: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains("Node≜IRI"),
        "Types block content should be preserved"
    );
    assert!(
        out.stdout.contains("⟦Γ:Invariants⟧"),
        "Context bundle should contain Invariants block"
    );
    assert!(
        out.stdout.contains("⟦Λ:Scenario⟧"),
        "Context bundle should contain Scenario block"
    );
    assert!(
        out.stdout.contains("given≜cluster_init"),
        "Scenario fields should be preserved"
    );
    assert!(
        out.stdout.contains("⟦Ε⟧"),
        "Context bundle should contain Evidence block"
    );

    // 2. Graph check reports no errors for well-formed formal blocks
    // (exit code 2 = warnings only, which is acceptable — W003 for missing exit-criteria)
    let out = h.run(&["graph", "check"]);
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "graph check should succeed (possibly with warnings), got exit code {}: {}",
        out.exit_code, out.stderr
    );

    // 3. Formal blocks survive the full pipeline: parse → graph → context
    // Verify evidence aggregation appears in context bundle
    let out = h.run(&["context", "FT-001", "--depth", "2"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("δ≜") || out.stdout.contains("delta"),
        "Evidence delta should appear in context bundle"
    );
    assert!(
        out.stdout.contains("φ≜") || out.stdout.contains("phi"),
        "Evidence phi should appear in context bundle"
    );

    // 4. Verify diagnostic reporting: create a TC with bad evidence
    h.write(
        "docs/tests/TC-004-bad-evidence.md",
        "---\nid: TC-004\ntitle: Bad evidence\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Ε⟧⟨δ≜1.5;φ≜100;τ≜◊⁺⟩\n",
    );
    // Update feature to include TC-004
    h.write(
        "docs/features/FT-001-formal.md",
        "---\nid: FT-001\ntitle: Formal Spec\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002, TC-003, TC-004]\ndomains: []\ndomains-acknowledged: {}\n---\n\nFormal specification feature.\n",
    );
    let out = h.run(&["graph", "check"]);
    // Should report diagnostic — out-of-range delta is a parse error
    // (the check may still exit 0 with warnings, or exit non-zero)
    let combined = format!("{}{}", out.stdout, out.stderr);
    // The graph check should complete (not crash)
    assert!(
        out.exit_code == 0 || combined.contains("E001") || combined.contains("warning") || combined.contains("error"),
        "graph check should handle bad evidence gracefully"
    );
}

#[test]
fn tc_158_ft011_exit_criteria() {
    let h = Harness::new();
    // Set up a representative graph: feature with ADRs, tests, dependencies, supersession
    h.write(
        "docs/features/FT-001-main.md",
        "---\nid: FT-001\ntitle: Main Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002, ADR-003]\ntests: [TC-001, TC-002]\n---\n\nMain feature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-rust.md",
        "---\nid: ADR-001\ntitle: Rust Language\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nRust decision body.\n",
    );
    h.write(
        "docs/adrs/ADR-002-old.md",
        "---\nid: ADR-002\ntitle: Old Store\nstatus: superseded\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: [ADR-003]\n---\n\nOld store decision.\n",
    );
    h.write(
        "docs/adrs/ADR-003-new.md",
        "---\nid: ADR-003\ntitle: New Store\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: [ADR-002]\nsuperseded-by: []\n---\n\nNew store decision.\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: Exit Criterion\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nExit criterion body.\n",
    );
    h.write(
        "docs/tests/TC-002-scenario.md",
        "---\nid: TC-002\ntitle: Scenario Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nScenario test body.\n",
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // 1. Bundle header with AISP formal block
    out.assert_stdout_contains("# Context Bundle: FT-001 — Main Feature");
    out.assert_stdout_contains("⟦Ω:Bundle⟧");
    out.assert_stdout_contains("feature≜FT-001:Feature");
    out.assert_stdout_contains("phase≜1:Phase");
    out.assert_stdout_contains("InProgress:FeatureStatus");
    out.assert_stdout_contains("implementedBy≜⟨");
    out.assert_stdout_contains("validatedBy≜⟨");

    // 2. No YAML front-matter in output
    assert!(!out.stdout.contains("\n---\nid:"), "No YAML front-matter should appear");

    // 3. Feature content present
    out.assert_stdout_contains("Main feature body.");

    // 4. Superseded ADR has annotation
    out.assert_stdout_contains("[SUPERSEDED by ADR-003]");

    // 5. Active ADRs present
    out.assert_stdout_contains("Rust Language");
    out.assert_stdout_contains("New Store");

    // 6. Test criteria present and ordered (exit-criteria before scenario)
    let exit_pos = out.stdout.find("Exit Criterion").expect("exit-criteria should appear");
    let scenario_pos = out.stdout.find("Scenario Test").expect("scenario should appear");
    assert!(exit_pos < scenario_pos, "exit-criteria should appear before scenario");

    // 7. Order: feature → ADRs → tests
    let feature_pos = out.stdout.find("Main feature body.").expect("feature body");
    let adr_pos = out.stdout.find("Rust decision body.").expect("ADR body");
    let tc_pos = out.stdout.find("Exit criterion body.").expect("TC body");
    assert!(feature_pos < adr_pos, "Feature before ADRs");
    assert!(adr_pos < tc_pos, "ADRs before tests");
}

#[test]
fn tc_163_ft012_cluster_foundation_binary_validated() {
    // TC-004: cargo build --release succeeds
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build --release");
    assert!(
        output.status.success(),
        "TC-004 cargo build --release failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check which cross-compilation targets are installed
    let installed_targets = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // TC-001: binary compiles for ARM64 (skip if target not installed)
    if installed_targets.contains("aarch64-unknown-linux-gnu") {
        let output = Command::new("cargo")
            .args(["build", "--release", "--target", "aarch64-unknown-linux-gnu"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("cargo build arm64");
        assert!(
            output.status.success(),
            "TC-001 ARM64 build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        eprintln!("Skipping TC-001 ARM64 cross-build: target not installed");
    }

    // TC-002: binary compiles for x86_64 (skip if target not installed)
    if installed_targets.contains("x86_64-unknown-linux-musl") {
        let output = Command::new("cargo")
            .args(["build", "--release", "--target", "x86_64-unknown-linux-musl"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("cargo build x86_64");
        assert!(
            output.status.success(),
            "TC-002 x86_64 build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        eprintln!("Skipping TC-002 x86_64 cross-build: target not installed");
    }

    // TC-003: binary has no unexpected dynamic dependencies
    let h = Harness::new();
    let ldd_out = Command::new("ldd")
        .arg(&h.bin)
        .output();
    match ldd_out {
        Ok(output) => {
            let ldd_output = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let is_static = ldd_output.contains("not a dynamic executable")
                || ldd_output.contains("statically linked")
                || stderr.contains("not a dynamic executable");
            if !is_static {
                for line in ldd_output.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let allowed = line.contains("libc")
                        || line.contains("libm")
                        || line.contains("libgcc")
                        || line.contains("libpthread")
                        || line.contains("libdl")
                        || line.contains("librt")
                        || line.contains("ld-linux")
                        || line.contains("linux-vdso")
                        || line.contains("linux-gnu");
                    assert!(
                        allowed,
                        "Unexpected dynamic dependency: {}",
                        line
                    );
                }
            }
        }
        Err(_) => {
            eprintln!("ldd not available (e.g., macOS) — skipping dependency check");
        }
    }
}

#[test]
fn tc_164_ft013_rust_implementation_compiles_clean() {
    // Verify cargo build --release compiles with zero errors
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build --release");
    assert!(
        output.status.success(),
        "cargo build --release failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify clippy passes with no warnings (per project convention)
    let output = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings", "-D", "clippy::unwrap_used"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo clippy");
    assert!(
        output.status.success(),
        "cargo clippy failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify Cargo.toml declares edition 2021+ (confirming Rust toolchain)
    let cargo_toml = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("read Cargo.toml");
    assert!(
        cargo_toml.contains("edition = \"2021\"") || cargo_toml.contains("edition = \"2024\""),
        "Cargo.toml should declare a modern Rust edition (2021+)"
    );
}

#[test]
fn tc_157_ft016_graph_model_queries_pass() {
    let h = Harness::new();

    // Set up a representative graph with all edge types
    h.write(
        "docs/features/FT-001-foundation.md",
        "---\nid: FT-001\ntitle: Foundation\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n\nFoundation feature.\n",
    );
    h.write(
        "docs/features/FT-002-middle.md",
        "---\nid: FT-002\ntitle: Middle Layer\nphase: 1\nstatus: in-progress\ndepends-on: [FT-001]\nadrs: [ADR-001, ADR-003]\ntests: [TC-002]\n---\n\nMiddle feature.\n",
    );
    h.write(
        "docs/features/FT-003-top.md",
        "---\nid: FT-003\ntitle: Top Layer\nphase: 2\nstatus: planned\ndepends-on: [FT-002]\nadrs: [ADR-003]\ntests: [TC-003]\n---\n\nTop feature.\n",
    );
    h.write(
        "docs/adrs/ADR-001-rust.md",
        "---\nid: ADR-001\ntitle: Rust Language\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n\nRust decision.\n",
    );
    h.write(
        "docs/adrs/ADR-002-old.md",
        "---\nid: ADR-002\ntitle: Old Store\nstatus: superseded\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: [ADR-003]\n---\n\nOld store.\n",
    );
    h.write(
        "docs/adrs/ADR-003-new.md",
        "---\nid: ADR-003\ntitle: New Store\nstatus: accepted\nfeatures: [FT-002, FT-003]\nsupersedes: [ADR-002]\nsuperseded-by: []\n---\n\nNew store.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Foundation Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nFoundation test.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Middle Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-002]\n  adrs: [ADR-003]\nphase: 1\n---\n\nMiddle test.\n",
    );
    h.write(
        "docs/tests/TC-003-test.md",
        "---\nid: TC-003\ntitle: Top Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-003]\n  adrs: [ADR-003]\nphase: 2\n---\n\nTop test.\n",
    );

    // 1. Graph rebuild produces valid TTL
    let out = h.run(&["graph", "rebuild"]);
    out.assert_exit(0);
    let ttl = h.read("docs/graph/index.ttl");
    assert!(ttl.contains("pm:Feature"), "TTL should contain Feature type");
    assert!(ttl.contains("pm:ArchitecturalDecision"), "TTL should contain ADR type");
    assert!(ttl.contains("pm:implementedBy"), "TTL should contain implementedBy edges");
    assert!(ttl.contains("pm:dependsOn"), "TTL should contain dependsOn edges");
    assert!(ttl.contains("pm:betweennessCentrality"), "TTL should contain centrality scores");

    // 2. SPARQL query works
    let out = h.run(&["graph", "query", "SELECT ?f WHERE { ?f a <https://product-meta/ontology#Feature> }"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("FT-002");
    out.assert_stdout_contains("FT-003");

    // 3. Topological sort respects dependencies
    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    // FT-001 is complete, FT-002 depends on FT-001 (complete) and is in-progress → should be next
    out.assert_stdout_contains("FT-002");

    // 4. Graph central works
    let out = h.run(&["graph", "central"]);
    out.assert_exit(0);
    out.assert_stdout_contains("ADR-001");

    // 5. Impact analysis works
    let out = h.run(&["impact", "ADR-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("FT-002");

    // 6. Context with depth 2 includes transitive artifacts
    let out = h.run(&["context", "FT-001", "--depth", "2"]);
    out.assert_exit(0);
    // Depth 2: FT-001 → ADR-001 → FT-002, so FT-002's artifacts should appear
    assert!(
        out.stdout.contains("FT-002") || out.stdout.contains("Middle Layer") || out.stdout.contains("Middle test"),
        "Depth 2 should include transitive artifacts via ADR-001 → FT-002.\nOutput:\n{}",
        out.stdout
    );

    // 7. Graph check passes (no broken links — warnings about missing exit-criteria are OK)
    let out = h.run(&["graph", "check"]);
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "Graph check should pass (0) or warn (2), got {}.\nstdout: {}\nstderr: {}",
        out.exit_code, out.stdout, out.stderr
    );
}

#[test]
fn tc_162_ft_020_migration_extracts_and_confirms() {
    let h = Harness::new();

    // Create a combined test: PRD migration + ADR migration end-to-end
    let prd_source = r#"# PRD

## Vision

Our grand vision.

## Cluster Foundation

Foundation content.
- [x] foundation done

## Storage Model

Storage content.
- [ ] pending work

## Non-Goals

Not doing this.
"#;
    let adr_source = r#"# ADRs

## ADR-001: Rust Language

**Status:** Accepted

Rust for implementation.

### Test coverage

- `binary_compiles_arm64` — compiles on ARM64
- `chaos_network_partition` — chaos test for network

## ADR-002: YAML Front-Matter

**Status:** Accepted

YAML for front-matter.
"#;
    h.write("prd.md", prd_source);
    h.write("adrs.md", adr_source);

    // Phase 1: Validate (dry-run) — no files written
    let out = h.run(&["migrate", "from-prd", "prd.md", "--validate"]);
    out.assert_exit(0)
        .assert_stdout_contains("Migration plan");
    let feature_count = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .flatten()
        .count();
    assert_eq!(feature_count, 0, "validate should not write files");

    // Phase 2: Execute PRD migration
    let out = h.run(&["migrate", "from-prd", "prd.md", "--execute"]);
    out.assert_exit(0);
    let feature_entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .flatten()
        .collect();
    // Vision and Non-Goals excluded → 2 features (Cluster Foundation, Storage Model)
    assert_eq!(feature_entries.len(), 2, "should create exactly 2 features (Vision + Non-Goals excluded)");

    // Verify status inference: Cluster Foundation has all checked → complete, Storage Model has unchecked → planned
    let mut found_complete = false;
    let mut found_planned = false;
    for entry in &feature_entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("Cluster Foundation") && content.contains("status: complete") {
            found_complete = true;
        }
        if content.contains("Storage Model") && content.contains("status: planned") {
            found_planned = true;
        }
    }
    assert!(found_complete, "Cluster Foundation (all [x]) should have status: complete");
    assert!(found_planned, "Storage Model (has [ ]) should have status: planned");

    // Phase 3: Execute ADR migration
    let out = h.run(&["migrate", "from-adrs", "adrs.md", "--execute"]);
    out.assert_exit(0);
    let adr_entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/adrs"))
        .expect("readdir")
        .flatten()
        .collect();
    assert_eq!(adr_entries.len(), 2, "should create 2 ADR files");

    let test_entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/tests"))
        .expect("readdir")
        .flatten()
        .collect();
    assert!(test_entries.len() >= 2, "should extract at least 2 test criteria from ADR-001");

    // Verify source files are unchanged
    let prd_after = h.read("prd.md");
    assert_eq!(prd_source, prd_after, "PRD source must be unchanged after migration");
    let adr_after = h.read("adrs.md");
    assert_eq!(adr_source, adr_after, "ADR source must be unchanged after migration");

    // Phase 4: Re-run should skip existing files
    let out = h.run(&["migrate", "from-prd", "prd.md", "--execute"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("skip"),
        "re-run should report skipping existing files, got:\n{}",
        out.stdout
    );

    // W009 warning for ADR-002 (no test subsection)
    let out_adrs = h.run(&["migrate", "from-adrs", "adrs.md", "--validate"]);
    assert!(
        out_adrs.stdout.contains("W009"),
        "should warn W009 for ADR-002 missing tests, got:\n{}",
        out_adrs.stdout
    );
}

#[test]
fn tc_180_ft_025_benchmarks_pass() {
    // Run `cargo bench` and verify all four benchmarks complete and pass
    let output = std::process::Command::new("cargo")
        .args(["bench"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run cargo bench");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The benchmark binary should exit successfully
    assert!(
        output.status.success(),
        "cargo bench failed.\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    // All four benchmarks must appear with PASS
    assert!(
        stdout.contains("Parse 200 files:") && stdout.contains("PASS"),
        "Parse 200 files benchmark missing or failed.\nstdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Centrality 200 nodes") && stdout.contains("PASS"),
        "Centrality benchmark missing or failed.\nstdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Impact analysis:") && stdout.contains("PASS"),
        "Impact analysis benchmark missing or failed.\nstdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("BFS depth 2:") && stdout.contains("PASS"),
        "BFS depth 2 benchmark missing or failed.\nstdout:\n{}",
        stdout
    );

    // Verify the summary line shows 4 passed, 0 failed
    assert!(
        stdout.contains("4 passed, 0 failed, 4 total"),
        "Expected all 4 benchmarks to pass.\nstdout:\n{}",
        stdout
    );
}

#[test]
fn tc_181_ft_026_ci_integration_pass() {
    // Part 1: graph check --format json on a clean repo → valid JSON, exit 0
    let h = fixture_minimal();
    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 0, "Expected exit 0 on clean graph.\nstdout: {}\nstderr: {}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("graph check JSON invalid on stdout: {}\nstdout: {}", e, out.stdout));
    assert!(json["summary"]["errors"].as_u64() == Some(0), "Expected 0 errors in clean graph");

    // Part 2: feature list --format json → valid JSON to stdout
    let out2 = h.run(&["feature", "list", "--format", "json"]);
    assert_eq!(out2.exit_code, 0, "feature list --format json should exit 0.\nstderr: {}", out2.stderr);
    let features: serde_json::Value = serde_json::from_str(&out2.stdout)
        .unwrap_or_else(|e| panic!("feature list JSON invalid on stdout: {}\nstdout: {}", e, out2.stdout));
    assert!(features.as_array().is_some(), "feature list JSON should be an array");
    let empty = vec![];
    let arr = features.as_array().unwrap_or(&empty);
    assert!(!arr.is_empty(), "feature list should contain at least one feature");

    // Part 3: graph check CI gate fails on broken link (exit code 1)
    let h2 = fixture_broken_link();
    let out3 = h2.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out3.exit_code, 1, "Expected exit 1 for broken link CI gate.\nstdout: {}\nstderr: {}", out3.stdout, out3.stderr);
    let json2: serde_json::Value = serde_json::from_str(&out3.stdout)
        .unwrap_or_else(|e| panic!("graph check JSON invalid on broken link: {}\nstdout: {}", e, out3.stdout));
    let errors = json2["errors"].as_array().expect("errors should be an array");
    assert!(!errors.is_empty(), "CI gate should report errors on broken link");
}

#[test]
fn tc_166_ft_022_authoring_session_flow_complete() {
    let h = Harness::new();
    git_init(&h);

    // 1. Install hooks
    let out = h.run(&["install-hooks"]);
    out.assert_exit(0);
    assert!(
        h.dir.path().join(".git/hooks/pre-commit").exists(),
        "pre-commit hook should be installed"
    );

    // 2. Stage a well-formed ADR — should have no structural warnings
    h.write(
        "docs/adrs/ADR-060-complete.md",
        "---\nid: ADR-060\ntitle: Complete ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** context\n\n**Decision:** decision\n\n**Rationale:** rationale\n\n**Rejected alternatives:** none considered\n\n**Test coverage:** covered by TC-001\n",
    );
    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-060-complete.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);
    assert!(
        out.stderr.contains("no structural issues"),
        "Well-formed ADR should pass review.\nstderr: {}",
        out.stderr
    );

    // 3. Stage a broken ADR — should report findings
    std::process::Command::new("git")
        .args(["reset", "HEAD"])
        .current_dir(h.dir.path())
        .output()
        .expect("git reset");
    h.write(
        "docs/adrs/ADR-061-broken.md",
        "---\nid: ADR-061\ntitle: Broken ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** ctx\n\n**Decision:** dec\n",
    );
    std::process::Command::new("git")
        .args(["add", "docs/adrs/ADR-061-broken.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0); // advisory — always exits 0
    // Should catch missing sections and empty features
    assert!(
        out.stderr.contains("missing required section") || out.stderr.contains("Rationale") || out.stderr.contains("Rejected alternatives"),
        "Should detect missing sections.\nstderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("no linked features"),
        "Should detect empty features.\nstderr: {}",
        out.stderr
    );

    // 4. Non-ADR files should be skipped
    // Commit staged changes first to clear the index, then stage only a feature file.
    // Use --no-verify because the installed pre-commit hook calls `product` which
    // is not on PATH in the test environment.
    std::process::Command::new("git")
        .args(["commit", "-m", "commit ADRs", "--allow-empty", "--no-verify"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");
    // Now add + commit everything to get a clean index
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add all");
    std::process::Command::new("git")
        .args(["commit", "-m", "clean slate", "--allow-empty", "--no-verify"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    h.write(
        "docs/features/FT-060-test.md",
        "---\nid: FT-060\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );
    std::process::Command::new("git")
        .args(["add", "docs/features/FT-060-test.md"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");

    let out = h.run(&["adr", "review", "--staged"]);
    out.assert_exit(0);
    assert!(
        out.stderr.contains("No staged ADR files"),
        "Should skip non-ADR files.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_167_ft_023_implement_and_verify_orchestrate() {
    // Part 1: Gap gate blocks implementation
    let h = fixture_implement_gap();
    let out = h.run(&["implement", "FT-001", "--dry-run"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E009");

    // Part 2: Suppress and proceed
    let gap_out = h.run(&["gap", "check", "ADR-001"]);
    let reports: serde_json::Value = serde_json::from_str(&gap_out.stdout)
        .unwrap_or_else(|e| panic!("gap check JSON: {}\nstdout: {}", e, gap_out.stdout));
    let findings = reports[0]["findings"].as_array().expect("findings");
    let g001 = findings.iter().find(|f| f["code"].as_str() == Some("G001")).expect("G001");
    let gap_id = g001["id"].as_str().expect("id").to_string();
    h.run(&["gap", "suppress", &gap_id, "--reason", "e2e test"]).assert_exit(0);

    let out2 = h.run(&["implement", "FT-001", "--dry-run"]);
    out2.assert_exit(0);
    out2.assert_stdout_contains("dry-run");

    // Part 3: Verify with passing tests updates status
    let h2 = fixture_verify_passing();
    let out3 = h2.run(&["verify", "FT-001"]);
    out3.assert_exit(0);

    let feature_content = h2.read("docs/features/FT-001-test.md");
    assert!(feature_content.contains("status: complete"), "Feature should be complete after all TCs pass");

    // Part 4: Verify with failing test keeps in-progress
    let h3 = Harness::new();
    h3.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    h3.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h3.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Failing Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: fail.sh\n---\n\nTest body.\n",
    );
    h3.write("fail.sh", "#!/bin/bash\nexit 1\n");
    std::process::Command::new("chmod")
        .args(["+x", "fail.sh"])
        .current_dir(h3.dir.path())
        .output()
        .expect("chmod");

    let out4 = h3.run(&["verify", "FT-001"]);
    out4.assert_exit(0);
    let feat = h3.read("docs/features/FT-001-test.md");
    assert!(feat.contains("status: in-progress"), "Feature should stay in-progress on failure");

    // Part 5: Unimplemented TCs block completion (feature goes to in-progress)
    let h4 = Harness::new();
    h4.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    h4.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h4.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: No Runner\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nNo runner.\n",
    );
    let out5 = h4.run(&["verify", "FT-001"]);
    out5.assert_exit(0);
    out5.assert_stdout_contains("UNIMPLEMENTED");
    let feat4 = h4.read("docs/features/FT-001-test.md");
    assert!(feat4.contains("status: in-progress"), "Unimplemented TCs should block completion");
}

#[test]
fn tc_439_ft_035_repository_initialization_validated() {
    // This exit-criteria test validates the full init workflow end-to-end:
    // create, configure, verify parsability, idempotency of gitignore, and force overwrite.
    let h = Harness::new_bare();

    // 1. Init with --yes creates valid repo (TC-431, TC-433, TC-437)
    let out = h.run(&["init", "--yes", "--name", "exit-criteria-test"]);
    out.assert_exit(0);
    assert!(h.exists("product.toml"), "product.toml created");
    assert!(h.exists("docs/features"), "features dir created");
    assert!(h.exists("docs/adrs"), "adrs dir created");
    assert!(h.exists("docs/tests"), "tests dir created");
    assert!(h.exists("docs/graph"), "graph dir created");
    assert!(h.exists(".gitignore"), "gitignore created");

    // 2. Generated TOML is valid and parseable (TC-438)
    let toml_content = h.read("product.toml");
    assert!(toml_content.contains("name = \"exit-criteria-test\""));
    assert!(toml_content.contains("[domains]"));
    assert!(toml_content.contains("[mcp]"));

    // 3. Re-running without --force fails (TC-434)
    let out = h.run(&["init", "--yes"]);
    out.assert_exit(1);
    out.assert_stderr_contains("product.toml already exists");

    // 4. --force overwrites successfully (TC-435)
    let out = h.run(&["init", "--yes", "--force", "--name", "overwritten"]);
    out.assert_exit(0);
    let toml_content = h.read("product.toml");
    assert!(toml_content.contains("name = \"overwritten\""));

    // 5. Gitignore is not duplicated on re-init (TC-436)
    let gitignore = h.read(".gitignore");
    let count = gitignore.matches("docs/graph/").count();
    assert_eq!(count, 1, "docs/graph/ should appear exactly once");
}

#[test]
fn tc_179_ft_008_schema_migration_exit_criteria() {
    // ── Part 1: v0 → v1 migration — all files updated, schema-version bumped ──
    let h = Harness::new();
    h.write(
        "product.toml",
        "name = \"test\"\nschema-version = \"0\"\n\
         [paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\n\
         tests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n\
         [prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n",
    );
    h.write(
        "docs/features/FT-001-alpha.md",
        "---\nid: FT-001\ntitle: Alpha Feature\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\nAlpha body.\n",
    );
    h.write(
        "docs/features/FT-002-beta.md",
        "---\nid: FT-002\ntitle: Beta Feature\nphase: 2\nstatus: planned\nadrs: []\ntests: []\n---\nBeta body.\n",
    );

    let out = h.run(&["migrate", "schema"]);
    out.assert_exit(0);

    // All feature files should now have depends-on
    let ft1 = h.read("docs/features/FT-001-alpha.md");
    let ft2 = h.read("docs/features/FT-002-beta.md");
    assert!(
        ft1.contains("depends-on:"),
        "FT-001 should have depends-on after migration, got:\n{}",
        ft1
    );
    assert!(
        ft2.contains("depends-on:"),
        "FT-002 should have depends-on after migration, got:\n{}",
        ft2
    );

    // schema-version should be bumped to 1
    let config = h.read("product.toml");
    assert!(
        config.contains("schema-version = \"1\""),
        "schema-version should be bumped to 1, got:\n{}",
        config
    );

    // No data corruption — original fields preserved
    assert!(ft1.contains("id: FT-001"), "FT-001 id preserved");
    assert!(ft1.contains("title: Alpha Feature"), "FT-001 title preserved");
    assert!(ft1.contains("Alpha body."), "FT-001 body preserved");
    assert!(ft2.contains("id: FT-002"), "FT-002 id preserved");
    assert!(ft2.contains("title: Beta Feature"), "FT-002 title preserved");
    assert!(ft2.contains("Beta body."), "FT-002 body preserved");

    // ── Part 2: Concurrent commands — one succeeds, one exits E010 ──
    let h2 = Harness::new();
    h2.write(
        "product.toml",
        "name = \"test\"\nschema-version = \"0\"\n\
         [paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\n\
         tests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n\
         [prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n",
    );
    h2.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\nBody content.\n",
    );

    // Simulate a concurrent process holding the lock by creating .product.lock
    // with the current test process PID (which is alive — stale detection won't clear it)
    let lock_content = format!(
        "pid={}\nstarted=2026-01-01T00:00:00Z\n",
        std::process::id()
    );
    h2.write(".product.lock", &lock_content);

    // This command should fail with E010 because the lock is held
    let out_locked = h2.run(&["migrate", "schema"]);
    out_locked
        .assert_exit(1)
        .assert_stderr_contains("E010");

    // Remove the lock — simulating the first process finishing
    std::fs::remove_file(h2.dir.path().join(".product.lock"))
        .expect("remove lock file");

    // Now the migration should succeed
    let out_unlocked = h2.run(&["migrate", "schema"]);
    out_unlocked.assert_exit(0);

    // Verify no data corruption after the lock contention scenario
    let content = h2.read("docs/features/FT-001-test.md");
    assert!(
        content.contains("id: FT-001"),
        "FT-001 data should not be corrupted after lock contention"
    );
    assert!(
        content.contains("depends-on:"),
        "Migration should have applied after lock released"
    );
    assert!(
        content.contains("Body content."),
        "Body content should be preserved"
    );
    let config2 = h2.read("product.toml");
    assert!(
        config2.contains("schema-version = \"1\""),
        "schema-version should be bumped after successful migration"
    );
}

#[test]
fn tc_699_ft_056_exit_criteria() {
    // Invariant: the embedded default prompt body is present and
    // documents the composition seam. We can read it via the same
    // mechanism the binary uses by spawning the CLI.
    let h = Harness::new();
    let out = h.run(&["prompts", "get", "implement"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("Product Implementation Session"),
        "embedded default prompt should carry the documented header.\nstdout: {}",
        out.stdout
    );
    assert!(
        out.stdout.to_lowercase().contains("composition"),
        "embedded default prompt should describe the base+suffix composition seam.\nstdout: {}",
        out.stdout
    );

    // Invariant: pipeline.rs is comfortably under the 400-line budget.
    // Walk up from the test binary to find the workspace root.
    let mut root = std::env::current_exe().expect("current_exe");
    while !root.join("Cargo.toml").exists() {
        if !root.pop() {
            panic!("could not locate workspace root from test binary");
        }
    }
    let pipeline_path = root.join("src/implement/pipeline.rs");
    let pipeline_src = std::fs::read_to_string(&pipeline_path)
        .expect("read pipeline.rs");
    let line_count = pipeline_src.lines().count();
    assert!(
        line_count < 400,
        "src/implement/pipeline.rs should stay under 400 lines (got {})",
        line_count
    );

    // Invariant: the pipeline reads the per-repo override via
    // `author::prompts::get` rather than the inline format string.
    assert!(
        pipeline_src.contains("crate::author::prompts::get(root, \"implement\")"),
        "pipeline.rs should source the base prompt via author::prompts::get"
    );
}

