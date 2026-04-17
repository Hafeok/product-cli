//! ST-006 — cross-links between artifacts are written in both directions.
//!
//! Validates TC-538. When a feature links to an ADR, the ADR's
//! `features:` list must contain the feature after the apply completes.

use super::harness::Session;

/// TC-538 — session ST-006 create-cross-links-bidirectional writes both sides.
#[test]
fn tc_538_session_st_006_create_cross_links_bidirectional_writes_both_sides() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-006 — bidirectional links"
artifacts:
  - type: adr
    ref: adr-x
    title: Crosslink Decision
    domains: [api]
    scope: domain
  - type: feature
    ref: ft-x
    title: Crosslink Feature
    phase: 1
    domains: [api]
    adrs: [ref:adr-x]
  - type: tc
    ref: tc-x
    title: Crosslink TC
    tc-type: scenario
    validates:
      features: [ref:ft-x]
      adrs: [ref:adr-x]
"#,
    );
    r.assert_applied();

    let ft = r.id_for("ft-x");
    let adr = r.id_for("adr-x");
    let tc = r.id_for("tc-x");

    let ft_file = format!("docs/features/{}-crosslink-feature.md", ft);
    let adr_file = format!("docs/adrs/{}-crosslink-decision.md", adr);
    let tc_file = format!("docs/tests/{}-crosslink-tc.md", tc);

    // Forward links
    s.assert_array_contains(&ft_file, "adrs", &adr);

    // Back-links materialised: ADR lists the feature; ADR lists the TC (via tests).
    s.assert_array_contains(&adr_file, "features", &ft);

    // TC's validates points to real IDs.
    let tc_body = s.read(&tc_file);
    assert!(tc_body.contains(&ft), "TC should list feature {}", ft);
    assert!(tc_body.contains(&adr), "TC should list adr {}", adr);

    s.assert_graph_clean();
}
