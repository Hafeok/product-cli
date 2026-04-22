//! ST-051 — `product verify <FT>` transitions the feature status to `complete`
//! when every linked TC passes.
//!
//! Validates TC-672.

use super::harness::Session;
use super::repo_scaffold::{git, init_git, write_adr, write_exit_script, write_feature, write_tc};

/// TC-672 — session ST-051 verify-complete-feature-status.
#[test]
fn tc_672_session_st_051_verify_complete_feature_status() {
    let s = Session::new();
    init_git(&s);

    write_adr(&s, "ADR-001", "Core Decision", "accepted");
    let script = write_exit_script(&s, "pass", 0);
    write_feature(
        &s,
        "FT-001",
        "First Feature",
        1,
        "in-progress",
        &["ADR-001"],
        &["TC-001"],
    );
    write_tc(
        &s,
        "TC-001",
        "First Exit",
        "exit-criteria",
        "FT-001",
        "bash",
        &script,
        "unimplemented",
    );
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);

    // Before verify: status is in-progress.
    s.assert_frontmatter("docs/features/FT-001-first-feature.md", "status", "in-progress");

    let out = s.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // After verify: status transitioned to complete.
    s.assert_frontmatter("docs/features/FT-001-first-feature.md", "status", "complete");
}
