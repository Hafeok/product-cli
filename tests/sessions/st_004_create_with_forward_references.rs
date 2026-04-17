//! ST-004 — forward-reference values are resolved in topological order.
//!
//! Validates TC-536. A five-artifact DAG with ref values in both scalar
//! and array fields. After apply, no `ref:xxx` placeholder survives in
//! any written file.

use super::harness::Session;

/// TC-536 — session ST-004 create-with-forward-references resolves ref values.
#[test]
fn tc_536_session_st_004_create_with_forward_references_resolves_ref_values() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-004 — forward refs resolve in topological order"
artifacts:
  - type: feature
    ref: ft-a
    title: Alpha
    phase: 2
    domains: [api]
    adrs: [ref:adr-b, ref:adr-c]
    tests: [ref:tc-d]
    uses: [ref:dep-e]
  - type: adr
    ref: adr-b
    title: Bravo
    domains: [api]
    scope: domain
  - type: adr
    ref: adr-c
    title: Charlie
    domains: [api]
    scope: domain
    governs: [ref:dep-e]
  - type: tc
    ref: tc-d
    title: Delta
    tc-type: scenario
    validates:
      features: [ref:ft-a]
      adrs: [ref:adr-b]
  - type: dep
    ref: dep-e
    title: Echo
    dep-type: service
    version: ">=1"
    adrs: [ref:adr-c]
"#,
    );
    r.assert_applied();
    assert_eq!(r.created.len(), 5);

    let ft_a = r.id_for("ft-a");
    let adr_b = r.id_for("adr-b");
    let adr_c = r.id_for("adr-c");
    let tc_d = r.id_for("tc-d");
    let dep_e = r.id_for("dep-e");

    // No `ref:` string remains in any created file.
    for (_id, file) in r.created.iter().map(|c| (c.id.clone(), c.file.clone())) {
        let body = std::fs::read_to_string(&file).expect("read created file");
        assert!(
            !body.contains("ref:"),
            "{} still contains ref: placeholder\n{}",
            file,
            body
        );
    }

    // Cross-links are real IDs now.
    let ft_file = format!("docs/features/{}-alpha.md", ft_a);
    s.assert_array_contains(&ft_file, "adrs", &adr_b);
    s.assert_array_contains(&ft_file, "adrs", &adr_c);
    s.assert_array_contains(&ft_file, "tests", &tc_d);

    // Feature↔Dep is stored on the dep side as a back-link (no `uses` in feature front-matter).
    let dep_file = format!("docs/dependencies/{}-echo.md", dep_e);
    s.assert_array_contains(&dep_file, "adrs", &adr_c);
    s.assert_array_contains(&dep_file, "features", &ft_a);

    // TC's `validates` targets real IDs.
    let tc_file = format!("docs/tests/{}-delta.md", tc_d);
    let tc_body = s.read(&tc_file);
    assert!(tc_body.contains(&ft_a), "TC should reference feature {}", ft_a);
    assert!(tc_body.contains(&adr_b), "TC should reference ADR {}", adr_b);

    s.assert_graph_clean();
}
