//! ST-035 — any declared domain must exist in `[domains]` (apply path).
//!
//! Validates TC-547. This is the apply-time partner of ST-033 (which
//! validates at the validate-only path): even requests that pass all
//! other checks must be rejected at apply when a domain is unknown.

use super::harness::Session;

/// TC-547 — session ST-035 validation-domain-not-in-vocabulary.
#[test]
fn tc_547_session_st_035_validation_domain_not_in_vocabulary() {
    let mut s = Session::new();

    let before = s.docs_digest();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-035 — apply rejects unknown domain"
artifacts:
  - type: feature
    title: Bad
    phase: 1
    domains: [absolutely-not-a-domain]
"#,
    );
    r.assert_failed();
    r.assert_finding("E012");

    // Nothing written.
    let after = s.docs_digest();
    assert_eq!(before, after);
}
