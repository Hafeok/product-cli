//! Integration tests — agent_context.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_409_agent_md_contains_current_front_matter_schemas() {
    let h = fixture_agent_context();
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    // Assert schemas section exists
    assert!(content.contains("## Front-Matter Schemas"), "Should have Front-Matter Schemas section");
    // Assert subsections
    assert!(content.contains("### Feature"), "Should have Feature schema subsection");
    assert!(content.contains("### ADR"), "Should have ADR schema subsection");
    assert!(content.contains("### Test Criterion"), "Should have Test Criterion schema subsection");
    assert!(content.contains("### Dependency"), "Should have Dependency schema subsection");
    // Schema content should match `product schema --all`
    let schema_out = h.run(&["schema", "--all"]);
    schema_out.assert_exit(0);
    // Check key fields appear in both
    assert!(content.contains("depends-on:"), "AGENTS.md schema should contain depends-on field");
    assert!(content.contains("supersedes:"), "AGENTS.md schema should contain supersedes field");
}

#[test]
fn tc_410_agent_md_contains_working_protocol_section() {
    let h = fixture_agent_context();
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Working Protocol"), "Should have Working Protocol section");
    assert!(content.contains("product_graph_check"), "Should mention product_graph_check");
    assert!(content.contains("product_graph_central"), "Should mention product_graph_central");
    assert!(content.contains("product_feature_list"), "Should mention product_feature_list");
    assert!(content.contains("product_context"), "Should mention product_context");
}

#[test]
fn tc_411_agent_md_contains_current_repository_state_summary() {
    let h = fixture_agent_context();
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Current Repository State"), "Should have Current Repository State section");
    // Should show correct feature count (2)
    assert!(content.contains("2 features"), "Should show 2 features, got:\n{}", content);
    // Should show correct ADR count (2)
    assert!(content.contains("2 ADRs"), "Should show 2 ADRs, got:\n{}", content);
    // Should show TC counts
    assert!(content.contains("3 test criteria"), "Should show 3 test criteria, got:\n{}", content);
    assert!(content.contains("1 passing"), "Should show 1 passing, got:\n{}", content);
    assert!(content.contains("1 failing"), "Should show 1 failing, got:\n{}", content);
    assert!(content.contains("1 unimplemented"), "Should show 1 unimplemented, got:\n{}", content);
    // Should include phase gate status
    assert!(content.contains("Phase 1"), "Should include phase gate info, got:\n{}", content);
}

#[test]
fn tc_412_agent_md_contains_domain_vocabulary_from_product_toml() {
    let h = fixture_agent_context();
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Domain Vocabulary"), "Should have Domain Vocabulary section");
    assert!(content.contains("security"), "Should list security domain");
    assert!(content.contains("storage"), "Should list storage domain");
    assert!(content.contains("networking"), "Should list networking domain");

    // Add a new domain and re-run
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
[domains]
security = "Authentication and authorization"
storage = "Data persistence"
networking = "Network protocols"
observability = "Monitoring and logging"
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content2 = h.read("AGENTS.md");
    assert!(content2.contains("observability"), "Should list newly added observability domain");
}

#[test]
fn tc_413_agent_md_contains_mcp_tool_usage_guide() {
    let h = fixture_agent_context();
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Key MCP Tools"), "Should have Key MCP Tools section");
    // Check required tools are listed
    assert!(content.contains("product_context"), "Should list product_context");
    assert!(content.contains("product_schema"), "Should list product_schema");
    assert!(content.contains("product_graph_central"), "Should list product_graph_central");
    assert!(content.contains("product_preflight"), "Should list product_preflight");
    assert!(content.contains("product_gap_check"), "Should list product_gap_check");
    assert!(content.contains("product_agent_context"), "Should list product_agent_context");
}

#[test]
fn tc_414_agent_md_is_regenerated_not_hand_edited() {
    let h = fixture_agent_context();
    // First generation
    h.run(&["agent-init"]).assert_exit(0);
    let content1 = h.read("AGENTS.md");
    assert!(!content1.is_empty(), "First generation should produce content");

    // Second generation overwrites cleanly
    h.run(&["agent-init"]).assert_exit(0);
    let content2 = h.read("AGENTS.md");
    // Both should contain the timestamp line (may differ by ms)
    assert!(content2.contains("> Generated by product"), "Second gen should have timestamp");

    // Hand-edit AGENTS.md by inserting a marker line
    let edited = format!("HAND-EDITED-MARKER\n{}", content2);
    h.write("AGENTS.md", &edited);
    assert!(h.read("AGENTS.md").contains("HAND-EDITED-MARKER"), "Marker should be present");

    // Re-run — marker should be gone
    h.run(&["agent-init"]).assert_exit(0);
    let content3 = h.read("AGENTS.md");
    assert!(!content3.contains("HAND-EDITED-MARKER"), "Hand-edit marker should be gone after regeneration");
    assert!(content3.contains("> Generated by product"), "Regenerated file should have timestamp");
}

#[test]
fn tc_418_agent_context_config_controls_agent_md_sections() {
    let h = fixture_agent_context();

    // Disable schemas
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
[domains]
security = "Auth"
[agent-context]
include-schemas = false
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(!content.contains("## Front-Matter Schemas"), "Schemas section should be absent when disabled");
    assert!(content.contains("## Working Protocol"), "Protocol section should still be present");
    assert!(content.contains("## Current Repository State"), "Repo state should still be present");

    // Re-enable schemas
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
[domains]
security = "Auth"
[agent-context]
include-schemas = true
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Front-Matter Schemas"), "Schemas section should reappear when enabled");

    // Disable repo-state
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
[domains]
security = "Auth"
[agent-context]
include-repo-state = false
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(!content.contains("## Current Repository State"), "Repo state section should be absent when disabled");

    // Disable domains
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
[domains]
security = "Auth"
[agent-context]
include-domains = false
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(!content.contains("## Domain Vocabulary"), "Domain section should be absent when disabled");

    // Disable tool guide
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
[domains]
security = "Auth"
[agent-context]
include-tool-guide = false
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(!content.contains("## Key MCP Tools"), "Tool guide section should be absent when disabled");
}

#[test]
fn tc_419_agent_context_generation_exit_criteria() {
    let h = fixture_agent_context();

    // 1. product schema --all contains all four schemas
    let schema_out = h.run(&["schema", "--all"]);
    schema_out.assert_exit(0);
    assert!(schema_out.stdout.contains("Feature"), "All schemas should contain Feature");
    assert!(schema_out.stdout.contains("ADR"), "All schemas should contain ADR");
    assert!(schema_out.stdout.contains("Test Criterion"), "All schemas should contain Test Criterion");
    assert!(schema_out.stdout.contains("Dependency"), "All schemas should contain Dependency");

    // 2. product agent-init creates AGENTS.md with all five sections
    h.run(&["agent-init"]).assert_exit(0);
    let content = h.read("AGENTS.md");
    assert!(content.contains("## Working Protocol"), "Should have protocol section");
    assert!(content.contains("## Current Repository State"), "Should have repo state section");
    assert!(content.contains("## Front-Matter Schemas"), "Should have schemas section");
    assert!(content.contains("## Domain Vocabulary"), "Should have domains section");
    assert!(content.contains("## Key MCP Tools"), "Should have tool guide section");

    // 3. Modify a feature status, re-run — repo state changes
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.run(&["agent-init"]).assert_exit(0);
    let content2 = h.read("AGENTS.md");
    // FT-001 and FT-002 are both complete now
    assert!(content2.contains("2/2 complete"), "Should reflect updated completion status, got:\n{}", content2);

    // 4. MCP tools work
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_schema","arguments":{"artifact_type":"feature"}}}"#;
    let mcp_out = run_mcp_stdio(&h, input);
    assert!(mcp_out.contains("id:"), "MCP schema should work");

    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_agent_context","arguments":{}}}"#;
    let mcp_out = run_mcp_stdio(&h, input);
    assert!(mcp_out.contains("Working Protocol"), "MCP agent context should work");

    // 5. Config toggle works
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
[domains]
security = "Authentication and authorization"
storage = "Data persistence"
networking = "Network protocols"
[agent-context]
include-schemas = false
"#);
    h.run(&["agent-init"]).assert_exit(0);
    let content3 = h.read("AGENTS.md");
    assert!(!content3.contains("## Front-Matter Schemas"), "Schemas should be absent when disabled");
}

#[test]
fn tc_605_custom_type_valid_when_in_toml() {
    let h = ft048_tc_types(&["contract"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001"]);
    ft048_write_tc(&h, "TC-001", "Ct", "contract", "passing", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(!out.stderr.contains("E006"), "no E006 expected. stderr: {}", out.stderr);
    let bundle = h.run(&["context", "FT-001"]);
    bundle.assert_stdout_contains("TC-001");
}

#[test]
fn tc_606_custom_type_e006_when_not_in_toml() {
    let h = ft048_tc_types(&["contract"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001"]);
    ft048_write_tc(&h, "TC-001", "Smk", "smoke", "passing", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E006");
    out.assert_stderr_contains("smoke");
    out.assert_stderr_contains("contract");
}

#[test]
fn tc_607_custom_type_treated_as_scenario_in_mechanics() {
    let h = ft048_tc_types(&["contract"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001", "TC-002"]);
    ft048_write_tc(&h, "TC-001", "Sc", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-002", "Ct", "contract", "passing", "FT-001", 1);
    let out = h.run(&["graph", "check"]);
    assert!(!out.stderr.contains("W004"), "custom must not trigger W004");
    let bundle = h.run(&["context", "FT-001"]);
    bundle.assert_stdout_contains("TC-001");
    bundle.assert_stdout_contains("TC-002");
}

#[test]
fn tc_608_custom_type_appears_in_agent_md_schema() {
    let h = ft048_tc_types(&["contract", "migration", "smoke"]);
    let out = h.run(&["schema", "test"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("contract") || out.stdout.contains("smoke"),
        "custom types must appear in schema. stdout: {}",
        out.stdout
    );
    out.assert_stdout_contains("exit-criteria");
    out.assert_stdout_contains("absence");
}

#[test]
fn tc_609_custom_type_appears_in_context_bundle_after_builtins() {
    let h = ft048_tc_types(&["contract"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001", "TC-002", "TC-003", "TC-004", "TC-005"]);
    ft048_write_tc(&h, "TC-001", "X", "exit-criteria", "passing", "FT-001", 1);
    h.write(
        "docs/tests/TC-002.md",
        "---\nid: TC-002\ntitle: Inv\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ x }\n",
    );
    h.write(
        "docs/tests/TC-003.md",
        "---\nid: TC-003\ntitle: Ch\ntype: chaos\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ y }\n",
    );
    ft048_write_tc(&h, "TC-004", "Sc", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-005", "Co", "contract", "passing", "FT-001", 1);
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    let p1 = out.stdout.find("TC-001").expect("TC-001");
    let p4 = out.stdout.find("TC-004").expect("TC-004");
    let p5 = out.stdout.find("TC-005").expect("TC-005");
    assert!(p1 < p5 && p4 < p5, "custom TC-005 (contract) must come last");
}

