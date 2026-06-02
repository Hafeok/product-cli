//! ST-032 — E003 fires when a change introduces a dependency cycle.
//!
//! Validates TC-544. Two features depend on each other through
//! `depends-on`, forming a cycle. `graph check` reports E003.

use super::harness::Session;

/// TC-544 — session ST-032 validation-e003-dep-cycle.
#[test]
fn tc_544_session_st_032_validation_e003_dep_cycle() {
    let mut s = Session::new();

    // Seed two features.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed A and B"
artifacts:
  - type: feature
    ref: ft-a
    title: A
    phase: 1
    domains: [api]
  - type: feature
    ref: ft-b
    title: B
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    // Induce a cycle: A depends-on B, B depends-on A.
    s.apply(
        r#"type: change
schema-version: 1
reason: "ST-032 — cycle"
changes:
  - target: FT-001
    mutations:
      - op: append
        field: depends-on
        value: FT-002
  - target: FT-002
    mutations:
      - op: append
        field: depends-on
        value: FT-001
"#,
    );
    // Regardless of whether the change is blocked at validation or
    // passes through, the resulting graph must fail `graph check` with
    // E003 (cycle).
    s.assert_graph_error("E003");
}
