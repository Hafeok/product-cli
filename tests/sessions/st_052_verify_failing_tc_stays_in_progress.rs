//! ST-052 — when a TC runner fails, `product verify` marks the TC failing,
//! leaves the feature in its non-complete status, and writes no completion tag.
//!
//! Validates TC-673.

use super::harness::Session;
use super::repo_scaffold::{git, init_git, write_adr, write_exit_script, write_feature, write_tc};
use std::process::{Command, Stdio};

/// TC-673 — session ST-052 verify-failing-tc-stays-in-progress.
#[test]
fn tc_673_session_st_052_verify_failing_tc_stays_in_progress() {
    let s = Session::new();
    init_git(&s);

    write_adr(&s, "ADR-001", "Core Decision", "accepted");
    let script = write_exit_script(&s, "fail", 1);
    write_feature(
        &s,
        "FT-001",
        "Broken Feature",
        1,
        "in-progress",
        &["ADR-001"],
        &["TC-001"],
    );
    write_tc(
        &s,
        "TC-001",
        "Failing Exit",
        "exit-criteria",
        "FT-001",
        "bash",
        &script,
        "unimplemented",
    );
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);

    // Per-feature `verify FT-XXX` can exit 0 even when a TC fails — it's a
    // reporting view, not a gate. The observable failure signal is the TC's
    // recorded status, the feature's non-advancement, and the absence of a
    // completion tag.
    s.run(&["verify", "FT-001"]);

    // Feature status has NOT advanced to complete.
    let ft_body = s.read("docs/features/FT-001-broken-feature.md");
    assert!(
        !ft_body.contains("status: complete"),
        "feature must not be marked complete when a TC fails; body:\n{ft_body}"
    );

    // TC status has been recorded as failing.
    s.assert_frontmatter("docs/tests/TC-001-failing-exit.md", "status", "failing");

    // No completion tag.
    let tags = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(s.dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("git tag -l");
    let listing = String::from_utf8_lossy(&tags.stdout);
    assert!(
        !listing.lines().any(|l| l.starts_with("product/FT-001/complete")),
        "no completion tag should exist for a feature with failing TCs; got:\n{listing}"
    );
}
