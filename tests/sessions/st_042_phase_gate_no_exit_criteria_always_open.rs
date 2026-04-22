//! ST-042 — a phase with zero exit-criteria TCs is always open; `feature next`
//! advances past it without regard to in-progress features in that phase.
//!
//! Validates TC-677.

use super::harness::Session;
use super::repo_scaffold::{write_adr, write_feature};

/// TC-677 — session ST-042 phase-gate-no-exit-criteria-always-open.
#[test]
fn tc_677_session_st_042_phase_gate_no_exit_criteria_always_open() {
    let s = Session::new();

    write_adr(&s, "ADR-001", "Core", "accepted");
    // Phase 1 has a feature but NO exit-criteria TCs. Gate is trivially open.
    write_feature(&s, "FT-001", "Phase One Feature", 1, "complete", &["ADR-001"], &[]);
    // Phase 2 feature ready.
    write_feature(&s, "FT-002", "Phase Two Feature", 2, "planned", &["ADR-001"], &[]);

    let out = s.run(&["feature", "next"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("FT-002"),
        "expected FT-002 to be offered when phase 1 has no exit-criteria; stdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}
