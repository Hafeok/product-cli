//! ST-054 — `product drift check` emits W020 when a complete feature has no
//! completion tag to bound structural drift against.
//!
//! Validates TC-674.

use super::harness::Session;
use super::repo_scaffold::{git, init_git, write_adr, write_feature};

/// TC-674 — session ST-054 drift-check-no-tag-emits-w020.
#[test]
fn tc_674_session_st_054_drift_check_no_tag_emits_w020() {
    let s = Session::new();
    init_git(&s);

    // Complete feature + accepted ADR, but no completion tag exists.
    write_adr(&s, "ADR-001", "Core Decision", "accepted");
    write_feature(
        &s,
        "FT-001",
        "Untagged Feature",
        1,
        "complete",
        &["ADR-001"],
        &[],
    );
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);

    let out = s.run(&["drift", "check", "FT-001"]);
    assert!(
        out.stderr.contains("W020") || out.stdout.contains("W020"),
        "expected W020 in drift check output; stdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}
