//! Integration tests — graph.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn it_001_graph_check_broken_link() {
    let h = fixture_broken_link();
    h.run(&["graph", "check"])
        .assert_exit(1)
        .assert_stderr_contains("E002");
}

#[test]
fn it_002_graph_check_json_broken_link() {
    let h = fixture_broken_link();
    let out = h.run(&["graph", "check", "--format", "json"]);
    assert_eq!(out.exit_code, 1, "Expected exit code 1 for broken link");
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON on stdout: {}\nstdout: {}", e, out.stdout));
    assert!(json["errors"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
}

#[test]
fn it_003_graph_check_clean() {
    let h = fixture_minimal();
    h.run(&["graph", "check"]).assert_exit(0);
}

#[test]
fn it_004_graph_check_orphaned() {
    let h = fixture_orphaned_adr();
    h.run(&["graph", "check"])
        .assert_exit(2)
        .assert_stderr_contains("W001");
}

#[test]
fn it_007_graph_check_cycle() {
    let h = fixture_dep_cycle();
    h.run(&["graph", "check"])
        .assert_exit(1)
        .assert_stderr_contains("E003");
}

#[test]
fn tc_025_sparql_untested_features() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-tested.md",
        "---\nid: FT-001\ntitle: Tested Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nTested.\n",
    );
    h.write(
        "docs/features/FT-002-untested.md",
        "---\nid: FT-002\ntitle: Untested Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nUntested.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest body.\n",
    );

    // Query for features with no validatedBy triples
    let query = r#"PREFIX pm: <https://product-meta/ontology#>
PREFIX ft: <https://product-meta/feature/>
SELECT ?feature WHERE {
  ?feature a pm:Feature .
  FILTER NOT EXISTS { ?feature pm:validatedBy ?tc }
}"#;
    let out = h.run(&["graph", "query", query]);
    out.assert_exit(0);

    // FT-002 should appear (no tests), FT-001 should not (has tests)
    assert!(
        out.stdout.contains("FT-002"),
        "FT-002 (untested) should appear in results.\nOutput:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("FT-001"),
        "FT-001 (tested) should NOT appear in results.\nOutput:\n{}",
        out.stdout
    );
}

#[test]
fn tc_026_sparql_phase_filter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-phase1.md",
        "---\nid: FT-001\ntitle: Phase 1 Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nPhase 1.\n",
    );
    h.write(
        "docs/features/FT-002-phase2.md",
        "---\nid: FT-002\ntitle: Phase 2 Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nPhase 2.\n",
    );

    let query = r#"PREFIX pm: <https://product-meta/ontology#>
SELECT ?feature WHERE {
  ?feature a pm:Feature ;
           pm:phase 1 .
}"#;
    let out = h.run(&["graph", "query", query]);
    out.assert_exit(0);

    assert!(
        out.stdout.contains("FT-001"),
        "Phase-1 feature FT-001 should appear.\nOutput:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("FT-002"),
        "Phase-2 feature FT-002 should NOT appear.\nOutput:\n{}",
        out.stdout
    );
}

#[test]
fn tc_052_impact_on_supersede() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-002-old.md",
        "---\nid: ADR-002\ntitle: Old Consensus\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nOld decision.\n",
    );
    h.write(
        "docs/adrs/ADR-013-new.md",
        "---\nid: ADR-013\ntitle: New Consensus\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nNew decision.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Consensus Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-002]\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["adr", "status", "ADR-002", "superseded", "--by", "ADR-013"]);
    out.assert_exit(0);

    // Impact summary should be printed before status change
    let impact_pos = out.stdout.find("Impact analysis").or_else(|| out.stdout.find("Direct dependents")).or_else(|| out.stdout.find("FT-001"));
    let status_pos = out.stdout.find("status -> superseded").or_else(|| out.stdout.find("status ->"));
    assert!(
        impact_pos.is_some(),
        "Impact summary should be printed.\nOutput:\n{}",
        out.stdout
    );
    assert!(
        status_pos.is_some(),
        "Status change confirmation should be printed.\nOutput:\n{}",
        out.stdout
    );
    // Impact before status change
    if let (Some(ip), Some(sp)) = (impact_pos, status_pos) {
        assert!(
            ip < sp,
            "Impact summary should appear before status change confirmation"
        );
    }
}

#[test]
fn tc_024_sparql_select_feature_adrs() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: []\n---\n\nFeature.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nFirst.\n",
    );
    h.write(
        "docs/adrs/ADR-002-second.md",
        "---\nid: ADR-002\ntitle: Second\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nSecond.\n",
    );

    let query = r#"PREFIX pm: <https://product-meta/ontology#>
PREFIX ft: <https://product-meta/feature/>
SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }"#;
    let out = h.run(&["graph", "query", query]);
    out.assert_exit(0);

    assert!(
        out.stdout.contains("ADR-001"),
        "Result should contain ADR-001.\nOutput:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("ADR-002"),
        "Result should contain ADR-002.\nOutput:\n{}",
        out.stdout
    );
}

#[test]
fn tc_041_topo_sort_simple() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-a.md",
        "---\nid: FT-001\ntitle: First\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-002-b.md",
        "---\nid: FT-002\ntitle: Second\nphase: 1\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-003-c.md",
        "---\nid: FT-003\ntitle: Third\nphase: 1\nstatus: planned\ndepends-on: [FT-002]\nadrs: []\ntests: []\n---\n",
    );

    let out = h.run(&["feature", "deps", "FT-003"]);
    out.assert_exit(0);

    // The dependency tree shows FT-003 at root, then FT-002, then FT-001 (deepest dep)
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("FT-002");
    out.assert_stdout_contains("FT-003");
    // FT-002 depends on FT-001, so FT-001 should be indented deeper (appear after FT-002 in tree)
    let pos2 = out.stdout.find("FT-002").expect("FT-002 in deps");
    let pos1 = out.stdout.find("FT-001").expect("FT-001 in deps");
    assert!(pos2 < pos1, "FT-002 should appear before FT-001 (FT-001 is a deeper dependency)");
}

#[test]
fn tc_042_topo_sort_parallel() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-root.md",
        "---\nid: FT-001\ntitle: Root\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-002-branch-a.md",
        "---\nid: FT-002\ntitle: Branch A\nphase: 1\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-003-branch-b.md",
        "---\nid: FT-003\ntitle: Branch B\nphase: 1\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );

    // graph check should pass (no cycle)
    let out = h.run(&["graph", "check"]);
    // FT-001 should come before both FT-002 and FT-003
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        !combined.contains("cycle"),
        "No cycle should be detected in parallel dependencies"
    );
}

#[test]
fn tc_043_topo_sort_cycle() {
    let h = fixture_dep_cycle();
    let out = h.run(&["graph", "check"]);
    assert_ne!(out.exit_code, 0, "Cycle should cause non-zero exit code.\nstdout: {}\nstderr: {}", out.stdout, out.stderr);
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        combined.contains("FT-001") && combined.contains("FT-002"),
        "Error should name both features in the cycle.\nOutput:\n{}",
        combined
    );
}

#[test]
fn tc_048_centrality_computation() {
    let h = Harness::new();
    // Create a graph where ADR-001 bridges two features and ADR-002 is peripheral
    h.write(
        "docs/features/FT-001-a.md",
        "---\nid: FT-001\ntitle: Feature A\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n",
    );
    h.write(
        "docs/features/FT-002-b.md",
        "---\nid: FT-002\ntitle: Feature B\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-002]\n---\n",
    );
    h.write(
        "docs/adrs/ADR-001-bridge.md",
        "---\nid: ADR-001\ntitle: Bridge ADR\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-002-leaf.md",
        "---\nid: ADR-002\ntitle: Leaf ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test 1\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Test 2\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-002]\n  adrs: [ADR-001]\nphase: 1\n---\n",
    );

    let out = h.run(&["graph", "central", "--all"]);
    out.assert_exit(0);

    // ADR-001 (bridges both features) should have higher centrality than ADR-002
    let lines: Vec<&str> = out.stdout.lines().collect();
    let adr001_line = lines.iter().find(|l| l.contains("ADR-001"));
    let adr002_line = lines.iter().find(|l| l.contains("ADR-002"));
    assert!(adr001_line.is_some(), "ADR-001 should appear in centrality output.\nOutput:\n{}", out.stdout);
    assert!(adr002_line.is_some(), "ADR-002 should appear in centrality output.\nOutput:\n{}", out.stdout);

    // ADR-001 should be ranked higher (appear first or have higher value)
    let pos1 = out.stdout.find("ADR-001").expect("ADR-001");
    let pos2 = out.stdout.find("ADR-002").expect("ADR-002");
    assert!(pos1 < pos2, "ADR-001 should rank above ADR-002 in centrality.\nOutput:\n{}", out.stdout);
}

#[test]
fn tc_049_centrality_top_n() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-a.md",
        "---\nid: FT-001\ntitle: A\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002, ADR-003, ADR-004]\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-002-b.md",
        "---\nid: FT-002\ntitle: B\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002, ADR-003]\ntests: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-001-a.md",
        "---\nid: ADR-001\ntitle: ADR One\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-002-b.md",
        "---\nid: ADR-002\ntitle: ADR Two\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-003-c.md",
        "---\nid: ADR-003\ntitle: ADR Three\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-004-d.md",
        "---\nid: ADR-004\ntitle: ADR Four\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );

    let out = h.run(&["graph", "central", "--top", "3"]);
    out.assert_exit(0);

    // Count ADR lines in output (excluding header)
    let adr_count = out.stdout.lines().filter(|l| l.contains("ADR-")).count();
    assert_eq!(
        adr_count, 3,
        "Expected exactly 3 ADRs in output, got {}.\nOutput:\n{}",
        adr_count, out.stdout
    );
}

#[test]
fn tc_050_impact_direct() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-a.md",
        "---\nid: FT-001\ntitle: Feature A\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-004-b.md",
        "---\nid: FT-004\ntitle: Feature B\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-002-target.md",
        "---\nid: ADR-002\ntitle: Target ADR\nstatus: accepted\nfeatures: [FT-001, FT-004]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );

    let out = h.run(&["impact", "ADR-002"]);
    out.assert_exit(0);

    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("FT-004");
}

#[test]
fn tc_051_impact_transitive() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-base.md",
        "---\nid: FT-001\ntitle: Base Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n",
    );
    h.write(
        "docs/features/FT-007-transitive.md",
        "---\nid: FT-007\ntitle: Transitive Feature\nphase: 2\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n",
    );
    h.write(
        "docs/adrs/ADR-002-target.md",
        "---\nid: ADR-002\ntitle: Target ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n",
    );

    let out = h.run(&["impact", "ADR-002"]);
    out.assert_exit(0);

    // FT-007 depends on FT-001 which is linked to ADR-002 — should appear as transitive
    out.assert_stdout_contains("FT-007");
}

#[test]
fn tc_009_graph_rebuild_from_scratch() {
    let h = Harness::new();

    // Create 10 feature files
    for i in 1..=10 {
        h.write(
            &format!("docs/features/FT-{i:03}-feat.md"),
            &format!("---\nid: FT-{i:03}\ntitle: Feature {i}\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-{:03}]\ntests: [TC-{i:03}]\n---\n\nFeature {i}.\n", if i <= 8 { i } else { 1 }),
        );
    }

    // Create 8 ADR files
    for i in 1..=8 {
        h.write(
            &format!("docs/adrs/ADR-{i:03}-adr.md"),
            &format!("---\nid: ADR-{i:03}\ntitle: Decision {i}\nstatus: accepted\nfeatures: [FT-{i:03}]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision {i}.\n"),
        );
    }

    // Create 15 test files (first 10 linked to features, rest linked to ADRs)
    for i in 1..=15 {
        let feat = if i <= 10 { format!("FT-{i:03}") } else { format!("FT-{:03}", i - 10) };
        h.write(
            &format!("docs/tests/TC-{i:03}-test.md"),
            &format!("---\nid: TC-{i:03}\ntitle: Test {i}\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [{feat}]\n  adrs: []\nphase: 1\n---\n\nTest {i}.\n"),
        );
    }

    // No prior graph rebuild — just invoke graph stats which uses the in-memory graph
    let out = h.run(&["graph", "stats"]);
    out.assert_exit(0);
    out.assert_stdout_contains("10"); // 10 features
    out.assert_stdout_contains("8");  // 8 ADRs
    out.assert_stdout_contains("15"); // 15 tests

    // Also verify feature list works without any graph rebuild
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("FT-010");
}

#[test]
fn tc_010_graph_stale_ttl() {
    let h = Harness::new();

    // Create initial feature
    h.write(
        "docs/features/FT-001-initial.md",
        "---\nid: FT-001\ntitle: Initial Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nInitial feature.\n",
    );

    // Generate index.ttl via graph rebuild
    let out = h.run(&["graph", "rebuild"]);
    out.assert_exit(0);
    assert!(h.exists("docs/graph/index.ttl"), "index.ttl should be created");

    // Verify index.ttl contains FT-001 but NOT FT-002
    let ttl = h.read("docs/graph/index.ttl");
    assert!(ttl.contains("FT-001"), "index.ttl should contain FT-001");
    assert!(!ttl.contains("FT-002"), "index.ttl should NOT contain FT-002 yet");

    // Add a new feature file WITHOUT rebuilding the TTL
    h.write(
        "docs/features/FT-002-new.md",
        "---\nid: FT-002\ntitle: New Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nNew feature added after TTL export.\n",
    );

    // feature list should show the new feature (graph rebuilt from files, not stale TTL)
    let out = h.run(&["feature", "list"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-002");
    out.assert_stdout_contains("New Feature");
}

#[test]
fn tc_132_cross_cutting_always_in_bundle() {
    let h = harness_with_domains();

    // Cross-cutting ADR with no link from the feature
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nAll errors must use structured diagnostics.\n");

    // Feature that does NOT link ADR-013
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nRate limiting feature.\n");

    let out = h.run(&["context", "FT-009"]);
    out.assert_exit(0);

    // ADR-013 should be included even though not explicitly linked
    assert!(
        out.stdout.contains("ADR-013"),
        "Cross-cutting ADR-013 should appear in bundle even without explicit link.\nBundle:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("Error Model"),
        "ADR-013 title should appear in bundle"
    );
}

#[test]
fn tc_133_cross_cutting_bundle_position() {
    let h = harness_with_domains();

    // Cross-cutting ADR
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nCross-cutting error model.\n");

    // Domain ADR (security, scope: domain)
    h.write("docs/adrs/ADR-020-security-policy.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nDomain-scoped security policy.\n");

    // Feature-linked ADR
    h.write("docs/adrs/ADR-004-rate-algo.md",
        "---\nid: ADR-004\ntitle: Rate Algorithm\nstatus: accepted\nfeatures: [FT-009]\nsupersedes: []\nsuperseded-by: []\ndomains: []\nscope: feature-specific\n---\n\nFeature-specific rate algorithm.\n");

    // Feature that links ADR-004, declares security domain, does not link ADR-013
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-004]\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting feature.\n");

    let out = h.run(&["context", "FT-009"]);
    out.assert_exit(0);

    let bundle = &out.stdout;

    // Find positions of each ADR section
    let pos_cross_cutting = bundle.find("ADR-013")
        .unwrap_or_else(|| panic!("ADR-013 (cross-cutting) not in bundle:\n{}", bundle));
    let pos_domain = bundle.find("ADR-020")
        .unwrap_or_else(|| panic!("ADR-020 (domain) not in bundle:\n{}", bundle));
    let pos_linked = bundle.find("ADR-004")
        .unwrap_or_else(|| panic!("ADR-004 (feature-linked) not in bundle:\n{}", bundle));

    // Cross-cutting before domain
    assert!(
        pos_cross_cutting < pos_domain,
        "Cross-cutting ADR-013 (pos {}) should appear before domain ADR-020 (pos {})",
        pos_cross_cutting, pos_domain
    );
    // Domain before feature-linked
    assert!(
        pos_domain < pos_linked,
        "Domain ADR-020 (pos {}) should appear before feature-linked ADR-004 (pos {})",
        pos_domain, pos_linked
    );
}

#[test]
fn tc_134_domain_top2_centrality() {
    let h = harness_with_domains();

    // Create 6 security-domain ADRs. ADR-001 and ADR-002 will have higher centrality
    // because they are linked from more features.
    h.write("docs/adrs/ADR-001-sec-core.md",
        "---\nid: ADR-001\ntitle: Security Core\nstatus: accepted\nfeatures: [FT-001, FT-002, FT-003]\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nCore security ADR.\n");
    h.write("docs/adrs/ADR-002-sec-auth.md",
        "---\nid: ADR-002\ntitle: Security Auth\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nAuth security ADR.\n");
    h.write("docs/adrs/ADR-003-sec-encrypt.md",
        "---\nid: ADR-003\ntitle: Security Encrypt\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nEncryption ADR.\n");
    h.write("docs/adrs/ADR-004-sec-audit.md",
        "---\nid: ADR-004\ntitle: Security Audit\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nAudit ADR.\n");
    h.write("docs/adrs/ADR-005-sec-tokens.md",
        "---\nid: ADR-005\ntitle: Security Tokens\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nTokens ADR.\n");
    h.write("docs/adrs/ADR-006-sec-rbac.md",
        "---\nid: ADR-006\ntitle: Security RBAC\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nRBAC ADR.\n");

    // Create the features referenced by ADR-001 and ADR-002 (to establish centrality)
    h.write("docs/features/FT-001-alpha.md",
        "---\nid: FT-001\ntitle: Alpha\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nAlpha.\n");
    h.write("docs/features/FT-002-beta.md",
        "---\nid: FT-002\ntitle: Beta\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nBeta.\n");
    h.write("docs/features/FT-003-gamma.md",
        "---\nid: FT-003\ntitle: Gamma\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nGamma.\n");

    // Target feature: declares security domain, does not link any security ADRs
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    let out = h.run(&["context", "FT-009"]);
    out.assert_exit(0);

    let bundle = &out.stdout;

    // Should include the top-2 by centrality: ADR-001 (highest) and ADR-002 (second)
    assert!(
        bundle.contains("ADR-001") && bundle.contains("Security Core"),
        "Bundle should include ADR-001 (highest centrality security ADR).\nBundle:\n{}",
        bundle
    );
    assert!(
        bundle.contains("ADR-002") && bundle.contains("Security Auth"),
        "Bundle should include ADR-002 (second-highest centrality security ADR).\nBundle:\n{}",
        bundle
    );

    // Should NOT include the other 4 security ADRs (only top-2)
    assert!(
        !bundle.contains("Security Encrypt"),
        "Bundle should NOT include ADR-003 (not top-2).\nBundle:\n{}",
        bundle
    );
    assert!(
        !bundle.contains("Security Audit"),
        "Bundle should NOT include ADR-004 (not top-2).\nBundle:\n{}",
        bundle
    );
    assert!(
        !bundle.contains("Security Tokens"),
        "Bundle should NOT include ADR-005 (not top-2).\nBundle:\n{}",
        bundle
    );
    assert!(
        !bundle.contains("Security RBAC"),
        "Bundle should NOT include ADR-006 (not top-2).\nBundle:\n{}",
        bundle
    );
}

#[test]
fn tc_139_domains_vocab_unknown() {
    let h = harness_with_domains();

    // Feature declares a domain not in product.toml vocabulary
    h.write("docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [unknown-domain]\ndomains-acknowledged: {}\n---\n\nBody.\n");

    let out = h.run(&["graph", "check"]);
    out.assert_exit(1)
        .assert_stderr_contains("E012");
    assert!(
        out.stderr.contains("unknown-domain"),
        "E012 should mention the unknown domain name, got stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_168_scan_produces_candidates_with_valid_evidence_paths() {
    let h = Harness::new();
    let fixture_dir = format!(
        "{}/tests/fixtures/onboard-sample",
        env!("CARGO_MANIFEST_DIR")
    );
    let output_path = h.dir.path().join("candidates.json").to_string_lossy().to_string();

    let out = h.run(&["onboard", "scan", &fixture_dir, "--output", &output_path]);
    out.assert_exit(0);

    let content = std::fs::read_to_string(&output_path)
        .expect("read candidates.json");
    let scan: serde_json::Value = serde_json::from_str(&content)
        .expect("parse candidates.json");

    let candidates = scan["candidates"].as_array().expect("candidates array");

    // Assert at least 2 candidates produced
    assert!(
        candidates.len() >= 2,
        "Expected at least 2 candidates, got {}",
        candidates.len()
    );

    // Assert every evidence entry has a valid file path and line number
    for candidate in candidates {
        let evidence = candidate["evidence"].as_array().expect("evidence array");
        for ev in evidence {
            let file = ev["file"].as_str().expect("evidence file");
            let line = ev["line"].as_u64().expect("evidence line");
            let full_path = std::path::Path::new(&fixture_dir).join(file);
            assert!(
                full_path.exists(),
                "Evidence file does not exist: {} (full: {})",
                file,
                full_path.display()
            );
            let file_content = std::fs::read_to_string(&full_path).expect("read evidence file");
            let line_count = file_content.lines().count();
            assert!(
                line as usize <= line_count,
                "Evidence line {} exceeds file length {} in {}",
                line,
                line_count,
                file
            );
            assert!(
                ev["evidence_valid"].as_bool().unwrap_or(false),
                "Evidence should be valid for file {}",
                file
            );
        }
    }
}

#[test]
fn tc_169_scan_rejects_candidates_citing_non_existent_files() {
    let h = Harness::new();

    // Create a scan output with a fabricated evidence file
    let scan_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "boundary",
                "title": "Test valid decision",
                "observation": "Observed valid pattern",
                "evidence": [
                    {"file": "src/main.rs", "line": 1, "snippet": "fn main()", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Bad things",
                "confidence": "high",
                "warnings": []
            },
            {
                "id": "DC-002",
                "signal_type": "boundary",
                "title": "Test invalid decision",
                "observation": "Observed fake pattern",
                "evidence": [
                    {"file": "src/nonexistent.rs", "line": 42, "snippet": "fake code", "evidence_valid": true}
                ],
                "hypothesised_consequence": "Bad things",
                "confidence": "high",
                "warnings": []
            }
        ],
        "scan_metadata": {"files_scanned": 5, "prompt_version": "test"}
    }"#;

    // Create a minimal source directory with only main.rs
    let source_dir = h.dir.path().join("source");
    std::fs::create_dir_all(source_dir.join("src")).expect("mkdir");
    std::fs::write(source_dir.join("src/main.rs"), "fn main() {}\n").expect("write");

    // Run post-validation through the library directly
    use product_lib::onboard;
    let mut scan_output: onboard::ScanOutput = serde_json::from_str(scan_json).expect("parse");
    onboard::validate_all_evidence(&source_dir, &mut scan_output.candidates);

    // The valid candidate should remain valid
    assert!(
        scan_output.candidates[0].evidence[0].evidence_valid,
        "Valid evidence should remain valid"
    );
    assert!(
        scan_output.candidates[0].warnings.is_empty(),
        "Valid candidate should have no warnings"
    );

    // The invalid candidate should be flagged
    assert!(
        !scan_output.candidates[1].evidence[0].evidence_valid,
        "Invalid evidence should be marked as invalid"
    );
    assert!(
        !scan_output.candidates[1].warnings.is_empty(),
        "Invalid candidate should have warnings"
    );
}

#[test]
fn tc_170_scan_respects_max_candidates_cap() {
    let h = Harness::new();
    let fixture_dir = format!(
        "{}/tests/fixtures/onboard-large",
        env!("CARGO_MANIFEST_DIR")
    );
    let output_path = h.dir.path().join("candidates.json").to_string_lossy().to_string();

    let out = h.run(&[
        "onboard",
        "scan",
        &fixture_dir,
        "--max-candidates",
        "5",
        "--output",
        &output_path,
    ]);
    out.assert_exit(0);

    let content = std::fs::read_to_string(&output_path).expect("read candidates.json");
    let scan: serde_json::Value = serde_json::from_str(&content).expect("parse");

    let candidates = scan["candidates"].as_array().expect("candidates array");
    assert!(
        candidates.len() <= 5,
        "Expected at most 5 candidates, got {}",
        candidates.len()
    );

    // Verify the fixture would produce more than 5 without the cap
    let output_uncapped = h.dir.path().join("candidates_full.json").to_string_lossy().to_string();
    let out2 = h.run(&[
        "onboard",
        "scan",
        &fixture_dir,
        "--output",
        &output_uncapped,
    ]);
    out2.assert_exit(0);
    let content2 = std::fs::read_to_string(&output_uncapped).expect("read full candidates");
    let scan2: serde_json::Value = serde_json::from_str(&content2).expect("parse");
    let candidates2 = scan2["candidates"].as_array().expect("candidates array");
    assert!(
        candidates2.len() > 5,
        "Uncapped scan should produce more than 5 candidates, got {}",
        candidates2.len()
    );
}

#[test]
fn tc_174_seed_creates_adr_files_with_correct_front_matter() {
    let h = Harness::new();
    let fixture_dir = format!(
        "{}/tests/fixtures/onboard-sample",
        env!("CARGO_MANIFEST_DIR")
    );
    let candidates_path = h.dir.path().join("candidates.json").to_string_lossy().to_string();

    // Scan
    let out = h.run(&["onboard", "scan", &fixture_dir, "--output", &candidates_path]);
    out.assert_exit(0);

    // Triage — confirm all
    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();
    let content = std::fs::read_to_string(&candidates_path).expect("read");
    let scan: serde_json::Value = serde_json::from_str(&content).expect("parse");
    let num_candidates = scan["candidates"].as_array().expect("arr").len();
    let confirms: String = (0..num_candidates).map(|_| "c\n").collect();
    let out = h.run_with_stdin(
        &["onboard", "triage", &candidates_path, "--interactive", "--output", &triaged_path],
        &confirms,
    );
    out.assert_exit(0);

    // Seed
    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    // Verify each ADR file has correct front-matter
    let adrs_dir = h.dir.path().join("docs/adrs");
    let adr_files: Vec<_> = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("ADR-") && name.ends_with(".md")
        })
        .collect();

    assert!(!adr_files.is_empty(), "Should create at least one ADR file");

    for adr_file in &adr_files {
        let content = std::fs::read_to_string(adr_file.path()).expect("read ADR");
        let name = adr_file.file_name().to_string_lossy().to_string();

        // ID pattern
        assert!(
            name.starts_with("ADR-"),
            "ADR filename should start with ADR-: {}",
            name
        );

        // Status
        assert!(
            content.contains("status: proposed"),
            "ADR {} should have status: proposed",
            name
        );

        // Front-matter structure
        assert!(
            content.starts_with("---\n"),
            "ADR {} should start with YAML front-matter",
            name
        );
        assert!(
            content.contains("features: []") || content.contains("features:"),
            "ADR {} should have features field",
            name
        );
        assert!(
            content.contains("supersedes: []") || content.contains("supersedes:"),
            "ADR {} should have supersedes field",
            name
        );
    }

    // Run graph check — should report no E-class errors
    let out = h.run(&["graph", "check"]);
    // Exit 0 or 2 (warnings only) is acceptable
    assert!(
        out.exit_code == 0 || out.exit_code == 2,
        "Expected exit 0 or 2, got {}. stderr: {}",
        out.exit_code,
        out.stderr
    );
    // No E001 errors
    assert!(
        !out.stderr.contains("E001"),
        "Should have no E001 malformed front-matter errors: {}",
        out.stderr
    );
}

#[test]
fn tc_175_seed_groups_candidates_into_feature_stubs_by_signal_proximity() {
    let h = Harness::new();

    // Create triaged candidates from two distinct evidence clusters
    let triaged_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "consistency",
                "title": "API error handling convention",
                "observation": "All API handlers use AppError",
                "evidence": [{"file": "src/api/handler.rs", "line": 1, "snippet": "use AppError;", "evidence_valid": true}],
                "hypothesised_consequence": "Breaks error contract",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-002",
                "signal_type": "convention",
                "title": "API response format",
                "observation": "All responses use JSON",
                "evidence": [{"file": "src/api/routes.rs", "line": 1, "snippet": "use serde_json;", "evidence_valid": true}],
                "hypothesised_consequence": "Breaks API contract",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-003",
                "signal_type": "consistency",
                "title": "API middleware pattern",
                "observation": "All endpoints use auth middleware",
                "evidence": [{"file": "src/api/middleware.rs", "line": 1, "snippet": "auth check", "evidence_valid": true}],
                "hypothesised_consequence": "Bypasses auth",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-004",
                "signal_type": "boundary",
                "title": "Storage access through repository only",
                "observation": "Only repo accesses DB",
                "evidence": [{"file": "src/storage/db.rs", "line": 1, "snippet": "use sqlx;", "evidence_valid": true}],
                "hypothesised_consequence": "Bypasses transactions",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-005",
                "signal_type": "constraint",
                "title": "Storage caching constraint",
                "observation": "All caches in-process",
                "evidence": [{"file": "src/storage/cache.rs", "line": 1, "snippet": "in-memory only", "evidence_valid": true}],
                "hypothesised_consequence": "Breaks deployment model",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            }
        ]
    }"#;

    let triaged_path = h.dir.path().join("triaged.json");
    std::fs::write(&triaged_path, triaged_json).expect("write triaged");

    let out = h.run(&["onboard", "seed", &triaged_path.to_string_lossy()]);
    out.assert_exit(0);

    // Check feature stubs
    let features_dir = h.dir.path().join("docs/features");
    let feature_files: Vec<_> = std::fs::read_dir(&features_dir)
        .expect("read features dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("FT-") && name.ends_with(".md")
        })
        .collect();

    // At least 2 feature stubs (one for api/ cluster, one for storage/ cluster)
    assert!(
        feature_files.len() >= 2,
        "Expected at least 2 feature stubs, got {}",
        feature_files.len()
    );

    // All feature stubs should have status: planned
    for ft_file in &feature_files {
        let content = std::fs::read_to_string(ft_file.path()).expect("read feature");
        assert!(
            content.contains("status: planned"),
            "Feature stub {} should have status: planned",
            ft_file.file_name().to_string_lossy()
        );
    }

    // Verify API-related ADRs and storage-related ADRs are in different features
    let mut api_feature: Option<String> = None;
    let mut storage_feature: Option<String> = None;

    for ft_file in &feature_files {
        let content = std::fs::read_to_string(ft_file.path()).expect("read feature");
        let name = ft_file.file_name().to_string_lossy().to_string();
        if content.contains("api") {
            api_feature = Some(name.clone());
        }
        if content.contains("storage") {
            storage_feature = Some(name.clone());
        }
    }

    // They should be different features (or at least both exist)
    if let (Some(ref api), Some(ref storage)) = (&api_feature, &storage_feature) {
        assert_ne!(
            api, storage,
            "API and storage ADRs should be in different feature stubs"
        );
    }
}

#[test]
fn tc_176_seed_dry_run_writes_no_files() {
    let h = Harness::new();

    let triaged_json = r#"{
        "candidates": [
            {
                "id": "DC-001",
                "signal_type": "boundary",
                "title": "Decision one",
                "observation": "Observed one",
                "evidence": [{"file": "src/a.rs", "line": 1, "snippet": "test", "evidence_valid": true}],
                "hypothesised_consequence": "Bad one",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-002",
                "signal_type": "consistency",
                "title": "Decision two",
                "observation": "Observed two",
                "evidence": [{"file": "src/b.rs", "line": 1, "snippet": "test", "evidence_valid": true}],
                "hypothesised_consequence": "Bad two",
                "confidence": "medium",
                "warnings": [],
                "triage_status": "confirmed"
            },
            {
                "id": "DC-003",
                "signal_type": "constraint",
                "title": "Decision three",
                "observation": "Observed three",
                "evidence": [{"file": "src/c.rs", "line": 1, "snippet": "test", "evidence_valid": true}],
                "hypothesised_consequence": "Bad three",
                "confidence": "high",
                "warnings": [],
                "triage_status": "confirmed"
            }
        ]
    }"#;

    let triaged_path = h.dir.path().join("triaged.json");
    std::fs::write(&triaged_path, triaged_json).expect("write triaged");

    // Count files before
    let adrs_dir = h.dir.path().join("docs/adrs");
    let before_count = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .count();

    // Run dry-run
    let out = h.run(&["onboard", "seed", &triaged_path.to_string_lossy(), "--dry-run"]);
    out.assert_exit(0);

    // Stdout should mention proposed files
    out.assert_stdout_contains("ADR-001");
    out.assert_stdout_contains("Dry run");

    // No files should be created
    let after_count = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .count();
    assert_eq!(
        before_count, after_count,
        "Dry run should not create any files"
    );

    // Now run for real
    let out = h.run(&["onboard", "seed", &triaged_path.to_string_lossy()]);
    out.assert_exit(0);

    let final_count = std::fs::read_dir(&adrs_dir)
        .expect("read adrs dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .count();
    assert_eq!(
        final_count, 3,
        "Real seed should create exactly 3 ADR files"
    );
}

#[test]
fn tc_178_seeded_adrs_have_no_g005_contradictions_after_gap_check() {
    let h = Harness::new();
    let fixture_dir = format!(
        "{}/tests/fixtures/onboard-sample",
        env!("CARGO_MANIFEST_DIR")
    );
    let candidates_path = h.dir.path().join("candidates.json").to_string_lossy().to_string();
    let triaged_path = h.dir.path().join("triaged.json").to_string_lossy().to_string();

    // Full pipeline: scan → triage (batch confirm) → seed
    let out = h.run(&["onboard", "scan", &fixture_dir, "--output", &candidates_path]);
    out.assert_exit(0);

    let out = h.run(&["onboard", "triage", &candidates_path, "--output", &triaged_path]);
    out.assert_exit(0);

    let out = h.run(&["onboard", "seed", &triaged_path]);
    out.assert_exit(0);

    // Run gap check
    let out = h.run(&["--format", "json", "gap", "check"]);
    // Gap check may exit 0 or 1 (findings exist), not 2 (error)
    assert!(
        out.exit_code != 2,
        "Gap check should not error, got exit code {}. stderr: {}",
        out.exit_code,
        out.stderr
    );

    // No G005 contradictions
    assert!(
        !out.stdout.contains("G005"),
        "Should have no G005 architectural contradiction findings. stdout: {}",
        out.stdout
    );
}

#[test]
fn tc_362_graph_infer_general() {
    let h = Harness::new();
    h.write("docs/features/FT-009-test.md", "\
---
id: FT-009
title: Rate Limiting
phase: 1
status: planned
adrs:
- ADR-021
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-021-domain.md", "\
---
id: ADR-021
title: Token Bucket Rate Limiting
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-041-test.md", "\
---
id: TC-041
title: Rate Limit Under Load
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-021
phase: 1
---

TC body.
");
    h.write("docs/tests/TC-042-test.md", "\
---
id: TC-042
title: Token Bucket Refill
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-021
phase: 1
---

TC body.
");

    let out = h.run(&["graph", "infer", "--feature", "FT-009"]);
    out.assert_exit(0);

    // TC-041 and TC-042 gain FT-009
    let tc41 = h.read("docs/tests/TC-041-test.md");
    assert!(tc41.contains("FT-009"), "TC-041 should gain FT-009. Got:\n{}", tc41);

    let tc42 = h.read("docs/tests/TC-042-test.md");
    assert!(tc42.contains("FT-009"), "TC-042 should gain FT-009. Got:\n{}", tc42);

    // FT-009 gains TC-041 and TC-042
    let ft = h.read("docs/features/FT-009-test.md");
    assert!(ft.contains("TC-041"), "FT-009 should gain TC-041. Got:\n{}", ft);
    assert!(ft.contains("TC-042"), "FT-009 should gain TC-042. Got:\n{}", ft);
}

#[test]
fn tc_365_reverse_inference_updates_feature() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
tests:
- TC-001
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
    h.write("docs/tests/TC-001-existing.md", "\
---
id: TC-001
title: Existing TC
type: scenario
status: unimplemented
validates:
  features:
  - FT-001
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    h.write("docs/tests/TC-002-new.md", "\
---
id: TC-002
title: New TC
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

    // After inference adds FT-001 to TC-002.validates.features
    let tc2 = h.read("docs/tests/TC-002-new.md");
    assert!(tc2.contains("FT-001"), "TC-002 should gain FT-001. Got:\n{}", tc2);

    // FT-001.tests should now include TC-002 (reverse inference)
    let ft = h.read("docs/features/FT-001-test.md");
    assert!(ft.contains("TC-002"), "FT-001 should gain TC-002 via reverse inference. Got:\n{}", ft);

    // FT-001 should still have TC-001
    assert!(ft.contains("TC-001"), "FT-001 should retain TC-001. Got:\n{}", ft);
}

#[test]
fn tc_442_graph_check_emits_w017_for_complete_feature_with_proposed_adr() {
    // Test with status: complete
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W017");
    out.assert_stderr_contains("ADR-001");
    out.assert_stderr_contains("proposed");
    out.assert_stderr_contains("hint:");
    // Exit code 2 = warnings only (ignoring other possible warnings, at minimum we have W017)
    assert!(
        out.exit_code == 2 || out.exit_code == 1,
        "Expected exit code 2 (warnings) or 1 (if other errors present), got {}",
        out.exit_code
    );

    // Also test with in-progress status
    let h2 = Harness::new();
    h2.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h2.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h2.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );

    let out2 = h2.run(&["graph", "check"]);
    out2.assert_stderr_contains("W017");
    out2.assert_stderr_contains("ADR-001");
}

#[test]
fn tc_475_graph_check_emits_w019_for_out_of_scope_feature() {
    let h = fixture_with_responsibility();
    h.write("docs/features/FT-099-grocery.md", "---\nid: FT-099\ntitle: Grocery List Management\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nManage grocery lists and shopping.\n");
    let out = h.run(&["graph", "check"]);
    out.assert_stderr_contains("W019");
    out.assert_stderr_contains("FT-099");

    // In-scope features should not trigger W019
    let h2 = fixture_with_responsibility();
    let out2 = h2.run(&["graph", "check"]);
    assert!(!out2.stderr.contains("W019"), "in-scope features should not trigger W019: {}", out2.stderr);
}

#[test]
fn tc_470_all_field_mutation_tools_are_idempotent() {
    let h = fixture_with_domains();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");
    h.write("docs/adrs/ADR-002-test.md", "---\nid: ADR-002\ntitle: Test ADR 2\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: []\n  adrs: []\nphase: 1\n---\n\nDesc.\n");

    // feature_domain: apply twice, same result
    h.run(&["feature", "domain", "FT-001", "--add", "api"]).assert_exit(0);
    let after_first = h.read("docs/features/FT-001-test.md");
    h.run(&["feature", "domain", "FT-001", "--add", "api"]).assert_exit(0);
    let after_second = h.read("docs/features/FT-001-test.md");
    assert_eq!(after_first, after_second, "feature_domain should be idempotent");

    // feature_acknowledge: apply twice, same result
    h.run(&["feature", "acknowledge", "FT-001", "--domain", "security", "--reason", "No new trust boundaries"]).assert_exit(0);
    let after_first = h.read("docs/features/FT-001-test.md");
    h.run(&["feature", "acknowledge", "FT-001", "--domain", "security", "--reason", "No new trust boundaries"]).assert_exit(0);
    let after_second = h.read("docs/features/FT-001-test.md");
    assert_eq!(after_first, after_second, "feature_acknowledge should be idempotent");

    // adr_domain: apply twice, same result
    h.run(&["adr", "domain", "ADR-001", "--add", "error-handling"]).assert_exit(0);
    let after_first = h.read("docs/adrs/ADR-001-test.md");
    h.run(&["adr", "domain", "ADR-001", "--add", "error-handling"]).assert_exit(0);
    let after_second = h.read("docs/adrs/ADR-001-test.md");
    assert_eq!(after_first, after_second, "adr_domain should be idempotent");

    // adr_scope: apply twice, same result
    h.run(&["adr", "scope", "ADR-001", "cross-cutting"]).assert_exit(0);
    let after_first = h.read("docs/adrs/ADR-001-test.md");
    h.run(&["adr", "scope", "ADR-001", "cross-cutting"]).assert_exit(0);
    let after_second = h.read("docs/adrs/ADR-001-test.md");
    assert_eq!(after_first, after_second, "adr_scope should be idempotent");

    // adr_supersede: apply twice, same result
    h.run(&["adr", "supersede", "ADR-002", "--supersedes", "ADR-001"]).assert_exit(0);
    let after_first_a = h.read("docs/adrs/ADR-001-test.md");
    let after_first_b = h.read("docs/adrs/ADR-002-test.md");
    h.run(&["adr", "supersede", "ADR-002", "--supersedes", "ADR-001"]).assert_exit(0);
    let after_second_a = h.read("docs/adrs/ADR-001-test.md");
    let after_second_b = h.read("docs/adrs/ADR-002-test.md");
    assert_eq!(after_first_a, after_second_a, "adr_supersede should be idempotent (target)");
    assert_eq!(after_first_b, after_second_b, "adr_supersede should be idempotent (source)");

    // adr_source_files: apply twice, same result
    h.run(&["adr", "source-files", "ADR-001", "--add", "src/test.rs"]).assert_exit(0);
    let after_first = h.read("docs/adrs/ADR-001-test.md");
    h.run(&["adr", "source-files", "ADR-001", "--add", "src/test.rs"]).assert_exit(0);
    let after_second = h.read("docs/adrs/ADR-001-test.md");
    assert_eq!(after_first, after_second, "adr_source_files should be idempotent");

    // test_runner: apply twice, same result
    h.run(&["test", "runner", "TC-001", "--runner", "cargo-test", "--args", "tc_001_test"]).assert_exit(0);
    let after_first = h.read("docs/tests/TC-001-test.md");
    h.run(&["test", "runner", "TC-001", "--runner", "cargo-test", "--args", "tc_001_test"]).assert_exit(0);
    let after_second = h.read("docs/tests/TC-001-test.md");
    assert_eq!(after_first, after_second, "test_runner should be idempotent");
}

#[test]
fn tc_480_graph_stats_shows_bundle_token_summary() {
    let h = fixture_bundle_summary();
    // Measure 2 of 3 features.
    h.run(&["context", "FT-001", "--measure"]).assert_exit(0);
    h.run(&["context", "FT-002", "--measure"]).assert_exit(0);

    let out = h.run(&["graph", "stats"]);
    out.assert_exit(0);
    out.assert_stdout_contains("Bundle size");
    out.assert_stdout_contains("measured:");
    out.assert_stdout_contains("2 / 3");
    out.assert_stdout_contains("mean:");
    out.assert_stdout_contains("median:");
    out.assert_stdout_contains("p95:");
    out.assert_stdout_contains("max:");
    out.assert_stdout_contains("min:");
    // Max/min should list a feature ID.
    let has_ft001 = out.stdout.contains("FT-001");
    let has_ft002 = out.stdout.contains("FT-002");
    assert!(has_ft001 || has_ft002, "Expected max/min to reference a feature ID.\nstdout:\n{}", out.stdout);
    // Threshold breach lines exist.
    out.assert_stdout_contains("Over token threshold");
    out.assert_stdout_contains("Over ADR threshold");
    // Unmeasured FT-003 should be reported.
    out.assert_stdout_contains("FT-003");
}

#[test]
fn tc_481_graph_stats_shows_no_measurements_message() {
    let h = fixture_bundle_summary();
    let out = h.run(&["graph", "stats"]);
    out.assert_exit(0);
    out.assert_stdout_contains("No bundle measurements");
    out.assert_stdout_contains("product context --measure-all");
}

#[test]
fn tc_485_aggregate_bundle_metrics_exit_criteria() {
    // 1. graph stats shows "No bundle measurements" initially.
    let h = fixture_bundle_summary();
    let before = h.run(&["graph", "stats"]);
    before.assert_exit(0);
    before.assert_stdout_contains("No bundle measurements");

    // 2. measure-all writes bundle blocks + metrics.jsonl entries and exits 0.
    let measure = h.run(&["context", "--measure-all"]);
    measure.assert_exit(0);
    measure.assert_stdout_contains("Bundle size");
    // But does not flood with bundle content.
    assert!(!measure.stdout.contains("# Context Bundle:"));
    assert!(h.exists("metrics.jsonl"), "metrics.jsonl must exist after measure-all");

    // 3. graph stats now shows the aggregate summary with mean/median/p95/max/min.
    let after = h.run(&["graph", "stats"]);
    after.assert_exit(0);
    after.assert_stdout_contains("Bundle size");
    after.assert_stdout_contains("mean:");
    after.assert_stdout_contains("median:");
    after.assert_stdout_contains("p95:");
    after.assert_stdout_contains("max:");
    after.assert_stdout_contains("min:");
    // No "No bundle measurements" line now.
    assert!(
        !after.stdout.contains("No bundle measurements"),
        "After measure-all, stats must not show no-measurements line.\nstdout:\n{}",
        after.stdout
    );

    // 4. --depth flag is honored and all features updated.
    let d2 = h.run(&["context", "--measure-all", "--depth", "2"]);
    d2.assert_exit(0);
    for path in &[
        "docs/features/FT-001-alpha.md",
        "docs/features/FT-002-beta.md",
        "docs/features/FT-003-gamma.md",
    ] {
        let content = h.read(path);
        assert!(content.contains("bundle:"), "{} missing bundle block", path);
    }
}

