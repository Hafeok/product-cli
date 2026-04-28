//! Integration tests — planning.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_644_planning_due_date_and_started_tag_exit() {
    // 1. due-date field parses.
    let h = fixture_with_domains();
    h.write(
        "docs/features/FT-009-seed.md",
        "---\nid: FT-009\ntitle: Seed\nphase: 1\nstatus: planned\ndue-date: \"2026-05-01\"\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\n## Description\n\nSeed.\n",
    );
    let out = h.run(&["graph", "check"]);
    assert!(
        !out.stderr.contains("E001") && !out.stderr.contains("E006"),
        "valid due-date should not trigger E-class: {}",
        out.stderr
    );

    // 2. status renders due-date column.
    let status_out = h.run(&["status"]);
    status_out.assert_stdout_contains("2026-05-01");

    // 3. Tag list accepts --type started.
    git_init(&h);
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("add");
    std::process::Command::new("git")
        .args(["commit", "-m", "seed"])
        .current_dir(h.dir.path())
        .output()
        .expect("commit");
    std::process::Command::new("git")
        .args([
            "tag",
            "-a",
            "product/FT-009/started",
            "-m",
            "FT-009 started: status changed to in-progress",
        ])
        .current_dir(h.dir.path())
        .output()
        .expect("tag");
    let tag_out = h.run(&["tags", "list", "--type", "started"]);
    tag_out.assert_exit(0);
    tag_out.assert_stdout_contains("FT-009");
    tag_out.assert_stdout_contains("started");
}

