//! ST-031 — E002 fires when a `ref:` value does not resolve.
//!
//! Validates TC-543.

use super::harness::Session;

/// TC-543 — session ST-031 validation-e002-broken-ref.
#[test]
fn tc_543_session_st_031_validation_e002_broken_ref() {
    let mut s = Session::new();

    let r = s.validate(
        r#"type: create
schema-version: 1
reason: "ST-031 — unresolved ref"
artifacts:
  - type: feature
    ref: ft-a
    title: Alpha
    phase: 1
    domains: [api]
    adrs: [ref:does-not-exist]
"#,
    );
    r.assert_finding("E002");
    assert_eq!(r.exit_code, 1);
}
