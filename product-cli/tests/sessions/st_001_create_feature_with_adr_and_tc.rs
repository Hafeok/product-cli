//! ST-001 — create feature with ADR and TC in a single request.
//!
//! Validates TC-533. This is the canonical onboarding session: an agent
//! creates a new feature, its governing ADR, and an exit-criteria TC in
//! one atomic apply.

use super::harness::Session;

/// TC-533 — session ST-001 create-feature-with-adr-and-tc.
#[test]
fn tc_533_session_st_001_create_feature_with_adr_and_tc() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-001 — bootstrap feature with governing ADR and exit-criteria TC"
artifacts:
  - type: adr
    ref: adr-core
    title: Core Decision
    domains: [api]
    scope: domain
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    domains: [api]
    adrs: [ref:adr-core]
    tests: [ref:tc-exit]
  - type: tc
    ref: tc-exit
    title: Main Exit Criteria
    tc-type: exit-criteria
    validates:
      features: [ref:ft-main]
      adrs: [ref:adr-core]
"#,
    );
    r.assert_applied();
    assert_eq!(r.created.len(), 3);

    let ft = r.id_for("ft-main");
    let adr = r.id_for("adr-core");
    let tc = r.id_for("tc-exit");

    // Files exist on disk
    let ft_file = format!("docs/features/{}-main-feature.md", ft);
    let adr_file = format!("docs/adrs/{}-core-decision.md", adr);
    let tc_file = format!("docs/tests/{}-main-exit-criteria.md", tc);
    s.assert_file_exists(&ft_file);
    s.assert_file_exists(&adr_file);
    s.assert_file_exists(&tc_file);

    // Cross-links resolved
    s.assert_array_contains(&ft_file, "adrs", &adr);
    s.assert_array_contains(&ft_file, "tests", &tc);

    // Bidirectional back-links materialised
    s.assert_array_contains(&adr_file, "features", &ft);

    // Graph is clean after the apply
    s.assert_graph_clean();
}
