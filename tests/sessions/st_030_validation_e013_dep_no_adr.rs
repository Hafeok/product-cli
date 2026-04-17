//! ST-030 — E013 fires when a dep has no linked ADR.
//!
//! Validates TC-542.

use super::harness::Session;

/// TC-542 — session ST-030 validation-e013-dep-no-adr.
#[test]
fn tc_542_session_st_030_validation_e013_dep_no_adr() {
    let mut s = Session::new();

    let r = s.validate(
        r#"type: create
schema-version: 1
reason: "ST-030 — dep without ADR"
artifacts:
  - type: dep
    title: Ungoverned
    dep-type: library
    version: ">=1"
"#,
    );
    r.assert_finding("E013");
    assert_eq!(r.exit_code, 1);
}
