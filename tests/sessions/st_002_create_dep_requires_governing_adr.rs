//! ST-002 — creating a dependency without a governing ADR emits E013.
//!
//! Validates TC-534.

use super::harness::Session;

/// TC-534 — session ST-002 create-dep-requires-governing-adr emits E013.
#[test]
fn tc_534_session_st_002_create_dep_requires_governing_adr_emits_e013() {
    let mut s = Session::new();

    let digest_before = s.docs_digest();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-002 — dep without ADR"
artifacts:
  - type: dep
    title: Orphaned Library
    dep-type: library
    version: ">=1"
"#,
    );
    r.assert_failed();
    r.assert_finding("E013");

    // Zero files changed
    let digest_after = s.docs_digest();
    assert_eq!(
        digest_before, digest_after,
        "failed apply must leave docs/ unchanged"
    );
}
