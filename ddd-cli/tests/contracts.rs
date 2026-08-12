//! `ddd diff-contracts` acceptance: the repo-diff half of the shared
//! classifier (M8), driven through the real binary over a temp git repo.

use std::path::Path;
use std::process::Command as Proc;

use assert_cmd::Command;
use predicates::prelude::*;

fn ddd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ddd").expect("binary");
    cmd.current_dir(dir);
    cmd
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Proc::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?}");
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    std::fs::create_dir_all(dir.join(".ddd")).expect("mkdir");
}

fn write(dir: &Path, rel: &str, text: &str) {
    std::fs::write(dir.join(rel), text).expect("write");
}

fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", msg]);
}

#[test]
fn out_of_band_contract_change_is_classified_from_git_alone() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "page.css", ":root { --ink: #111; }\n");
    commit_all(dir, "before");
    // The out-of-band change: an editor edit committed with git — the
    // governed path never sees it.
    write(dir, "page.css", ":root { --ink: #111; --gap: 4px; }\n");
    commit_all(dir, "after");

    ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD"])
        .assert()
        .success()
        .stdout(predicate::str::contains("contract/htmlcss/page.css#--gap@added"))
        .stdout(predicate::str::contains("contract-surface event(s)"));
}

#[test]
fn json_report_carries_stable_finding_ids_and_hashes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "page.css", ":root { --ink: #111; }\n");
    commit_all(dir, "before");
    // A token changing type (color -> length) is a signature change;
    // a value-only change within one type is not contract surface.
    write(dir, "page.css", ":root { --ink: 4px; }\n");
    commit_all(dir, "after");

    let out = ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&out).expect("json");
    let files = report["files"].as_array().expect("files");
    assert_eq!(files.len(), 1);
    assert!(files[0]["before"].as_str().expect("before").starts_with("sha256:"));
    assert!(files[0]["after"].as_str().expect("after").starts_with("sha256:"));
    let ids: Vec<&str> = files[0]["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter_map(|e| e["id"].as_str())
        .collect();
    assert!(ids.contains(&"contract/htmlcss/page.css#--ink@signature-changed"), "{ids:?}");
}

#[test]
fn clean_range_reports_no_contract_surface() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "notes.md", "prose\n");
    commit_all(dir, "before");
    write(dir, "notes.md", "more prose\n");
    commit_all(dir, "after");

    ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no contract-surface changes"));
}
