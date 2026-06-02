//! ST-041 — after every phase-N exit-criteria TC passes, the gate opens and
//! `product feature next` surfaces a phase-(N+1) feature.
//!
//! Validates TC-676.

use super::harness::Session;
use super::repo_scaffold::{write_adr, write_feature, write_tc};

/// TC-676 — session ST-041 phase-gate-opens-after-verify.
#[test]
fn tc_676_session_st_041_phase_gate_opens_after_verify() {
    let s = Session::new();

    write_adr(&s, "ADR-001", "Core", "accepted");
    // Phase 1 feature with a PASSING exit-criteria TC (complete).
    write_feature(&s, "FT-001", "Phase One Feature", 1, "complete", &["ADR-001"], &["TC-001"]);
    write_tc(
        &s,
        "TC-001",
        "Phase One Exit",
        "exit-criteria",
        "FT-001",
        "",
        "",
        "passing",
    );
    // Phase 2 feature, ready to be picked up.
    write_feature(&s, "FT-002", "Phase Two Feature", 2, "planned", &["ADR-001"], &[]);

    let out = s.run(&["feature", "next"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("FT-002"),
        "expected FT-002 to be offered as next when phase 1 gate is open; stdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}
