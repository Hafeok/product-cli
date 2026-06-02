//! ST-033 — E012 fires when a declared domain is not in the vocabulary.
//!
//! Validates TC-545.

use super::harness::Session;

/// TC-545 — session ST-033 validation-e012-unknown-domain.
#[test]
fn tc_545_session_st_033_validation_e012_unknown_domain() {
    let mut s = Session::new();

    let r = s.validate(
        r#"type: create
schema-version: 1
reason: "ST-033 — unknown domain"
artifacts:
  - type: feature
    title: Broken
    phase: 1
    domains: [this-domain-does-not-exist]
"#,
    );
    r.assert_finding("E012");
    assert_eq!(r.exit_code, 1);
}
