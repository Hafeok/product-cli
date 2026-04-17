//! ST-034 — E011 fires when a request has no reason (or empty reason).
//!
//! Validates TC-546. Per ADR-038 decision 5, every request must carry a
//! non-empty `reason:`. This also aligns with the ADR-025 principle that
//! domain acknowledgements must carry reasoning — E011 is the shared code.

use super::harness::Session;

/// TC-546 — session ST-034 validation-e011-empty-acknowledgement.
#[test]
fn tc_546_session_st_034_validation_e011_empty_acknowledgement() {
    let mut s = Session::new();

    for body in [
        // Missing reason
        r#"type: create
schema-version: 1
artifacts:
  - type: feature
    title: X
    phase: 1
    domains: [api]
"#,
        // Empty reason
        r#"type: create
schema-version: 1
reason: ""
artifacts:
  - type: feature
    title: X
    phase: 1
    domains: [api]
"#,
        // Whitespace-only reason
        r#"type: create
schema-version: 1
reason: "   "
artifacts:
  - type: feature
    title: X
    phase: 1
    domains: [api]
"#,
    ] {
        let r = s.validate(body);
        r.assert_finding("E011");
        assert_eq!(r.exit_code, 1);
    }
}
