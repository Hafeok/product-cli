//! End-to-end scenarios over the `spec` binary.
//!
//! The write-back leg is the thing under test: a slice can be built
//! unattended, and it cannot be closed unattended. Every assertion here is
//! about the exit code a CI system or an agent harness actually reads.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;

/// A git repo with an initial commit, so an opening has a revision to pin.
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for args in [
        vec!["init", "-q", "."],
        vec!["config", "user.email", "emil@example.com"],
        vec!["config", "user.name", "Emil"],
        vec!["commit", "-q", "--allow-empty", "-m", "init"],
    ] {
        Command::new("git")
            .current_dir(dir.path())
            .args(&args)
            .status()
            .expect("git runs");
    }
    dir
}

fn spec(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("spec").expect("the spec binary builds");
    cmd.arg("--root").arg(root);
    cmd
}

fn code(output: &std::process::Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn open_a_record(root: &Path) -> String {
    spec(root)
        .args(["implement", "--slice", "checkout-totals", "--act", "act/settle-basket"])
        .args(["--by", "agent@example.invalid"])
        .output()
        .expect("implement runs");
    let listed = spec(root).args(["records", "--json"]).output().expect("records runs");
    let parsed: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("records emits json");
    parsed[0]["record"].as_str().expect("a record id").to_string()
}

#[test]
fn an_empty_store_is_conformant() {
    let dir = repo();
    let out = spec(dir.path()).arg("check").output().expect("check runs");
    assert_eq!(code(&out), 0);
}

#[test]
fn implement_never_exits_conformant() {
    let dir = repo();
    let out = spec(dir.path())
        .args(["implement", "--slice", "s", "--act", "act/a", "--by", "agent@example.invalid"])
        .output()
        .expect("implement runs");
    assert_eq!(code(&out), 3, "the write-back has not happened yet");
}

#[test]
fn check_fails_while_a_record_is_open() {
    let dir = repo();
    open_a_record(dir.path());
    let out = spec(dir.path()).arg("check").output().expect("check runs");
    assert_eq!(code(&out), 1);
    assert!(String::from_utf8_lossy(&out.stdout).contains("S001"));
}

#[test]
fn a_machine_cannot_close_a_record() {
    let dir = repo();
    let id = open_a_record(dir.path());
    let out = spec(dir.path())
        .args(["close", &id, "--principal", "ci@example.com", "--nothing-arose"])
        .output()
        .expect("close runs");
    assert_eq!(code(&out), 1, "a refused close reports a finding, not a crash");
    assert!(String::from_utf8_lossy(&out.stdout).contains("S002"));

    let gate = spec(dir.path()).arg("check").output().expect("check runs");
    assert_eq!(code(&gate), 1, "the refused write must not have landed");
}

#[test]
fn a_principal_closing_with_nothing_arose_clears_the_gate() {
    let dir = repo();
    let id = open_a_record(dir.path());
    let out = spec(dir.path())
        .args(["close", &id, "--principal", "emil@example.com", "--nothing-arose"])
        .output()
        .expect("close runs");
    assert_eq!(code(&out), 0);

    let gate = spec(dir.path()).arg("check").output().expect("check runs");
    assert_eq!(code(&gate), 0);
}

#[test]
fn a_closure_declares_something_or_it_is_refused() {
    let dir = repo();
    let id = open_a_record(dir.path());
    let out = spec(dir.path())
        .args(["close", &id, "--principal", "emil@example.com"])
        .output()
        .expect("close runs");
    assert_eq!(code(&out), 2, "silence is not a closure");
    assert!(String::from_utf8_lossy(&out.stderr).contains("declares something"));
}

#[test]
fn determinations_close_the_record_and_survive_the_gate() {
    let dir = repo();
    let id = open_a_record(dir.path());
    let out = spec(dir.path())
        .args(["close", &id, "--principal", "emil@example.com"])
        .args(["--determination", "det/basket-rounding-is-half-even"])
        .output()
        .expect("close runs");
    assert_eq!(code(&out), 0);

    let gate = spec(dir.path()).args(["check", "--json"]).output().expect("check runs");
    assert_eq!(code(&gate), 0);
}

#[test]
fn editing_an_opened_record_after_closure_breaks_its_binding() {
    let dir = repo();
    let id = open_a_record(dir.path());
    spec(dir.path())
        .args(["close", &id, "--principal", "emil@example.com", "--nothing-arose"])
        .output()
        .expect("close runs");

    let path = dir.path().join(".spec/records").join(format!("{id}.yml"));
    let text = std::fs::read_to_string(&path).expect("read record");
    std::fs::write(&path, text.replace("checkout-totals", "something-else")).expect("tamper");

    let gate = spec(dir.path()).arg("check").output().expect("check runs");
    assert_eq!(code(&gate), 1);
    assert!(String::from_utf8_lossy(&gate.stdout).contains("S003"));
}
