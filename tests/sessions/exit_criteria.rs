//! Exit-criteria aggregator for FT-043 (TC-551).
//!
//! This test is the runner hook for TC-551. Rather than duplicate every
//! linked TC's assertion, it verifies the minimum signal that the Phase 1
//! session library is wired up correctly: the harness types are reachable,
//! the binary is executable from a fresh session, and a minimal end-to-end
//! apply through the harness succeeds. If any linked scenario or invariant
//! TC is broken, `cargo test --test sessions` will fail at the offending
//! test — this aggregator does not replicate that work.

use super::harness::Session;

/// TC-551 — session harness and phase-1 session library pass.
#[test]
fn tc_551_session_harness_and_phase_1_session_library_pass() {
    // The harness types reachable and usable.
    let mut s = Session::new();

    // End-to-end create round-trip through the harness.
    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "FT-043 exit-criteria smoke test"
artifacts:
  - type: feature
    ref: ft-smoke
    title: Smoke
    phase: 1
    domains: [api]
"#,
    );
    r.assert_applied();
    let id = r.id_for("ft-smoke");
    assert!(id.starts_with("FT-"));
    s.assert_graph_clean();
}
