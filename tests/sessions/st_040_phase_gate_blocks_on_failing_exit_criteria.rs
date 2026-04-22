//! ST-040 — `product feature next` reports a phase gate block when a phase-N
//! exit-criteria TC is failing, refusing to advance to phase N+1.
//!
//! Validates TC-675.

use super::harness::Session;
use super::repo_scaffold::{write_adr, write_feature, write_tc};

/// TC-675 — session ST-040 phase-gate-blocks-on-failing-exit-criteria.
#[test]
fn tc_675_session_st_040_phase_gate_blocks_on_failing_exit_criteria() {
    let s = Session::new();

    write_adr(&s, "ADR-001", "Core", "accepted");
    // Phase 1 feature is abandoned so `feature next` doesn't return it itself;
    // but it still owns a FAILING exit-criteria TC, so phase 1's gate is locked.
    write_feature(&s, "FT-001", "Phase One Feature", 1, "abandoned", &["ADR-001"], &["TC-001"]);
    write_tc(
        &s,
        "TC-001",
        "Phase One Exit",
        "exit-criteria",
        "FT-001",
        "",
        "",
        "failing",
    );
    // Phase 2 feature that would be ready — but phase 1's gate is locked.
    write_feature(&s, "FT-002", "Phase Two Feature", 2, "planned", &["ADR-001"], &[]);

    let out = s.run(&["feature", "next"]);
    let combined = format!("{}\n{}", out.stdout, out.stderr);
    assert!(
        combined.to_lowercase().contains("phase")
            && (combined.contains("blocked")
                || combined.contains("locked")
                || combined.contains("gate")),
        "expected phase-gate blocking language from `feature next`; got stdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    // FT-002 must NOT be offered as the next feature.
    assert!(
        !out.stdout.contains("FT-002 is next") && !out.stdout.contains("Next: FT-002"),
        "FT-002 must not be reported as next while phase 1 gate is locked; stdout:\n{}",
        out.stdout
    );
}
