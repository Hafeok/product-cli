//! Integration tests — dep.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_381_dep_parse_library() {
    let h = fixture_dep_library();
    let out = h.run(&["dep", "show", "DEP-001", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(json["id"], "DEP-001");
    assert_eq!(json["title"], "openraft");
    assert_eq!(json["type"], "library");
    assert_eq!(json["version"], ">=0.9,<1.0");
    assert_eq!(json["status"], "active");
    assert!(json["availability-check"].is_null(), "availability-check should be null for library");
}

#[test]
fn tc_382_dep_parse_service() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "show", "DEP-005", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(json["type"], "service");
    let iface = &json["interface"];
    assert_eq!(iface["protocol"], "tcp");
    assert_eq!(iface["port"], 5432);
    assert_eq!(iface["auth"], "md5");
    assert_eq!(iface["connection-string-env"], "DATABASE_URL");
}

#[test]
fn tc_383_dep_uses_edge() {
    let h = fixture_dep_library();
    let out = h.run(&["impact", "DEP-001", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let direct_features = json["direct_features"].as_array().expect("array");
    assert!(direct_features.iter().any(|v| v == "FT-001"), "FT-001 should be a direct dependent of DEP-001");
}

#[test]
fn tc_384_dep_governs_edge() {
    let h = fixture_dep_library();
    let out = h.run(&["impact", "DEP-001", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let direct_adrs = json["direct_adrs"].as_array().expect("array");
    assert!(direct_adrs.iter().any(|v| v == "ADR-002"), "ADR-002 should govern DEP-001");
}

#[test]
fn tc_385_dep_impact_direct() {
    let h = fixture_dep_service();
    // DEP-001 linked to FT-001; also DEP-005 linked to FT-007
    // Add FT-002 using DEP-001
    h.write("docs/features/FT-002-test2.md", "---\nid: FT-002\ntitle: Test2\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h.write("docs/dependencies/DEP-001-openraft.md", "---\nid: DEP-001\ntitle: openraft\ntype: library\nstatus: active\nfeatures: [FT-001, FT-002]\nadrs: [ADR-002]\navailability-check: ~\nbreaking-change-risk: medium\n---\n\nLib.\n");
    let out = h.run(&["impact", "DEP-001", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let direct = json["direct_features"].as_array().expect("array");
    let ids: Vec<&str> = direct.iter().filter_map(|v| v.as_str()).collect();
    assert!(ids.contains(&"FT-001"), "FT-001 should be direct dependent");
    assert!(ids.contains(&"FT-002"), "FT-002 should be direct dependent");
}

#[test]
fn tc_386_dep_impact_transitive() {
    let h = fixture_dep_library();
    // FT-003 depends-on FT-001, FT-001 uses DEP-001
    h.write("docs/features/FT-003-child.md", "---\nid: FT-003\ntitle: Child\nphase: 1\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n");
    let out = h.run(&["impact", "DEP-001", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let transitive = json["transitive_features"].as_array().expect("array");
    assert!(transitive.iter().any(|v| v == "FT-003"), "FT-003 should be transitive dependent of DEP-001");
}

#[test]
fn tc_387_dep_preflight_check_passes() {
    let h = fixture_dep_service();
    let out = h.run(&["preflight", "FT-007"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("DEP-005"), "DEP-005 should appear in preflight output");
    assert!(out.stdout.contains("\u{2713}"), "Check should show pass mark");
}

#[test]
fn tc_388_dep_preflight_check_fails() {
    let h = fixture_dep_service();
    // Overwrite DEP-005 with a failing availability check
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL Event Store\ntype: service\nversion: \">=14\"\nstatus: active\nfeatures: [FT-007]\nadrs: [ADR-015]\navailability-check: \"false\"\nbreaking-change-risk: low\ninterface:\n  protocol: tcp\n  port: 5432\n  auth: md5\n  connection-string-env: DATABASE_URL\n---\n\nPostgreSQL for events.\n");
    let out = h.run(&["preflight", "FT-007"]);
    out.assert_exit(2);
    assert!(out.stdout.contains("DEP-005"), "DEP-005 should appear");
    assert!(out.stdout.contains("not running") || out.stdout.contains("FAILED"), "Should show unavailable");
}

#[test]
fn tc_389_dep_tc_requires_dep_id() {
    // This test verifies at unit level that the DEP ID resolves to the check command.
    // The integration approach: check that the graph has the dependency with its check command
    let h = fixture_dep_service();
    h.write("docs/tests/TC-042-event-persist.md", "---\nid: TC-042\ntitle: Event Persistence\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-007]\n  adrs: []\nphase: 1\nrequires: [DEP-005]\n---\n\nTest body.\n");
    let out = h.run(&["dep", "show", "DEP-005", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(json["availability-check"], "true", "DEP-005 availability-check should be resolvable");
}

#[test]
fn tc_390_dep_context_bundle_section() {
    let h = fixture_dep_service();
    // FT-007 uses DEP-005 (service); also link DEP-001 to FT-007
    h.write("docs/dependencies/DEP-001-openraft.md", "---\nid: DEP-001\ntitle: openraft\ntype: library\nstatus: active\nfeatures: [FT-001, FT-007]\nadrs: [ADR-002]\navailability-check: ~\nbreaking-change-risk: medium\n---\n\nLib.\n");
    let out = h.run(&["context", "FT-007", "--depth", "2"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("## Dependencies"), "Bundle should contain Dependencies section");
    assert!(out.stdout.contains("DEP-005"), "DEP-005 should be in bundle");
    assert!(out.stdout.contains("protocol: tcp"), "Interface block should be in bundle for DEP-005");
}

#[test]
fn tc_391_dep_bom_output() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "bom"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Libraries (build-time)"), "BOM should group by type");
    assert!(out.stdout.contains("Services (runtime)"), "BOM should have service section");
    assert!(out.stdout.contains("DEP-001"), "DEP-001 should be listed");
    assert!(out.stdout.contains("DEP-005"), "DEP-005 should be listed");
    // JSON variant
    let out_json = h.run(&["dep", "bom", "--format", "json"]);
    out_json.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out_json.stdout).expect("valid JSON");
    assert!(json["dependencies"].is_array(), "JSON BOM should have dependencies array");
}

#[test]
fn tc_392_dep_bom_json_schema() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "bom", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    let deps = json["dependencies"].as_array().expect("deps array");
    for dep in deps {
        assert!(dep["id"].is_string(), "Each dep should have id");
        assert!(dep["title"].is_string(), "Each dep should have title");
        assert!(dep["type"].is_string(), "Each dep should have type");
        assert!(dep["status"].is_string(), "Each dep should have status");
        assert!(dep["features"].is_array(), "Each dep should have features list");
        assert!(dep["breaking-change-risk"].is_string(), "Each dep should have breaking-change-risk");
    }
}

#[test]
fn tc_393_dep_w013_deprecated() {
    let h = fixture_dep_service();
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL Event Store\ntype: service\nversion: \">=14\"\nstatus: deprecated\nfeatures: [FT-007]\nadrs: [ADR-015]\navailability-check: \"true\"\nbreaking-change-risk: low\n---\n\nDeprecated.\n");
    let out = h.run(&["graph", "check"]);
    out.assert_exit(2).assert_stderr_contains("W013");
    assert!(out.stderr.contains("FT-007"), "W013 should name FT-007");
    assert!(out.stderr.contains("DEP-005"), "W013 should name DEP-005");
}

#[test]
fn tc_394_dep_e013_no_adr() {
    let h = Harness::new();
    h.write("docs/features/FT-007-events.md", "---\nid: FT-007\ntitle: Events\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL\ntype: service\nstatus: active\nfeatures: [FT-007]\nadrs: []\navailability-check: ~\nbreaking-change-risk: low\n---\n\nNo ADR.\n");
    let out = h.run(&["graph", "check"]);
    out.assert_exit(1).assert_stderr_contains("E013");
    assert!(out.stderr.contains("DEP-005"), "E013 should name DEP-005");
    assert!(out.stderr.contains("every dependency requires a governing decision"), "E013 should have correct message");
}

#[test]
fn tc_395_dep_gap_g008() {
    let h = Harness::new();
    h.write("docs/features/FT-007-events.md", "---\nid: FT-007\ntitle: Events\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL\ntype: service\nstatus: active\nfeatures: [FT-007]\nadrs: []\navailability-check: ~\nbreaking-change-risk: low\n---\n\nNo ADR governs.\n");
    let out = h.run(&["gap", "check", "FT-007"]);
    assert!(out.stdout.contains("G008"), "Should contain G008 finding");
}

#[test]
fn tc_396_dep_list_filter() {
    let h = fixture_dep_service();
    let out = h.run(&["dep", "list", "--type", "service"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("DEP-005"), "DEP-005 (service) should be listed");
    assert!(!out.stdout.contains("DEP-001"), "DEP-001 (library) should NOT be listed");
}

#[test]
fn tc_397_dep_check_manual() {
    let h = fixture_dep_service();
    // Check pass
    let out = h.run(&["dep", "check", "DEP-005"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("check passed") || out.stdout.contains("\u{2713}"), "check should pass");
    // Check fail
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL Event Store\ntype: service\nversion: \">=14\"\nstatus: active\nfeatures: [FT-007]\nadrs: [ADR-015]\navailability-check: \"false\"\nbreaking-change-risk: low\n---\n\nPostgreSQL.\n");
    let out2 = h.run(&["dep", "check", "DEP-005"]);
    out2.assert_exit(2);
    assert!(out2.stdout.contains("FAILED") || out2.stdout.contains("\u{2717}"), "check should fail");
}

#[test]
fn tc_398_dep_supersedes_edge() {
    let h = fixture_dep_service();
    h.write("docs/adrs/ADR-020-new-db.md", "---\nid: ADR-020\ntitle: New DB\nstatus: accepted\nfeatures: []\n---\n\nDecision.\n");
    h.write("docs/dependencies/DEP-011-newdb.md", "---\nid: DEP-011\ntitle: New Database\ntype: service\nstatus: active\nfeatures: []\nadrs: [ADR-020]\nsupersedes: [DEP-005]\navailability-check: ~\nbreaking-change-risk: low\n---\n\nReplacement.\n");
    let out = h.run(&["impact", "DEP-005", "--format", "json"]);
    out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    // DEP-011 supersedes DEP-005, so DEP-011 should appear as a dependent
    let direct_deps = json["direct_deps"].as_array().expect("array");
    assert!(direct_deps.iter().any(|v| v == "DEP-011"), "DEP-011 should be in dependents of DEP-005 via supersedes edge");
}

#[test]
fn tc_403_dependency_bom_and_impact_analysis_produce_correct_output() {
    let h = fixture_dep_service();
    // BOM produces correct type groupings
    let bom_out = h.run(&["dep", "bom"]);
    bom_out.assert_exit(0);
    assert!(bom_out.stdout.contains("Libraries"), "BOM groups libraries");
    assert!(bom_out.stdout.contains("Services"), "BOM groups services");
    // Impact DEP-001 returns features
    let impact_out = h.run(&["impact", "DEP-001", "--format", "json"]);
    impact_out.assert_exit(0);
    let json: serde_json::Value = serde_json::from_str(&impact_out.stdout).expect("valid JSON");
    assert!(!json["direct_features"].as_array().expect("array").is_empty(), "DEP-001 should have feature dependents");
    // TC requires resolution: DEP-005 has availability-check field
    let dep_out = h.run(&["dep", "show", "DEP-005", "--format", "json"]);
    dep_out.assert_exit(0);
    let dep_json: serde_json::Value = serde_json::from_str(&dep_out.stdout).expect("valid JSON");
    assert!(dep_json["availability-check"].is_string(), "DEP-005 should have resolvable availability-check");
}

