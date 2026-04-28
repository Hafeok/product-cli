//! Integration tests — parser.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn it_008_bad_yaml_no_panic() {
    let h = Harness::new();
    h.write("docs/features/bad.md", "not yaml at all {{{");
    let out = h.run(&["feature", "list"]);
    // Should not contain "panicked"
    assert!(!out.stderr.contains("panicked"), "Should not panic on bad YAML");
}

#[test]
fn tc_005_frontmatter_parse_feature() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 2\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-a.md",
        "---\nid: ADR-001\ntitle: ADR One\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-002-b.md",
        "---\nid: ADR-002\ntitle: ADR Two\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/tests/TC-001-a.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nBody.\n",
    );
    h.write(
        "docs/tests/TC-002-b.md",
        "---\nid: TC-002\ntitle: Test Two\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nBody.\n",
    );
    // Feature list should parse and show FT-001
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0).assert_stdout_contains("FT-001").assert_stdout_contains("Test Feature");
    // Feature show should show all linked ADRs and tests
    let out = h.run(&["feature", "show", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("ADR-001"), "Should show linked ADR-001");
    assert!(out.stdout.contains("ADR-002"), "Should show linked ADR-002");
    assert!(out.stdout.contains("TC-001"), "Should show linked TC-001");
    assert!(out.stdout.contains("TC-002"), "Should show linked TC-002");
}

#[test]
fn tc_006_frontmatter_parse_adr() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-main.md",
        "---\nid: ADR-001\ntitle: Main Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: [ADR-002]\n---\n\nDecision body.\n",
    );
    h.write(
        "docs/adrs/ADR-002-new.md",
        "---\nid: ADR-002\ntitle: Replacement Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: [ADR-001]\nsuperseded-by: []\n---\n\nNew decision body.\n",
    );
    let out = h.run(&["adr", "list"]);
    out.assert_exit(0).assert_stdout_contains("ADR-001").assert_stdout_contains("ADR-002");
    let out = h.run(&["adr", "show", "ADR-002"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("ADR-001") || out.stdout.contains("supersedes"), "ADR-002 should show supersession info");
}

#[test]
fn tc_007_frontmatter_invalid_id() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-999]\ntests: []\n---\n\nBody.\n",
    );
    let out = h.run(&["graph", "check"]);
    // Should report broken link (E002) and exit with code 1
    assert!(
        out.stderr.contains("E002") || out.stderr.contains("broken link"),
        "Expected broken link error, got stderr: {}",
        out.stderr
    );
    assert_eq!(out.exit_code, 1, "graph check should exit 1 on broken link");
}

#[test]
fn tc_008_frontmatter_missing_required() {
    let h = Harness::new();
    // Feature file with no id field
    h.write("docs/features/FT-001-bad.md", "---\ntitle: Missing ID\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    let out = h.run(&["feature", "list"]);
    // Should produce E006 or a YAML parse error about missing field
    assert!(
        out.stderr.contains("E006") || out.stderr.contains("missing"),
        "Expected missing field error, got stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_071_parse_types_block() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-types.md",
        "---\nid: TC-001\ntitle: Types\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Σ:Types⟧{\n  Node≜IRI\n  Role≜Leader|Follower\n}\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Node≜IRI"), "Should contain Node type def: {}", out.stdout);
    assert!(out.stdout.contains("Role≜Leader|Follower"), "Should contain Role union type: {}", out.stdout);
}

#[test]
fn tc_072_parse_invariants_block() {
    let h = Harness::new();
    let invariant = "∀x:Node: connected(x) = true";
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-inv.md",
        &format!("---\nid: TC-001\ntitle: Invariants\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{{\n  {}\n}}\n", invariant),
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains(invariant), "Invariant raw should roundtrip verbatim: {}", out.stdout);
}

#[test]
fn tc_073_parse_scenario_block() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-scen.md",
        "---\nid: TC-001\ntitle: Scenario\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Λ:Scenario⟧{\n  given≜cluster_init(nodes:3)\n  when≜leader_fails()\n  then≜new_leader_elected()\n}\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("given≜"), "Should contain given field: {}", out.stdout);
    assert!(out.stdout.contains("when≜"), "Should contain when field: {}", out.stdout);
    assert!(out.stdout.contains("then≜"), "Should contain then field: {}", out.stdout);
}

#[test]
fn tc_074_parse_evidence_block() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-ev.md",
        "---\nid: TC-001\ntitle: Evidence\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Evidence block should be preserved in output
    assert!(out.stdout.contains("δ≜0.95") || out.stdout.contains("0.95"), "Should contain delta value: {}", out.stdout);
}

#[test]
fn tc_075_parse_evidence_delta_out_of_range() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-bad-ev.md",
        "---\nid: TC-001\ntitle: Bad Evidence\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Ε⟧⟨δ≜1.5;φ≜100;τ≜◊⁺⟩\n",
    );
    // Graph check should report E001 for out-of-range delta
    let out = h.run(&["graph", "check"]);
    assert!(
        out.stderr.contains("E001") || out.stderr.contains("out of range"),
        "Expected E001 for out-of-range delta, got stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_076_parse_unclosed_delimiter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    // Unclosed brace — note we also add a valid evidence block after to verify error recovery
    h.write(
        "docs/tests/TC-001-unclosed.md",
        "---\nid: TC-001\ntitle: Unclosed\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{ ∀x:Node: x.id > 0\n\n⟦Ε⟧⟨δ≜0.90;φ≜50;τ≜◊?⟩\n",
    );
    let out = h.run(&["graph", "check"]);
    // Should report E001 for unclosed delimiter
    assert!(
        out.stderr.contains("E001") || out.stderr.contains("unclosed"),
        "Expected unclosed delimiter error, got stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_077_parse_empty_block_warning() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-empty.md",
        "---\nid: TC-001\ntitle: Empty\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{}\n",
    );
    let out = h.run(&["graph", "check"]);
    // W004 warning for empty block — should still succeed (exit 0 or 2 for warnings)
    assert!(
        out.stderr.contains("W004") || out.stderr.contains("empty block"),
        "Expected W004 empty block warning, got stderr: {}",
        out.stderr
    );
    // Should NOT exit with code 1 (that's errors only)
    assert_ne!(out.exit_code, 1, "Empty block should be a warning, not an error");
}

#[test]
fn tc_079_parse_unknown_block_type() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-unknown.md",
        "---\nid: TC-001\ntitle: Unknown Block\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦X:Unknown⟧{ some content }\n",
    );
    let out = h.run(&["graph", "check"]);
    assert!(
        out.stderr.contains("E001") || out.stderr.contains("unrecognised block type"),
        "Expected unrecognised block type error, got stderr: {}",
        out.stderr
    );
}

#[test]
fn tc_078_parse_raw_roundtrip() {
    // We test this indirectly: write a TC with an invariant block, include it in a context bundle,
    // and verify the raw content appears verbatim.
    let h = Harness::new();
    let invariant_text = "∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1";
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n");
    h.write("docs/tests/TC-001-test.md", &format!(
        "---\nid: TC-001\ntitle: Inv Test\ntype: invariant\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{{\n  {}\n}}\n",
        invariant_text
    ));
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains(invariant_text),
        "Invariant raw text should roundtrip through context bundle: {}",
        out.stdout
    );
}

#[test]
fn tc_035_formal_block_parse_types() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-types.md",
        "---\nid: TC-001\ntitle: Types Block\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Σ:Types⟧{\n  Node≜IRI\n  Role≜Leader|Follower|Learner\n  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩\n}\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // All three type definitions should be present with correct names and variants
    assert!(out.stdout.contains("Node≜IRI"), "Should contain Node type def: {}", out.stdout);
    assert!(
        out.stdout.contains("Role≜Leader|Follower|Learner"),
        "Should contain Role union type with all variants: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains("ClusterState≜⟨nodes:Node+, roles:Node→Role⟩"),
        "Should contain ClusterState tuple type: {}",
        out.stdout
    );
}

#[test]
fn tc_036_formal_block_parse_invariants() {
    let h = Harness::new();
    let invariant = "∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1";
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-inv.md",
        &format!(
            "---\nid: TC-001\ntitle: Invariants\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Γ:Invariants⟧{{\n  {}\n}}\n",
            invariant
        ),
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Invariant with universal quantifier should be preserved verbatim
    assert!(out.stdout.contains("∀"), "Should contain universal quantifier: {}", out.stdout);
    assert!(
        out.stdout.contains(invariant),
        "Invariant expression should roundtrip verbatim: {}",
        out.stdout
    );
    // Verify the block delimiter is present
    assert!(out.stdout.contains("⟦Γ:Invariants⟧"), "Should contain invariants block delimiter: {}", out.stdout);
}

#[test]
fn tc_037_formal_block_parse_scenario() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-scenario.md",
        "---\nid: TC-001\ntitle: Scenario Block\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Λ:Scenario⟧{\n  given≜cluster_init(nodes:2)\n  when≜elapsed(10s)\n  then≜∃n∈nodes: roles(n)=Leader ∧ graph_contains(n, picloud:hasRole, picloud:Leader)\n}\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // All three scenario fields must be present and non-empty
    assert!(out.stdout.contains("given≜cluster_init(nodes:2)"), "given field should be present and non-empty: {}", out.stdout);
    assert!(out.stdout.contains("when≜elapsed(10s)"), "when field should be present and non-empty: {}", out.stdout);
    assert!(out.stdout.contains("then≜∃n∈nodes"), "then field should be present and non-empty: {}", out.stdout);
}

#[test]
fn tc_038_formal_block_evidence() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    h.write(
        "docs/tests/TC-001-evidence.md",
        "---\nid: TC-001\ntitle: Evidence\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩\n",
    );
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Evidence block should be preserved with all three fields
    assert!(out.stdout.contains("δ≜0.95"), "Should contain delta=0.95: {}", out.stdout);
    assert!(out.stdout.contains("φ≜100"), "Should contain phi=100: {}", out.stdout);
    assert!(out.stdout.contains("τ≜◊⁺"), "Should contain tau=Stable (◊⁺): {}", out.stdout);
}

#[test]
fn tc_039_formal_block_missing_invariant_warning() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature.\n",
    );
    // An invariant-type TC with NO formal blocks — only prose
    h.write(
        "docs/tests/TC-001-no-formal.md",
        "---\nid: TC-001\ntitle: Missing Formal\ntype: invariant\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nThis invariant-type test criterion has no formal blocks.\nIt only has prose description.\n",
    );
    let out = h.run(&["graph", "check"]);
    // Should produce W004 warning for missing formal blocks on invariant type
    assert!(
        out.stderr.contains("W004") || out.stderr.contains("missing formal"),
        "Expected W004 for invariant TC missing formal blocks, got stderr: {}",
        out.stderr
    );
    // Exit code should be 2 (warnings), not 1 (errors)
    assert_eq!(out.exit_code, 2, "Missing formal blocks should be warning (exit 2), not error (exit 1), got exit code: {}", out.exit_code);
}

#[test]
fn tc_011_markdown_front_matter_strip() {
    let h = fixture_minimal();
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // No YAML front-matter delimiters in output
    assert!(!out.stdout.starts_with("---\n"), "Context should not start with front-matter delimiter");
    // Check no raw YAML fields leaked
    assert!(!out.stdout.contains("status: planned"), "YAML fields should not appear in context bundle");
    assert!(!out.stdout.contains("depends-on:"), "YAML fields should not appear in context bundle");
}

#[test]
fn tc_012_markdown_passthrough() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\n```rust\nfn main() {}\n```\n\n| Col1 | Col2 |\n|------|------|\n| a    | b    |\n\n- item 1\n  - nested\n");
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("```rust"), "Code blocks preserved");
    assert!(out.stdout.contains("fn main() {}"), "Code content preserved");
    assert!(out.stdout.contains("| Col1 | Col2 |"), "Tables preserved");
    assert!(out.stdout.contains("- item 1"), "Lists preserved");
    assert!(out.stdout.contains("  - nested"), "Nested lists preserved");
}

#[test]
fn tc_013_id_auto_increment() {
    let h = Harness::new();
    let out1 = h.run(&["feature", "new", "First"]);
    out1.assert_exit(0).assert_stdout_contains("FT-001");
    let out2 = h.run(&["feature", "new", "Second"]);
    out2.assert_exit(0).assert_stdout_contains("FT-002");
    let out3 = h.run(&["feature", "new", "Third"]);
    out3.assert_exit(0).assert_stdout_contains("FT-003");
}

#[test]
fn tc_014_id_gap_fill() {
    let h = Harness::new();
    // Create FT-001 and FT-003 (gap at FT-002)
    h.write("docs/features/FT-001-first.md", "---\nid: FT-001\ntitle: First\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nFirst feature.\n");
    h.write("docs/features/FT-003-third.md", "---\nid: FT-003\ntitle: Third\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nThird feature.\n");

    // Run product feature new
    let out = h.run(&["feature", "new", "Gap Test"]);
    out.assert_exit(0);
    // Should assign FT-004 (max+1), NOT FT-002 (gap fill)
    assert!(
        out.stdout.contains("FT-004"),
        "Expected FT-004 (max+1, no gap fill), got stdout: {}",
        out.stdout
    );
    // FT-002 should NOT exist
    assert!(
        !h.exists("docs/features/FT-002-gap-test.md"),
        "FT-002 should not be created — gaps are not filled"
    );
}

#[test]
fn tc_015_id_conflict() {
    let h = Harness::new();
    // Create two feature files with the same ID
    h.write("docs/features/FT-001-alpha.md", "---\nid: FT-001\ntitle: Alpha\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nAlpha feature.\n");
    h.write("docs/features/FT-001-beta.md", "---\nid: FT-001\ntitle: Beta\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBeta feature.\n");

    // graph check should report a duplicate ID error
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1)
        .assert_stderr_contains("E011");
    assert!(
        out.stderr.contains("FT-001"),
        "Error should mention the duplicate ID FT-001, got stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("duplicate"),
        "Error should mention 'duplicate', got stderr: {}",
        out.stderr
    );

    // Both files should still exist (nothing overwritten)
    assert!(h.exists("docs/features/FT-001-alpha.md"), "Alpha file should still exist");
    assert!(h.exists("docs/features/FT-001-beta.md"), "Beta file should still exist");
}

#[test]
fn tc_081_title() {
    let h = Harness::new();
    let prd_source = "# PRD\n\n## 5. Products and IAM\n\nContent about products.\n\n## Storage Model\n\nStorage stuff.\n";
    h.write("source-prd.md", prd_source);
    let out = h.run(&["migrate", "from-prd", "source-prd.md", "--execute"]);
    out.assert_exit(0);

    // Check that feature files were created with correct titles
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .expect("readdir")
        .flatten()
        .collect();
    assert_eq!(entries.len(), 2, "should create 2 feature files");

    // Verify titles: "5. Products and IAM" should become "Products and IAM" (stripped number)
    let mut found_products = false;
    let mut found_storage = false;
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("title: Products and IAM") {
            found_products = true;
        }
        if content.contains("title: Storage Model") {
            found_storage = true;
        }
    }
    assert!(found_products, "title should strip leading number: '5. Products and IAM' → 'Products and IAM'");
    assert!(found_storage, "title 'Storage Model' should be preserved as-is");
}

#[test]
fn tc_082_type() {
    let h = Harness::new();
    let adr_source = r#"# ADRs

## ADR-001: Test Types

**Status:** Accepted

Context.

### Test coverage

- `chaos_network_partition` — chaos test for partitions
- `invariant_monotonic_clock` — invariant for clock
- `binary_compiles` — scenario test
"#;
    h.write("source-adrs.md", adr_source);
    let out = h.run(&["migrate", "from-adrs", "source-adrs.md", "--execute"]);
    out.assert_exit(0);

    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/tests"))
        .expect("readdir")
        .flatten()
        .collect();

    let mut found_chaos = false;
    let mut found_invariant = false;
    let mut found_scenario = false;
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("type: chaos") {
            found_chaos = true;
        }
        if content.contains("type: invariant") {
            found_invariant = true;
        }
        if content.contains("type: scenario") {
            found_scenario = true;
        }
    }
    assert!(found_chaos, "bullet containing 'chaos' should produce type: chaos");
    assert!(found_invariant, "bullet containing 'invariant' should produce type: invariant");
    assert!(found_scenario, "other bullets should produce type: scenario");
}

#[test]
fn tc_420_hash_computed_on_adr_acceptance() {
    let h = Harness::new();
    // Create a feature so ADR is not orphaned
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n",
    );

    // Create a new ADR
    let out = h.run(&["adr", "new", "Test Content Hash"]);
    out.assert_exit(0);

    // Find the created ADR file
    let adr_dir = h.dir.path().join("docs/adrs");
    let entries: Vec<_> = std::fs::read_dir(&adr_dir)
        .expect("read adr dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();
    assert_eq!(entries.len(), 1, "should have one ADR file");
    let adr_path = entries[0].path();
    let adr_content = std::fs::read_to_string(&adr_path).expect("read adr");

    // Verify no content-hash in proposed ADR
    assert!(
        !adr_content.contains("content-hash"),
        "Proposed ADR should not have content-hash"
    );

    // Extract the ADR ID from the filename
    let filename = adr_path.file_name().expect("filename").to_str().expect("utf8");
    let adr_id = &filename[..7]; // e.g. "ADR-001"

    // Accept the ADR
    let out = h.run(&["adr", "status", adr_id, "accepted"]);
    out.assert_exit(0);

    // Read back and verify content-hash exists
    let adr_content = std::fs::read_to_string(&adr_path).expect("read adr");
    assert!(
        adr_content.contains("content-hash: sha256:"),
        "Accepted ADR should have content-hash.\nGot:\n{}",
        adr_content
    );

    // Verify the hash matches manual computation
    // Extract title and body from the file
    let hash_line = adr_content
        .lines()
        .find(|l| l.starts_with("content-hash: "))
        .expect("content-hash line");
    let stored_hash = hash_line.strip_prefix("content-hash: ").expect("strip prefix");
    assert!(stored_hash.starts_with("sha256:"), "hash should start with sha256:");
    assert_eq!(stored_hash.len(), 7 + 64, "hash should be sha256: + 64 hex chars");

    // Manual computation: extract body from file
    let parts: Vec<&str> = adr_content.splitn(3, "---").collect();
    assert!(parts.len() >= 3, "should have front-matter delimiters");
    let body = parts[2].trim_start_matches('\n');
    let expected_hash = compute_adr_content_hash("Test Content Hash", body);
    assert_eq!(
        stored_hash, expected_hash,
        "Stored hash should match manual computation"
    );
}

#[test]
fn tc_426_hash_seal_computes_and_writes_tc_content_hash() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001, TC-002, TC-003]\n---\n\nBody.\n",
    );

    // Create three TCs
    let tc1 = "---\nid: TC-001\ntitle: First Test\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\n## Description\n\nFirst test body.\n";
    let tc2 = "---\nid: TC-002\ntitle: Second Test\ntype: invariant\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n## Description\n\nSecond test body.\n";
    let tc3 = "---\nid: TC-003\ntitle: Already Sealed\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\ncontent-hash: sha256:0000000000000000000000000000000000000000000000000000000000000000\n---\n\n## Description\n\nThird test body.\n";

    h.write("docs/tests/TC-001-first.md", tc1);
    h.write("docs/tests/TC-002-second.md", tc2);
    h.write("docs/tests/TC-003-sealed.md", tc3);

    // Verify TC-001 has no content-hash
    let content = h.read("docs/tests/TC-001-first.md");
    assert!(!content.contains("content-hash"), "TC-001 should not have content-hash yet");

    // Seal TC-001 individually
    let out = h.run(&["hash", "seal", "TC-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("sealed");

    let content = h.read("docs/tests/TC-001-first.md");
    assert!(
        content.contains("content-hash: sha256:"),
        "TC-001 should now have content-hash.\nGot:\n{}",
        content
    );

    // Verify hash matches manual computation
    let stored_hash = content
        .lines()
        .find(|l| l.starts_with("content-hash: "))
        .expect("hash line")
        .strip_prefix("content-hash: ")
        .expect("strip");
    let expected = compute_tc_content_hash(
        "First Test",
        "scenario",
        &["ADR-001"],
        "## Description\n\nFirst test body.\n",
    );
    assert_eq!(stored_hash, expected, "Hash should match manual computation");

    // Seal all unsealed TCs
    let out = h.run(&["hash", "seal", "--all-unsealed"]);
    out.assert_exit(0);
    out.assert_stdout_contains("TC-002"); // TC-002 should get sealed

    // TC-002 should now have hash
    let content = h.read("docs/tests/TC-002-second.md");
    assert!(content.contains("content-hash: sha256:"), "TC-002 should now have hash");

    // TC-003 should NOT have been modified (already sealed)
    let content = h.read("docs/tests/TC-003-sealed.md");
    assert!(
        content.contains("content-hash: sha256:0000000000000000000000000000000000000000000000000000000000000000"),
        "TC-003 should retain its original hash"
    );
}

#[test]
fn tc_427_hash_verify_checks_content_hashes_independently() {
    let h = Harness::new();

    // Create a valid accepted ADR
    let valid_body = "Valid decision body.\n";
    let valid_hash = compute_adr_content_hash("Valid ADR", valid_body.trim());
    h.write(
        "docs/adrs/ADR-001-valid.md",
        &format!(
            "---\nid: ADR-001\ntitle: Valid ADR\nstatus: accepted\ncontent-hash: {}\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\n{}",
            valid_hash, valid_body
        ),
    );

    // Create a tampered accepted ADR
    let tampered_hash = compute_adr_content_hash("Tampered ADR", "Original body.");
    h.write(
        "docs/adrs/ADR-002-tampered.md",
        &format!(
            "---\nid: ADR-002\ntitle: Tampered ADR\nstatus: accepted\ncontent-hash: {}\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nModified body that doesn't match hash.\n",
            tampered_hash
        ),
    );

    // hash verify should report E014 for the tampered one
    let out = h.run(&["hash", "verify"]);
    assert_eq!(out.exit_code, 1, "Should fail with exit 1 for tampered hash.\nstderr: {}", out.stderr);
    out.assert_stderr_contains("E014");

    // Verify specific ADR — valid one should pass
    let out = h.run(&["hash", "verify", "ADR-001"]);
    assert_eq!(out.exit_code, 0, "Valid ADR should pass.\nstderr: {}", out.stderr);

    // Verify specific tampered ADR should fail
    let out = h.run(&["hash", "verify", "ADR-002"]);
    assert_eq!(out.exit_code, 1, "Tampered ADR should fail.\nstderr: {}", out.stderr);
    out.assert_stderr_contains("E014");

    // hash verify should NOT run full graph checks (no orphan warnings etc.)
    let all_out = h.run(&["hash", "verify"]);
    assert!(
        !all_out.stderr.contains("W001"),
        "hash verify should not run orphan checks.\nstderr: {}",
        all_out.stderr
    );
}

#[test]
fn tc_430_content_hash_system_passes_on_sealed_repository() {
    let h = Harness::new();

    // Set up a repo with accepted ADRs and finalized TCs
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-decision.md",
        "---\nid: ADR-001\ntitle: Test Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test Criterion\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\n## Description\n\nTest body.\n",
    );

    // Before sealing, graph check should emit W016
    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W016");

    // Seal everything
    h.run(&["adr", "rehash", "--all"]).assert_exit(0);
    h.run(&["hash", "seal", "--all-unsealed"]).assert_exit(0);

    // 1. graph check should produce zero E014, E015, or W016
    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E014"),
        "Should not have E014 after sealing.\nstderr: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("E015"),
        "Should not have E015 after sealing.\nstderr: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("W016"),
        "Should not have W016 after sealing.\nstderr: {}",
        out.stderr
    );

    // 2. hash verify exits with code 0
    let out = h.run(&["hash", "verify"]);
    assert_eq!(
        out.exit_code, 0,
        "hash verify should pass on sealed repo.\nstderr: {}",
        out.stderr
    );

    // 3. adr amend succeeds and subsequent graph check still passes
    // First, modify the ADR body slightly
    let adr_content = h.read("docs/adrs/ADR-001-decision.md");
    let modified = adr_content.replace("Decision body.", "Decision body with correction.");
    std::fs::write(
        h.dir.path().join("docs/adrs/ADR-001-decision.md"),
        &modified,
    )
    .expect("write modified");

    let out = h.run(&["adr", "amend", "ADR-001", "--reason", "test amendment"]);
    out.assert_exit(0);

    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E014"),
        "graph check should still pass after amend.\nstderr: {}",
        out.stderr
    );

    let out = h.run(&["hash", "verify"]);
    assert_eq!(
        out.exit_code, 0,
        "hash verify should pass after amend.\nstderr: {}",
        out.stderr
    );
}

#[test]
fn tc_471_front_matter_field_management_complete() {
    let h = fixture_with_domains();
    // 1. Create a feature, ADR, and TC
    h.run(&["feature", "new", "Test Feature"]).assert_exit(0);
    h.run(&["adr", "new", "Test Decision"]).assert_exit(0);
    h.run(&["test", "new", "Test Criterion"]).assert_exit(0);

    // 2. Feature domain management
    h.run(&["feature", "domain", "FT-001", "--add", "api", "--add", "security"]).assert_exit(0);
    h.run(&["feature", "domain", "FT-001", "--remove", "security"]).assert_exit(0);
    let content = h.read("docs/features/FT-001-test-feature.md");
    assert!(content.contains("api"), "feature should have api domain");

    // 3. Feature acknowledgement
    h.run(&["feature", "acknowledge", "FT-001", "--domain", "networking", "--reason", "Not applicable"]).assert_exit(0);
    let content = h.read("docs/features/FT-001-test-feature.md");
    assert!(content.contains("Not applicable"), "feature should have acknowledgement");

    // 4. ADR domain + scope
    h.run(&["adr", "domain", "ADR-001", "--add", "error-handling"]).assert_exit(0);
    h.run(&["adr", "scope", "ADR-001", "cross-cutting"]).assert_exit(0);
    let content = h.read("docs/adrs/ADR-001-test-decision.md");
    assert!(content.contains("error-handling"), "ADR should have error-handling domain");
    assert!(content.contains("cross-cutting"), "ADR should have cross-cutting scope");

    // 5. ADR source files
    h.run(&["adr", "source-files", "ADR-001", "--add", "src/test.rs"]).assert_exit(0);

    // 6. Test runner configuration
    h.run(&["test", "runner", "TC-001", "--runner", "cargo-test", "--args", "tc_001_test"]).assert_exit(0);
    let content = h.read("docs/tests/TC-001-test-criterion.md");
    assert!(content.contains("cargo-test"), "TC should have runner");

    // 7. Full authoring session is possible without manual YAML editing
    // All above commands succeeded — complete authoring flow works
}

#[test]
fn tc_497_body_mutation_on_accepted_adr_succeeds_and_surfaces_e014() {
    let h = fixture_request();
    // Make ADR-001 accepted + sealed.
    h.write(
        "docs/adrs/ADR-001-seed.md",
        "---\nid: ADR-001\ntitle: Seed ADR\nstatus: accepted\nfeatures:\n- FT-001\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: feature-specific\n---\n\n## Context\n\nInitial body.\n\n## Decision\n\nDecision.\n\n## Rationale\n\nRationale.\n\n## Rejected alternatives\n\nNone.\n\n## Test coverage\n\nTC.\n",
    );
    h.run(&["hash", "seal", "--all-unsealed"]);  // (may or may not operate on ADRs; we also try rehash below)
    h.run(&["adr", "rehash", "--all"]);

    write_req(
        &h,
        "rbody.yaml",
        "type: change\nschema-version: 1\nreason: \"fix typo\"\nchanges:\n  - target: ADR-001\n    mutations:\n      - op: set\n        field: body\n        value: \"## Context\\n\\nCorrected body.\\n\"\n",
    );
    let out = h.run(&["request", "apply", "rbody.yaml"]);
    out.assert_exit(0);
    // Subsequent graph check should surface E014.
    let check = h.run(&["graph", "check"]);
    assert!(
        check.stderr.contains("E014") || check.exit_code == 1,
        "graph check should surface E014 after body mutation on accepted ADR. exit={} stderr={}",
        check.exit_code,
        check.stderr
    );
}

#[test]
fn tc_619_formal_blocks_schema_exit() {
    let h = Harness::new();

    // 1. `product schema` includes the Formal Blocks section.
    let out = h.run(&["schema"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("## Formal Blocks"));

    // 2. `product schema --type formal` renders only the Formal Blocks section.
    let out_formal = h.run(&["schema", "--type", "formal"]);
    out_formal.assert_exit(0);
    assert!(out_formal.stdout.contains("Sigma-Types"));
    assert!(!out_formal.stdout.contains("## Feature"));
    assert!(!out_formal.stdout.contains("## Dependency"));

    // 3. TC schema cross-reference.
    let tc_out = h.run(&["schema", "test"]);
    tc_out.assert_exit(0);
    assert!(tc_out.stdout.contains("Formal Blocks"));

    // 4. `product agent-init` regenerates AGENTS.md with the new section.
    let init_out = h.run(&["agent-init"]);
    init_out.assert_exit(0);
    assert!(h.exists("AGENTS.md"), "AGENTS.md should be created");
    let agent_md = h.read("AGENTS.md");
    // The schemas section is included by default — the formal block schema
    // is reachable through the test-schema's cross-reference at minimum.
    assert!(
        agent_md.contains("Formal Blocks") || agent_md.contains("Sigma-Types")
            || agent_md.contains("Front-Matter Schemas"),
        "AGENTS.md should surface the new section or its cross-reference; got:\n{}",
        agent_md
    );

    // 5. An `invariant` TC with a Gamma-Invariants block (exactly the form
    // the schema teaches) passes `graph check` without W004.
    h.write(
        "docs/features/FT-001.md",
        "---\nid: FT-001\ntitle: F\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nBody\n",
    );
    h.write(
        "docs/tests/TC-001.md",
        "---\nid: TC-001\ntitle: Inv\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ x = 1 }\n",
    );
    let check = h.run(&["graph", "check"]);
    // Exit 0 or 2 (warnings unrelated to W004 are acceptable); W004 must
    // not be emitted for a TC that carries the block the schema taught.
    assert!(
        check.exit_code == 0 || check.exit_code == 2,
        "graph check should pass (exit 0 or 2); got {}; stderr: {}",
        check.exit_code, check.stderr
    );
    assert!(
        !check.stderr.contains("W004") && !check.stdout.contains("W004"),
        "invariant TC with Gamma-Invariants block must not trigger W004; stderr: {}",
        check.stderr
    );
}

#[test]
fn tc_690_empty_meaning_section_satisfies_w030() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    let body = "\
## Description

x

## Functional Specification

### Inputs

x

### Outputs

x

### State

Stateless. No data is retained between requests.

### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x

## Out of scope

x
";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let count = warnings.iter().filter(|w| w["code"] == "W030").count();
    assert_eq!(count, 0, "empty-meaning content satisfies W030, got: {:#?}", warnings);
}

#[test]
fn tc_691_whitespace_only_section_emits_w030() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    let body = "\
## Description

x

## Functional Specification

### Inputs

x

### Outputs

x

### State



### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x

## Out of scope

x
";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030: Vec<&serde_json::Value> = warnings.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030.len(), 1, "expected one W030; got: {:#?}", warnings);
    let detail = w030[0]["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("Functional Specification > State"));
    assert!(!detail.contains("Functional Specification > Behaviour"));
}

#[test]
fn tc_692_absent_section_emits_w030() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    let body = "\
## Description

x

## Functional Specification

### Inputs

x

### Outputs

x

### State

x

### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x
";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["graph", "check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let warnings = json["warnings"].as_array().expect("warnings");
    let w030: Vec<&serde_json::Value> = warnings.iter().filter(|w| w["code"] == "W030").collect();
    assert_eq!(w030.len(), 1);
    let detail = w030[0]["detail"].as_str().unwrap_or_default();
    assert!(
        detail.contains("- Out of scope"),
        "expected 'Out of scope' missing in detail:\n{}",
        detail
    );
    // Make sure exactly one section is reported missing in this body.
    let dash_count = detail.matches("\n  -").count();
    assert_eq!(dash_count, 1, "expected exactly one missing section bullet:\n{}", detail);
}

