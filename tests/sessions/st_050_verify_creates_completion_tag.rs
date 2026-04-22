//! ST-050 — `product verify <FT>` creates a `product/<FT>/complete` git tag
//! when every linked TC passes.
//!
//! Validates TC-671.

use super::harness::Session;
use super::repo_scaffold::{git, init_git, write_adr, write_exit_script, write_feature, write_tc};
use std::process::{Command, Stdio};

/// TC-671 — session ST-050 verify-creates-completion-tag.
#[test]
fn tc_671_session_st_050_verify_creates_completion_tag() {
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

    // Commit baseline so tags have something to point at.
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);

    let out = s.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // Tag should now exist.
    let tags = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(s.dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("git tag -l");
    let listing = String::from_utf8_lossy(&tags.stdout);
    assert!(
        listing.lines().any(|l| l == "product/FT-001/complete"),
        "expected tag 'product/FT-001/complete'; got tags:\n{listing}"
    );
}
