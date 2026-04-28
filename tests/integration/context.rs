//! Integration tests — context.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn it_005_context_bundle_header() {
    let h = fixture_minimal();
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0)
        .assert_stdout_contains("Bundle");
    // No YAML front-matter delimiters in output (stripped)
    assert!(!out.stdout.starts_with("---\n"));
}

#[test]
fn tc_040_context_bundle_formal_blocks_preserved() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: invariant\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nSome text.\n\n⟦Γ:Invariants⟧{\n  ∀x:Node: connected(x) = true\n}\n");
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Formal blocks must be in the output, not stripped
    assert!(
        out.stdout.contains("⟦Γ:Invariants⟧"),
        "Formal blocks should be preserved in context bundle, got: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains("∀x:Node"),
        "Invariant content should be preserved"
    );
}

#[test]
fn tc_017_context_bundle_no_frontmatter() {
    let h = fixture_minimal();
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // The YAML front-matter delimiter "---" at the start of a section should be stripped.
    // The bundle should not contain any "---\nid:" patterns (front-matter blocks).
    let lines: Vec<&str> = out.stdout.lines().collect();
    let mut in_frontmatter = false;
    for (i, line) in lines.iter().enumerate() {
        // Front-matter starts with "---" and contains "id:" on the next line(s)
        if *line == "---" && i + 1 < lines.len() {
            // Check if next lines look like YAML front-matter (key: value)
            if let Some(next) = lines.get(i + 1) {
                if next.starts_with("id:") || next.starts_with("title:") || next.starts_with("status:") {
                    in_frontmatter = true;
                    panic!(
                        "Context bundle contains YAML front-matter at line {}: {}",
                        i + 1,
                        line
                    );
                }
            }
        }
    }
    assert!(!in_frontmatter, "Context bundle should not contain any YAML front-matter blocks");
    // Also verify the output doesn't start with front-matter
    assert!(!out.stdout.starts_with("---\n"), "Bundle should not start with front-matter delimiter");
}

#[test]
fn tc_019_context_bundle_superseded_adr() {
    let h = Harness::new();
    // Create a feature linked to both a superseded ADR and its successor
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: []\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-old.md",
        "---\nid: ADR-001\ntitle: Old Decision\nstatus: superseded\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: [ADR-002]\n---\n\nOld decision body.\n",
    );
    h.write(
        "docs/adrs/ADR-002-new.md",
        "---\nid: ADR-002\ntitle: New Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: [ADR-001]\nsuperseded-by: []\n---\n\nNew decision body.\n",
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // The superseded ADR should appear in the bundle with annotation
    assert!(
        out.stdout.contains("[SUPERSEDED by ADR-002]"),
        "Superseded ADR should have [SUPERSEDED by ADR-XXX] annotation.\nOutput:\n{}",
        out.stdout
    );
    // Both ADRs should be present
    assert!(
        out.stdout.contains("ADR-001"),
        "Superseded ADR-001 should appear in bundle"
    );
    assert!(
        out.stdout.contains("ADR-002"),
        "Successor ADR-002 should appear in bundle"
    );
}

#[test]
fn tc_047_context_bundle_adr_order_centrality() {
    let h = Harness::new();
    // ADR-001 is linked to many features (high centrality)
    // ADR-007 is linked to only one feature (low centrality)
    h.write(
        "docs/features/FT-001-main.md",
        "---\nid: FT-001\ntitle: Main Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-007]\ntests: []\n---\n\nMain feature.\n",
    );
    h.write(
        "docs/features/FT-002-extra.md",
        "---\nid: FT-002\ntitle: Extra Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nExtra.\n",
    );
    h.write(
        "docs/features/FT-003-extra2.md",
        "---\nid: FT-003\ntitle: Extra Feature 2\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nExtra 2.\n",
    );
    h.write(
        "docs/adrs/ADR-001-foundational.md",
        "---\nid: ADR-001\ntitle: Foundational ADR\nstatus: accepted\nfeatures: [FT-001, FT-002, FT-003]\nsupersedes: []\nsuperseded-by: []\n---\n\nFoundational decision.\n",
    );
    h.write(
        "docs/adrs/ADR-007-peripheral.md",
        "---\nid: ADR-007\ntitle: Peripheral ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nPeripheral decision.\n",
    );

    // Default bundle output orders ADRs by centrality (high first)
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    let adr001_pos = out.stdout.find("ADR-001").expect("ADR-001 should appear in bundle");
    let adr007_pos = out.stdout.find("ADR-007").expect("ADR-007 should appear in bundle");
    assert!(
        adr001_pos < adr007_pos,
        "ADR-001 (high centrality) should appear before ADR-007 (low centrality).\nBundle:\n{}",
        out.stdout
    );
}

#[test]
fn tc_016_context_bundle_feature() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\n---\n\nFeature content here.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nFirst ADR content.\n",
    );
    h.write(
        "docs/adrs/ADR-002-second.md",
        "---\nid: ADR-002\ntitle: Second Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nSecond ADR content.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test Criterion\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest criterion content.\n",
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // All content present
    out.assert_stdout_contains("Feature content here.");
    out.assert_stdout_contains("First ADR content.");
    out.assert_stdout_contains("Second ADR content.");
    out.assert_stdout_contains("Test criterion content.");

    // Correct order: feature → ADRs → tests
    let ft_pos = out.stdout.find("Feature content here.").expect("feature body");
    let adr1_pos = out.stdout.find("First ADR content.").expect("ADR-001 body");
    let adr2_pos = out.stdout.find("Second ADR content.").expect("ADR-002 body");
    let tc_pos = out.stdout.find("Test criterion content.").expect("TC body");
    assert!(ft_pos < adr1_pos, "Feature should appear before ADR-001");
    assert!(ft_pos < adr2_pos, "Feature should appear before ADR-002");
    assert!(adr1_pos < tc_pos, "ADR-001 should appear before TC");
    assert!(adr2_pos < tc_pos, "ADR-002 should appear before TC");
}

#[test]
fn tc_018_context_bundle_header() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Header Test\nphase: 2\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nHeader test feature.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 2\n---\n\nTC body.\n",
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);

    // Header should contain correct metadata
    out.assert_stdout_contains("feature≜FT-001:Feature");
    out.assert_stdout_contains("phase≜2:Phase");
    out.assert_stdout_contains("InProgress:FeatureStatus");
    out.assert_stdout_contains("implementedBy≜⟨ADR-001⟩:Decision+");
    out.assert_stdout_contains("validatedBy≜⟨TC-001⟩:TestCriterion+");
}

#[test]
fn tc_045_context_depth_2() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-seed.md",
        "---\nid: FT-001\ntitle: Seed Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nSeed feature.\n",
    );
    h.write(
        "docs/features/FT-004-transitive.md",
        "---\nid: FT-004\ntitle: Transitive Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: [TC-009]\n---\n\nTransitive feature.\n",
    );
    h.write(
        "docs/adrs/ADR-002-shared.md",
        "---\nid: ADR-002\ntitle: Shared ADR\nstatus: accepted\nfeatures: [FT-001, FT-004]\nsupersedes: []\nsuperseded-by: []\n---\n\nShared decision.\n",
    );
    h.write(
        "docs/tests/TC-009-transitive.md",
        "---\nid: TC-009\ntitle: Transitive Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-004]\n  adrs: [ADR-002]\nphase: 1\n---\n\nTransitive test.\n",
    );

    // Depth 1 should NOT include TC-009 (it validates FT-004, not FT-001)
    let out1 = h.run(&["context", "FT-001", "--depth", "1"]);
    out1.assert_exit(0);
    assert!(
        !out1.stdout.contains("TC-009") && !out1.stdout.contains("Transitive test."),
        "Depth 1 should not include TC-009.\nOutput:\n{}",
        out1.stdout
    );

    // Depth 2 should include TC-009 (via ADR-002 → FT-004 → TC-009)
    let out2 = h.run(&["context", "FT-001", "--depth", "2"]);
    out2.assert_exit(0);
    assert!(
        out2.stdout.contains("TC-009") || out2.stdout.contains("Transitive test."),
        "Depth 2 should include TC-009 (transitive via ADR-002 → FT-004).\nOutput:\n{}",
        out2.stdout
    );
}

#[test]
fn tc_046_context_depth_dedup() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-main.md",
        "---\nid: FT-001\ntitle: Main\nphase: 1\nstatus: planned\ndepends-on: [FT-002]\nadrs: [ADR-002]\ntests: []\n---\n\nMain feature.\n",
    );
    h.write(
        "docs/features/FT-002-dep.md",
        "---\nid: FT-002\ntitle: Dep\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nDep feature.\n",
    );
    h.write(
        "docs/adrs/ADR-002-shared.md",
        "---\nid: ADR-002\ntitle: Shared Decision\nstatus: accepted\nfeatures: [FT-001, FT-002]\nsupersedes: []\nsuperseded-by: []\n---\n\nShared ADR body unique marker.\n",
    );

    let out = h.run(&["context", "FT-001", "--depth", "2"]);
    out.assert_exit(0);

    // Count occurrences of the ADR body — should appear exactly once
    let count = out.stdout.matches("Shared ADR body unique marker.").count();
    assert_eq!(
        count, 1,
        "ADR-002 should appear exactly once in the bundle, found {} times.\nOutput:\n{}",
        count, out.stdout
    );
}

#[test]
fn tc_148_coverage_matrix_domain_filter() {
    let h = harness_with_domains();

    // Domain-scoped ADRs
    h.write("docs/adrs/ADR-020-security-policy.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");
    h.write("docs/adrs/ADR-030-networking.md",
        "---\nid: ADR-030\ntitle: Networking Core\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [networking]\nscope: domain\n---\n\nNetworking.\n");

    // Feature
    h.write("docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-020]\ntests: []\ndomains: [security, networking]\ndomains-acknowledged: {}\n---\n\nTest.\n");

    // Unfiltered should show both columns
    let out_all = h.run(&["graph", "coverage"]);
    out_all.assert_exit(0);
    assert!(
        out_all.stdout.contains("secur") && out_all.stdout.contains("netwo"),
        "Unfiltered coverage should show both domains, got:\n{}",
        out_all.stdout
    );

    // Filtered to security only
    let out_sec = h.run(&["graph", "coverage", "--domain", "security"]);
    out_sec.assert_exit(0);
    assert!(
        out_sec.stdout.contains("secur"),
        "Filtered coverage should show security column, got:\n{}",
        out_sec.stdout
    );
    assert!(
        !out_sec.stdout.contains("netwo"),
        "Filtered coverage should NOT show networking column, got:\n{}",
        out_sec.stdout
    );
}

#[test]
fn tc_146_coverage_matrix_renders() {
    let h = harness_with_domains();

    // Domain ADRs
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");
    h.write("docs/adrs/ADR-030-networking.md",
        "---\nid: ADR-030\ntitle: Networking Core\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [networking]\nscope: domain\n---\n\nNetworking.\n");

    // FT-001: links ADR-020 (security ✓), declares networking (gap ✗)
    h.write("docs/features/FT-001-cluster.md",
        "---\nid: FT-001\ntitle: Cluster\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-020]\ntests: []\ndomains: [security, networking]\ndomains-acknowledged: {}\n---\n\nCluster.\n");

    // FT-002: acknowledges security (~), does not declare networking (·)
    h.write("docs/features/FT-002-products.md",
        "---\nid: FT-002\ntitle: Products\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: [security]\ndomains-acknowledged:\n  security: \"no trust boundaries\"\n---\n\nProducts.\n");

    let out = h.run(&["graph", "coverage"]);
    out.assert_exit(0);

    // All features present
    assert!(out.stdout.contains("FT-001"), "Should contain FT-001");
    assert!(out.stdout.contains("FT-002"), "Should contain FT-002");

    // Domain columns present
    assert!(out.stdout.contains("secur"), "Should show security domain");
    assert!(out.stdout.contains("netwo"), "Should show networking domain");

    // Coverage symbols: expect ✓ (linked), ~ (acknowledged), ✗ (gap), · (not applicable)
    assert!(out.stdout.contains('✓'), "Should contain ✓ for linked coverage");
    assert!(out.stdout.contains('~'), "Should contain ~ for acknowledged");
    assert!(out.stdout.contains('✗') || out.stdout.contains('·'),
        "Should contain ✗ or · for gap/not-applicable, got:\n{}", out.stdout);

    // Legend
    assert!(out.stdout.contains("Legend"), "Should contain legend");
}

#[test]
fn tc_147_coverage_matrix_json() {
    let h = harness_with_domains();

    // Domain ADR
    h.write("docs/adrs/ADR-020-security.md",
        "---\nid: ADR-020\ntitle: Security Policy\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [security]\nscope: domain\n---\n\nSecurity.\n");

    // Feature
    h.write("docs/features/FT-001-cluster.md",
        "---\nid: FT-001\ntitle: Cluster\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-020]\ntests: []\ndomains: [security]\ndomains-acknowledged: {}\n---\n\nCluster.\n");

    let out = h.run(&["graph", "coverage", "--format", "json"]);
    out.assert_exit(0);

    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .expect("Should produce valid JSON");

    // Must have features array
    assert!(json["features"].is_array(), "JSON should have 'features' array");
    let features = json["features"].as_array().expect("features is array");
    assert!(!features.is_empty(), "features should not be empty");

    // Each feature should have a domains map with coverage status
    for feat in features {
        assert!(feat["id"].is_string(), "Feature should have 'id' string field");
        assert!(feat["domains"].is_object(), "Feature should have 'domains' map");
        let domains = feat["domains"].as_object().expect("domains is object");
        for (_domain_name, status) in domains {
            assert!(status.is_string(), "Domain status should be a string");
        }
    }
}

#[test]
fn tc_201_context_measure_updates_frontmatter() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001, ADR-002]\ntests: [TC-001]\ndomains: [storage, network]\n---\n\nTest feature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First Decision\nstatus: accepted\nfeatures: [FT-001]\n---\n\nFirst ADR body.\n",
    );
    h.write(
        "docs/adrs/ADR-002-second.md",
        "---\nid: ADR-002\ntitle: Second Decision\nstatus: accepted\nfeatures: [FT-001]\n---\n\nSecond ADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest one body.\n",
    );

    let out = h.run(&["context", "FT-001", "--measure"]);
    out.assert_exit(0);

    // Read the updated feature file
    let content = h.read("docs/features/FT-001-test.md");
    assert!(
        content.contains("depth-1-adrs:"),
        "Feature file should contain depth-1-adrs field.\nContent:\n{}",
        content
    );
    assert!(
        content.contains("tcs:"),
        "Feature file should contain tcs field.\nContent:\n{}",
        content
    );
    assert!(
        content.contains("tokens-approx:"),
        "Feature file should contain tokens-approx field.\nContent:\n{}",
        content
    );
    assert!(
        content.contains("measured-at:"),
        "Feature file should contain measured-at field.\nContent:\n{}",
        content
    );
    // Check specific values
    assert!(
        content.contains("depth-1-adrs: 2"),
        "Should have 2 depth-1 ADRs.\nContent:\n{}",
        content
    );
    assert!(
        content.contains("tcs: 1"),
        "Should have 1 TC.\nContent:\n{}",
        content
    );
}

#[test]
fn tc_202_context_measure_appends_metrics() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First\nstatus: accepted\nfeatures: [FT-001]\n---\n\nADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest body.\n",
    );

    let out = h.run(&["context", "FT-001", "--measure"]);
    out.assert_exit(0);

    // Check metrics.jsonl exists and has correct content
    let metrics = h.read("metrics.jsonl");
    assert!(
        !metrics.is_empty(),
        "metrics.jsonl should exist and not be empty"
    );
    assert!(
        metrics.contains("FT-001"),
        "metrics.jsonl should contain feature ID.\nContent:\n{}",
        metrics
    );
    assert!(
        metrics.contains("depth-1-adrs"),
        "metrics.jsonl should contain depth-1-adrs field.\nContent:\n{}",
        metrics
    );
    assert!(
        metrics.contains("tokens-approx"),
        "metrics.jsonl should contain tokens-approx field.\nContent:\n{}",
        metrics
    );
}

#[test]
fn tc_203_context_measure_idempotent() {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-001-first.md",
        "---\nid: ADR-001\ntitle: First\nstatus: accepted\nfeatures: [FT-001]\n---\n\nADR body.\n",
    );

    // First run
    let out1 = h.run(&["context", "FT-001", "--measure"]);
    out1.assert_exit(0);

    // Second run
    let out2 = h.run(&["context", "FT-001", "--measure"]);
    out2.assert_exit(0);

    // metrics.jsonl should have exactly 2 lines (one per invocation)
    let metrics = h.read("metrics.jsonl");
    let lines: Vec<&str> = metrics.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len(),
        2,
        "metrics.jsonl should have 2 entries (one per invocation). Got: {}",
        lines.len()
    );

    // Front-matter should have only one bundle block (the most recent)
    let content = h.read("docs/features/FT-001-test.md");
    let bundle_count = content.matches("measured-at:").count();
    assert_eq!(
        bundle_count, 1,
        "Feature front-matter should have exactly one measured-at field (most recent). Got: {}",
        bundle_count
    );
}

#[test]
fn tc_474_context_bundle_includes_responsibility_in_header() {
    let h = fixture_with_responsibility();
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("product\u{225c}picloud:Product"),
        "bundle should contain product line: {}", out.stdout);
    assert!(out.stdout.contains("responsibility\u{225c}"),
        "bundle should contain responsibility line: {}", out.stdout);
    assert!(out.stdout.contains("private cloud platform"),
        "responsibility should contain the statement: {}", out.stdout);
    // Verify product and responsibility appear before feature line
    let product_pos = out.stdout.find("product\u{225c}").unwrap_or(usize::MAX);
    let feature_pos = out.stdout.find("feature\u{225c}").unwrap_or(0);
    assert!(product_pos < feature_pos, "product should appear before feature in header");
}

#[test]
fn tc_477_context_bundle_omits_responsibility_when_field_not_configured() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ncontent-hash: sha256:041d699c4fbf6ed027d18d01345d5dbc758c222150d9ae85257d83e98ccf3ede\n---\n\nBody.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n");
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    assert!(!out.stdout.contains("product\u{225c}"), "should not contain product line when unconfigured: {}", out.stdout);
    assert!(!out.stdout.contains("responsibility\u{225c}"), "should not contain responsibility line when unconfigured: {}", out.stdout);
}

#[test]
fn tc_482_context_measure_all_measures_all_features() {
    let h = fixture_bundle_summary();
    let out = h.run(&["context", "--measure-all"]);
    out.assert_exit(0);

    // All 3 feature files should now contain bundle blocks.
    for (path, id) in &[
        ("docs/features/FT-001-alpha.md", "FT-001"),
        ("docs/features/FT-002-beta.md", "FT-002"),
        ("docs/features/FT-003-gamma.md", "FT-003"),
    ] {
        let content = h.read(path);
        assert!(
            content.contains("bundle:"),
            "{} should have bundle block.\nContent:\n{}",
            id,
            content
        );
        assert!(
            content.contains("tokens-approx:"),
            "{} should have tokens-approx.\nContent:\n{}",
            id,
            content
        );
    }

    // metrics.jsonl should have one entry per feature.
    let metrics = h.read("metrics.jsonl");
    assert!(metrics.contains("FT-001"), "metrics.jsonl missing FT-001: {}", metrics);
    assert!(metrics.contains("FT-002"), "metrics.jsonl missing FT-002: {}", metrics);
    assert!(metrics.contains("FT-003"), "metrics.jsonl missing FT-003: {}", metrics);
    let lines: Vec<&str> = metrics.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 3, "Expected 3 lines in metrics.jsonl, got {}", lines.len());
}

#[test]
fn tc_483_context_measure_all_with_depth_flag() {
    let h = fixture_bundle_summary();

    // First run with depth 1.
    let out1 = h.run(&["context", "--measure-all"]);
    out1.assert_exit(0);
    let content_d1 = h.read("docs/features/FT-001-alpha.md");
    let tokens_d1 = extract_tokens_approx(&content_d1);

    // Second run with depth 2 — shared ADR-001 means depth-2 pulls in adjacent features.
    let out2 = h.run(&["context", "--measure-all", "--depth", "2"]);
    out2.assert_exit(0);
    let content_d2 = h.read("docs/features/FT-001-alpha.md");
    let tokens_d2 = extract_tokens_approx(&content_d2);

    // Depth 2 should produce a bundle at least as large as depth 1.
    assert!(
        tokens_d2 >= tokens_d1,
        "Depth 2 bundle ({}) should be >= depth 1 bundle ({}) for shared-ADR graph.\nd1:\n{}\n\nd2:\n{}",
        tokens_d2, tokens_d1, content_d1, content_d2
    );
    // And exit 0 plus front-matter updated.
    assert!(content_d2.contains("bundle:"), "FT-001 should still have bundle block after --depth 2");
}

#[test]
fn tc_484_context_measure_all_prints_summary_not_bundles() {
    let h = fixture_bundle_summary();
    let out = h.run(&["context", "--measure-all"]);
    out.assert_exit(0);

    // Aggregate table lines on stdout.
    out.assert_stdout_contains("Bundle size");
    out.assert_stdout_contains("measured:");
    out.assert_stdout_contains("mean:");
    out.assert_stdout_contains("median:");

    // Individual bundle content should NOT be on stdout.
    assert!(
        !out.stdout.contains("# Context Bundle:"),
        "measure-all must not flood stdout with bundle content. Got:\n{}",
        out.stdout
    );
    // Nor the AISP bundle header marker.
    assert!(
        !out.stdout.contains("\u{27E6}\u{03A9}:Bundle\u{27E7}"),
        "measure-all must not print AISP bundle headers. Got:\n{}",
        out.stdout
    );
}

#[test]
fn tc_612_bundle_type_ordering_exit_criteria_first() {
    let h = Harness::new();
    ft048_write_feature(&h, "FT-001", 1, &["TC-099", "TC-004", "TC-003", "TC-002", "TC-001", "TC-005"]);
    ft048_write_tc(&h, "TC-001", "X", "exit-criteria", "passing", "FT-001", 1);
    h.write(
        "docs/tests/TC-002.md",
        "---\nid: TC-002\ntitle: Inv\ntype: invariant\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ x }\n",
    );
    h.write(
        "docs/tests/TC-003.md",
        "---\nid: TC-003\ntitle: Ch\ntype: chaos\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\n\u{27E6}\u{0393}:Invariants\u{27E7}{ y }\n",
    );
    h.write(
        "docs/tests/TC-004.md",
        "---\nid: TC-004\ntitle: Ab\ntype: absence\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nBody\n",
    );
    ft048_write_tc(&h, "TC-005", "Sc", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-099", "Bn", "benchmark", "passing", "FT-001", 1);
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    let order = ["TC-001", "TC-002", "TC-003", "TC-004", "TC-005", "TC-099"];
    let mut last = 0usize;
    for id in order {
        let pos = out.stdout.find(id).unwrap_or_else(|| panic!("{} missing", id));
        assert!(pos >= last, "{} pos={} vs last={}", id, pos, last);
        last = pos;
    }
}

#[test]
fn tc_613_bundle_type_ordering_custom_types_last_alphabetical() {
    let h = ft048_tc_types(&["migration", "contract", "smoke"]);
    ft048_write_feature(&h, "FT-001", 1, &["TC-001", "TC-002", "TC-003", "TC-004", "TC-005"]);
    ft048_write_tc(&h, "TC-001", "Sa", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-002", "Sb", "scenario", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-003", "M", "migration", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-004", "C", "contract", "passing", "FT-001", 1);
    ft048_write_tc(&h, "TC-005", "S", "smoke", "passing", "FT-001", 1);
    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    let pc = out.stdout.find("TC-004").expect("contract");
    let pm = out.stdout.find("TC-003").expect("migration");
    let ps = out.stdout.find("TC-005").expect("smoke");
    let p_sa = out.stdout.find("TC-001").expect("sa");
    let p_sb = out.stdout.find("TC-002").expect("sb");
    assert!(p_sa < pc && p_sb < pc, "scenarios before custom");
    assert!(pc < pm && pm < ps, "custom alphabetical");
}

#[test]
fn tc_693_context_bundle_includes_full_functional_spec() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            COMPLETE_BODY
        ),
    );

    let out = h.run(&["context", "FT-001", "--depth", "2"]);
    out.assert_exit(0);
    for needle in [
        "### Inputs",
        "### Outputs",
        "### State",
        "### Behaviour",
        "### Invariants",
        "### Error handling",
        "### Boundaries",
        "## Out of scope",
    ] {
        assert!(
            out.stdout.contains(needle),
            "expected '{}' in context output:\n{}",
            needle,
            out.stdout
        );
    }
}

#[test]
fn tc_694_context_bundle_preserves_subsection_structure() {
    let h = Harness::new();
    h.write("product.toml", CONFIG_W030_DEFAULT);
    let body = "\
## Description

prose

## Functional Specification

### Inputs

```yaml
key: value
```

### Outputs

| col1 | col2 |
| --- | --- |
| a | b |

### State

stateless

### Behaviour

1. step one

### Invariants

- p

### Error handling

err

### Boundaries

edges

## Out of scope

nothing
";
    h.write(
        "docs/features/FT-001-x.md",
        &format!(
            "---\nid: FT-001\ntitle: X\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {{}}\n---\n\n{}",
            body
        ),
    );

    let out = h.run(&["context", "FT-001"]);
    out.assert_exit(0);
    // Code fence preserved verbatim.
    assert!(out.stdout.contains("```yaml"), "expected fenced yaml; stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("key: value"));
    // Table preserved.
    assert!(out.stdout.contains("| col1 | col2 |"));
    // H3 not promoted/demoted.
    assert!(out.stdout.contains("### Inputs"));
    assert!(out.stdout.contains("## Out of scope"));
}

