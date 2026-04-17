//! ST-020 — failed apply leaves zero files on disk.
//!
//! Validates TC-539 (invariant). This test uses a concrete triggering
//! scenario; TC-548 (TC-P012) asserts the same property via proptest.

use super::harness::Session;

/// TC-539 — session ST-020 failed-apply-leaves-zero-files.
#[test]
fn tc_539_session_st_020_failed_apply_leaves_zero_files() {
    let mut s = Session::new();

    // Seed a feature to give the graph non-empty state.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed"
artifacts:
  - type: feature
    title: Seed
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    let before = s.docs_digest();
    assert!(!before.is_empty(), "docs/ should have content after seed");

    // Attempt a create with an unknown domain — E012.
    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-020 — should fail with E012"
artifacts:
  - type: feature
    title: Doomed
    phase: 1
    domains: [absolutely-unknown-domain]
"#,
    );
    r.assert_failed();
    r.assert_finding("E012");

    let after = s.docs_digest();
    assert_eq!(
        before, after,
        "failed apply must leave docs/ byte-identical"
    );

    // And no new feature file was written.
    assert!(!s.dir.path().join("docs/features/FT-002-doomed.md").exists());
}
