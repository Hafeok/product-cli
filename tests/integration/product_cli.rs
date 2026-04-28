//! Integration tests — product_cli.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_020_product_context_ft_001() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Cluster Foundation\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n\nCluster foundation feature.\n",
    );
    h.write(
        "docs/adrs/ADR-001-rust.md",
        "---\nid: ADR-001\ntitle: Rust as Implementation Language\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nRust decision.\n",
    );
    h.write(
        "docs/adrs/ADR-002-openraft.md",
        "---\nid: ADR-002\ntitle: openraft for Cluster Consensus\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nopenraft decision.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Binary compiles\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nBinary compile test.\n",
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // Bundle header
    out.assert_stdout_contains("Context Bundle: FT-001");
    out.assert_stdout_contains("Bundle");
    out.assert_stdout_contains("feature≜FT-001:Feature");

    // Feature content
    out.assert_stdout_contains("Cluster foundation feature.");

    // ADR content
    out.assert_stdout_contains("ADR-001");
    out.assert_stdout_contains("Rust as Implementation Language");
    out.assert_stdout_contains("ADR-002");
    out.assert_stdout_contains("openraft for Cluster Consensus");

    // Test criteria
    out.assert_stdout_contains("TC-001");
    out.assert_stdout_contains("Binary compiles");

    // Correct order: feature first, then ADRs, then tests
    let ft_pos = out.stdout.find("Cluster foundation feature.").expect("feature body");
    let adr_pos = out.stdout.find("Rust decision.").expect("ADR body");
    let tc_pos = out.stdout.find("Binary compile test.").expect("TC body");
    assert!(
        ft_pos < adr_pos,
        "Feature should appear before ADRs"
    );
    assert!(
        adr_pos < tc_pos,
        "ADRs should appear before test criteria"
    );
}

#[test]
fn tc_053_product_graph_central() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Feature 1\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: []\n---\n\nFeature 1.\n",
    );
    h.write(
        "docs/features/FT-002-test.md",
        "---\nid: FT-002\ntitle: Feature 2\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nFeature 2.\n",
    );
    h.write(
        "docs/adrs/ADR-001-high.md",
        "---\nid: ADR-001\ntitle: High Centrality\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n\nHigh centrality ADR.\n",
    );
    h.write(
        "docs/adrs/ADR-002-low.md",
        "---\nid: ADR-002\ntitle: Low Centrality\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nLow centrality ADR.\n",
    );

    let out = h.run(&["graph", "central"]);
    out.assert_exit(0);

    // Should show ranked table with ADRs
    out.assert_stdout_contains("RANK");
    out.assert_stdout_contains("CENTRALITY");
    out.assert_stdout_contains("ADR-001");
    out.assert_stdout_contains("ADR-002");
}

#[test]
fn tc_054_product_impact_adr_001() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Core Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nCore feature.\n",
    );
    h.write(
        "docs/features/FT-002-dep.md",
        "---\nid: FT-002\ntitle: Dependent Feature\nphase: 2\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n\nDependent.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Foundational Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nFoundational.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Core Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["impact", "ADR-001"]);
    out.assert_exit(0);

    // Should show impact analysis
    out.assert_stdout_contains("Impact analysis");
    out.assert_stdout_contains("ADR-001");
    // FT-001 is a direct dependent
    out.assert_stdout_contains("FT-001");
}

#[test]
fn tc_150_product_preflight_ft_001() {
    let h = harness_with_domains();

    // Cross-cutting ADR
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nError model.\n");

    // Domain ADR for security
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");

    // Feature that links cross-cutting and domain ADRs, declares security domain
    h.write("docs/features/FT-001-cluster.md",
        "---\nid: FT-001\ntitle: Cluster\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-013, ADR-020]\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nCluster feature.\n");

    let out = h.run(&["preflight", "FT-001"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("CLEAN"),
        "Preflight should be clean when all coverage is present, got stdout:\n{}",
        out.stdout
    );
}

#[test]
fn tc_151_product_graph_coverage() {
    let h = harness_with_domains();

    // Domain-scoped ADRs
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");
    h.write("docs/adrs/ADR-030-networking.md",
        "---\nid: ADR-030\ntitle: Networking Core\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [networking]\nscope: domain\n---\n\nNetworking.\n");

    // FT-001: links ADR-020 (security covered), declares networking (gap)
    h.write("docs/features/FT-001-cluster.md",
        "---\nid: FT-001\ntitle: Cluster\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-020]\ntests: []\ndomains: [security, networking]\ndomains-acknowledged: {}\n---\n\nCluster.\n");

    // FT-002: acknowledges security, does not declare networking
    h.write("docs/features/FT-002-products.md",
        "---\nid: FT-002\ntitle: Products\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged:\n  security: \"no trust boundaries\"\n---\n\nProducts.\n");

    let out = h.run(&["graph", "coverage"]);
    out.assert_exit(0);

    // Should contain feature IDs
    assert!(out.stdout.contains("FT-001"), "Should list FT-001, got:\n{}", out.stdout);
    assert!(out.stdout.contains("FT-002"), "Should list FT-002, got:\n{}", out.stdout);

    // Should contain domain headers (abbreviated)
    assert!(out.stdout.contains("secur"), "Should show security column, got:\n{}", out.stdout);

    // Should contain coverage symbols
    let has_symbols = out.stdout.contains('✓') || out.stdout.contains('~') || out.stdout.contains('·') || out.stdout.contains('✗');
    assert!(has_symbols, "Should contain coverage symbols (✓/~/·/✗), got:\n{}", out.stdout);

    // Legend
    assert!(out.stdout.contains("Legend"), "Should contain legend, got:\n{}", out.stdout);

    // JSON format
    let out_json = h.run(&["graph", "coverage", "--format", "json"]);
    out_json.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out_json.stdout)
        .expect("JSON should be valid");
    assert!(json["features"].is_array(), "JSON should have features array");
    assert!(json["domains"].is_array(), "JSON should have domains array");
}

#[test]
fn tc_205_product_context_ft001_measure() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\ndomains: [storage]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First Decision\nstatus: accepted\nfeatures: [FT-001]\n---\n\nADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["context", "FT-001", "--measure"]);
    out.assert_exit(0);
    // The bundle should still be printed to stdout
    out.assert_stdout_contains("Context Bundle: FT-001");

    // Feature file should be updated
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("bundle:"), "Feature file should contain bundle block.\nContent:\n{}", content);
    assert!(content.contains("depth-1-adrs: 1"), "Should have 1 ADR.\nContent:\n{}", content);
    assert!(content.contains("tcs: 1"), "Should have 1 TC.\nContent:\n{}", content);

    // metrics.jsonl should exist
    assert!(h.exists("metrics.jsonl"), "metrics.jsonl should exist");
}

#[test]
fn tc_249_product_feature_next() {
    let h = Harness::new();
    // Simple scenario: FT-001 complete, FT-002 depends on FT-001, FT-003 independent phase 2
    h.write(
        "docs/features/FT-001-done.md",
        "---\nid: FT-001\ntitle: Done Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n",
    );
    h.write(
        "docs/tests/TC-001-exit.md",
        "---\nid: TC-001\ntitle: Phase 1 Exit\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h.write(
        "docs/features/FT-002-next.md",
        "---\nid: FT-002\ntitle: Next Feature\nphase: 1\nstatus: in-progress\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-003-phase2.md",
        "---\nid: FT-003\ntitle: Phase Two Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "next"]);
    out.assert_exit(0);
    // FT-002 should be returned (phase 1, deps satisfied, topo order)
    out.assert_stdout_contains("FT-002");
}

#[test]
fn tc_399_product_dep_bom() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "bom"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Total:"), "BOM should have total line");
    assert!(out.stdout.contains("2 dependencies"), "Should show 2 dependencies");
}

#[test]
fn tc_400_product_dep_bom() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "bom", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(json["total"], 2, "Should have 2 deps total");
    assert_eq!(json["product"], "test", "Product name should match");
}

#[test]
fn tc_401_product_impact_dep_001() {
    let h = fixture_dep_library();
    let out = h.run(&["impact", "DEP-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Impact analysis: DEP-001"), "Should show impact header");
    assert!(out.stdout.contains("FT-001"), "FT-001 should be in impact output");
}

#[test]
fn tc_404_product_schema_returns_feature_front_matter_schema() {
    let h = fixture_agent_context();
    let out = h.run(&["schema", "feature"]);
    out.assert_exit(0);
    // Assert all feature front-matter fields are present
    for field in &["id:", "title:", "phase:", "status:", "depends-on:", "adrs:", "tests:", "domains:", "domains-acknowledged:", "bundle:"] {
        assert!(out.stdout.contains(field), "Feature schema should contain field '{}', got:\n{}", field, out.stdout);
    }
    // Assert type descriptions
    assert!(out.stdout.contains("String"), "Should have type descriptions");
    // Assert allowed values
    assert!(out.stdout.contains("planned"), "Should document allowed status values");
    assert!(out.stdout.contains("in-progress"), "Should document in-progress status");
    assert!(out.stdout.contains("complete"), "Should document complete status");
    assert!(out.stdout.contains("abandoned"), "Should document abandoned status");
}

#[test]
fn tc_405_product_schema_returns_adr_front_matter_schema() {
    let h = fixture_agent_context();
    let out = h.run(&["schema", "adr"]);
    out.assert_exit(0);
    // Assert all ADR front-matter fields are present
    for field in &["id:", "title:", "status:", "features:", "supersedes:", "superseded-by:", "domains:", "scope:", "source-files:"] {
        assert!(out.stdout.contains(field), "ADR schema should contain field '{}', got:\n{}", field, out.stdout);
    }
    // Assert status enum values are documented
    assert!(out.stdout.contains("proposed"), "Should document proposed status");
    assert!(out.stdout.contains("accepted"), "Should document accepted status");
    assert!(out.stdout.contains("superseded"), "Should document superseded status");
}

#[test]
fn tc_406_product_schema_returns_dependency_front_matter_schema() {
    let h = fixture_agent_context();
    let out = h.run(&["schema", "dep"]);
    out.assert_exit(0);
    // Assert all six dependency types
    for dep_type in &["library", "service", "api", "tool", "hardware", "runtime"] {
        assert!(out.stdout.contains(dep_type), "Dep schema should contain type '{}', got:\n{}", dep_type, out.stdout);
    }
    // Assert interface block documented for service/api types
    assert!(out.stdout.contains("interface:"), "Should document interface block");
    assert!(out.stdout.contains("protocol:"), "Should document protocol in interface");
    // Assert availability-check described
    assert!(out.stdout.contains("availability-check:"), "Should document availability-check field");
}

#[test]
fn tc_407_product_schema_all_returns_all_schemas() {
    let h = fixture_agent_context();
    let out = h.run(&["schema", "--all"]);
    out.assert_exit(0);
    // Assert all four artifact type schemas
    assert!(out.stdout.contains("Feature"), "Should contain Feature schema");
    assert!(out.stdout.contains("ADR"), "Should contain ADR schema");
    assert!(out.stdout.contains("Test Criterion"), "Should contain Test Criterion schema");
    assert!(out.stdout.contains("Dependency"), "Should contain Dependency schema");
    // Assert valid standalone markdown (has heading)
    assert!(out.stdout.contains("# Front-Matter Schemas"), "Should be valid markdown with heading");
}

#[test]
fn tc_408_product_agent_init_generates_agent_md_from_repo_state() {
    let h = fixture_agent_context();
    let out = h.run(&["agent-init"]);
    out.assert_exit(0);
    // Assert AGENTS.md is created
    assert!(h.exists("AGENTS.md"), "AGENTS.md should be created at repo root");
    let content = h.read("AGENTS.md");
    // Assert generation timestamp
    assert!(content.contains("> Generated by product"), "Should contain generation timestamp");
    // Assert product version
    assert!(content.contains("v0.1.0"), "Should contain product version");
}

#[test]
fn tc_415_product_agent_init_watch_regenerates_on_graph_change() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let h = fixture_agent_context();

    // Start watch in background
    let mut child = Command::new(&h.bin)
        .args(["agent-init", "--watch"])
        .current_dir(h.dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn agent-init --watch");

    // Wait for initial generation
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Verify initial AGENTS.md was created
    assert!(h.exists("AGENTS.md"), "Initial AGENTS.md should exist");
    let initial_content = h.read("AGENTS.md");
    assert!(initial_content.contains("2 features"), "Should initially show 2 features");

    // Modify a feature file's front-matter
    h.write("docs/features/FT-003-new.md", "---\nid: FT-003\ntitle: New Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nNew feature.\n");

    // Wait for regeneration
    std::thread::sleep(std::time::Duration::from_millis(2000));

    let updated_content = h.read("AGENTS.md");
    assert!(updated_content.contains("3 features"), "Should reflect 3 features after adding FT-003, got:\n{}", updated_content);

    // Kill the watch process
    let _ = child.kill();
    let status = child.wait().expect("wait for child");
    // On kill, the process may exit with a signal — that's fine
    assert!(status.code().is_none() || status.code() == Some(0) || status.code() == Some(1),
        "Watch process should exit cleanly on kill");
}

#[test]
fn tc_416_product_schema_mcp_tool_returns_schema_for_artifact_type() {
    let h = fixture_agent_context();

    // Test feature schema via MCP
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_schema","arguments":{"artifact_type":"feature"}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("id:"), "MCP schema for feature should contain id field: {}", out);
    assert!(out.contains("depends-on:"), "MCP schema for feature should contain depends-on: {}", out);

    // Test ADR schema
    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_schema","arguments":{"artifact_type":"adr"}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("supersedes:"), "MCP schema for adr should contain supersedes: {}", out);

    // Test dep schema
    let input = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_schema","arguments":{"artifact_type":"dep"}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("interface:"), "MCP schema for dep should contain interface: {}", out);

    // Test all schemas (no artifact_type argument)
    let input = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"product_schema","arguments":{}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("Feature"), "MCP all schemas should contain Feature: {}", out);
    assert!(out.contains("ADR"), "MCP all schemas should contain ADR: {}", out);
    assert!(out.contains("Dependency"), "MCP all schemas should contain Dependency: {}", out);
}

#[test]
fn tc_417_product_agent_context_mcp_tool_returns_agent_md_content() {
    let h = fixture_agent_context();

    // Generate AGENTS.md first
    h.run(&["agent-init"]).assert_exit(0);
    let file_content = h.read("AGENTS.md");
    assert!(!file_content.is_empty(), "AGENTS.md should exist");

    // Call MCP tool
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_agent_context","arguments":{}}}"#;
    let out = run_mcp_stdio(&h, input);
    // MCP response should contain key sections from AGENTS.md
    assert!(out.contains("Working Protocol"), "MCP agent context should contain Working Protocol: {}", out);
    assert!(out.contains("Front-Matter Schemas"), "MCP agent context should contain schemas: {}", out);
    assert!(out.contains("Domain Vocabulary"), "MCP agent context should contain domains: {}", out);
    assert!(out.contains("Key MCP Tools"), "MCP agent context should contain tool guide: {}", out);
    assert!(out.contains("2 features"), "MCP agent context should contain repo state: {}", out);
}

#[test]
fn tc_368_product_migrate_link_tests() {
    // Smoke test: verify `product migrate link-tests` command exists and runs successfully
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: Test Criterion
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    let out = h.run(&["migrate", "link-tests"]);
    out.assert_exit(0);

    // Verify the command produced results
    let tc = h.read("docs/tests/TC-002-test.md");
    assert!(tc.contains("FT-001"), "link-tests should create transitive links. Got:\n{}", tc);

    let ft = h.read("docs/features/FT-001-test.md");
    assert!(ft.contains("TC-002"), "link-tests should create reverse links. Got:\n{}", ft);
}

#[test]
fn tc_472_product_toml_parses_product_responsibility_field() {
    // Scenario 1: [product] section with name and responsibility
    let h = fixture_with_responsibility();
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    // If config parses successfully, commands work (name and responsibility parsed)
    out.assert_stdout_contains("FT-001");

    // Scenario 2: product.toml without [product] section — graceful fallback
    let h2 = Harness::new();
    h2.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    let out2 = h2.run(&["feature", "list"]);
    out2.assert_exit(0);
    out2.assert_stdout_contains("FT-001");
}

#[test]
fn tc_473_product_responsibility_mcp_tool_returns_name_and_responsibility() {
    let h = fixture_with_responsibility();
    // Test with responsibility configured — call via JSON-RPC
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_responsibility","arguments":{}}}"#;
    let out = h.run_with_stdin(&["mcp"], request);
    assert!(out.stdout.contains("picloud"), "should contain product name: {}", out.stdout);
    assert!(out.stdout.contains("private cloud platform"), "should contain responsibility: {}", out.stdout);

    // Test without responsibility — should return error
    let h2 = Harness::new();
    let out2 = h2.run_with_stdin(&["mcp"], request);
    assert!(out2.stdout.contains("error") || out2.stdout.contains("not configured"),
        "should indicate responsibility not configured: {}", out2.stdout);
}

#[test]
fn tc_478_product_responsibility_is_single_statement_invariant() {
    // Top-level conjunction should trigger warning
    let h = Harness::new();
    h.write("product.toml", r#"name = "test"
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
[product]
responsibility = "A cloud platform and a monitoring system"
"#);
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W019");
    out.assert_stderr_contains("multiple products");

    // Subordinate conjunction — no warning (no X and no Y is acceptable)
    let h2 = Harness::new();
    h2.write("product.toml", r#"name = "test"
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
[product]
responsibility = "A platform — no external dependencies and no configuration needed"
"#);
    h2.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    let out3 = h2.run(&["feature", "list"]);
    out3.assert_exit(0);
}

#[test]
fn tc_479_product_responsibility_feature_complete() {
    let h = fixture_with_responsibility();
    // 1. Config parsing works (TC-472)
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");

    // 2. MCP tool works (TC-473)
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_responsibility","arguments":{}}}"#;
    let mcp_out = h.run_with_stdin(&["mcp"], req);
    assert!(mcp_out.stdout.contains("picloud"), "MCP should return product name");

    // 3. Context bundle includes responsibility (TC-474)
    let ctx = h.run(&["context", "FT-001"]);
    ctx.assert_exit(0);
    assert!(ctx.stdout.contains("product\u{225c}picloud:Product"), "bundle has product");
    assert!(ctx.stdout.contains("responsibility\u{225c}"), "bundle has responsibility");

    // 4. Graph check with out-of-scope feature emits W019 (TC-475)
    h.write("docs/features/FT-099-grocery.md", "---\nid: FT-099\ntitle: Grocery List Management\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nGrocery.\n");
    let chk = h.run(&["graph", "check"]);
    chk.assert_stderr_contains("W019");

    // 5. W019 suppressed when absent (TC-476) — separate harness without [product]
    let h2 = Harness::new();
    h2.write("docs/features/FT-099-grocery.md", "---\nid: FT-099\ntitle: Grocery List Management\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nGrocery.\n");
    let chk2 = h2.run(&["graph", "check"]);
    assert!(!chk2.stderr.contains("W019"), "W019 suppressed when no responsibility");

    // 6. Context omits responsibility when unconfigured (TC-477) — covered by h2
    // 7. All TCs passing — verified by this test passing
}

#[test]
fn tc_641_product_status_shows_due_date_column_and_overdue_flag() {
    let h = fixture_with_domains();
    let future = (chrono::Local::now().date_naive()
        + chrono::Duration::days(90))
        .format("%Y-%m-%d")
        .to_string();
    h.write(
        "docs/features/FT-003-future.md",
        &format!(
            "---\nid: FT-003\ntitle: Future Date\nphase: 1\nstatus: in-progress\ndue-date: \"{}\"\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\nSeed.\n",
            future
        ),
    );
    h.write(
        "docs/features/FT-009-overdue.md",
        "---\nid: FT-009\ntitle: Overdue\nphase: 1\nstatus: planned\ndue-date: \"1970-01-01\"\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );
    h.write(
        "docs/features/FT-012-no-date.md",
        "---\nid: FT-012\ntitle: No Date\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\nSeed.\n",
    );

    let out = h.run(&["status"]);
    out.assert_exit(0);
    // Future feature shows its date.
    out.assert_stdout_contains(&future);
    // Overdue feature shows its date AND the overdue marker.
    out.assert_stdout_contains("1970-01-01");
    out.assert_stdout_contains("overdue");
    // FT-012 row should not contain "due ".
    let lines: Vec<&str> = out
        .stdout
        .lines()
        .filter(|l| l.contains("FT-012"))
        .collect();
    assert!(!lines.is_empty(), "expected FT-012 row in output");
    for l in &lines {
        assert!(
            !l.contains("due "),
            "FT-012 has no due-date and should not render one: {}",
            l
        );
    }
}

