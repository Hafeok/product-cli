//! ST-012 — change against a non-existent target fails with E002.
//!
//! Validates TC-667. The request validator must reject unknown target
//! IDs before any writes occur. Combined with ST-020's zero-files
//! invariant, this proves invalid changes are fully atomic.

use super::harness::Session;

/// TC-667 — session ST-012 change-invalid-target.
#[test]
fn tc_667_session_st_012_change_invalid_target() {
    let mut s = Session::new();

    let before = s.docs_digest();

    let r = s.apply(
        r#"type: change
schema-version: 1
reason: "ST-012 — target that does not exist"
changes:
  - target: FT-999
    mutations:
      - op: append
        field: domains
        value: api
"#,
    );
    r.assert_failed();
    r.assert_finding("E002");

    let after = s.docs_digest();
    assert_eq!(
        before, after,
        "docs/ must be byte-identical after a failed change"
    );
}
