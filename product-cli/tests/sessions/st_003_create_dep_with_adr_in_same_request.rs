//! ST-003 — creating a dep together with its governing ADR satisfies E013.
//!
//! Validates TC-535.

use super::harness::Session;

/// TC-535 — session ST-003 create-dep-with-adr-in-same-request satisfies E013.
#[test]
fn tc_535_session_st_003_create_dep_with_adr_in_same_request_satisfies_e013() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-003 — dep + governing ADR together"
artifacts:
  - type: adr
    ref: adr-g
    title: Governance
    domains: [api]
    scope: domain
    governs: [ref:dep-tool]
  - type: dep
    ref: dep-tool
    title: Chosen Tool
    dep-type: library
    version: ">=1"
    adrs: [ref:adr-g]
"#,
    );
    r.assert_applied();
    r.assert_no_finding("E013");

    let adr = r.id_for("adr-g");
    let dep = r.id_for("dep-tool");

    let dep_file = format!("docs/dependencies/{}-chosen-tool.md", dep);
    let adr_file = format!("docs/adrs/{}-governance.md", adr);
    s.assert_file_exists(&dep_file);
    s.assert_file_exists(&adr_file);

    // The dep links back to the ADR (forward ref resolved).
    s.assert_array_contains(&dep_file, "adrs", &adr);

    s.assert_graph_clean();
}
