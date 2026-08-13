//! The binary's surface: scaffolding, root resolution, exit-code discipline.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

fn ledger(root: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root").arg(root).args(args);
    cmd.output().expect("run")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn init_scaffolds_the_layout_then_verify_passes_over_the_empty_store() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = ledger(dir.path(), &["init"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(dir.path().join(".decisions/sets").is_dir());
    assert!(dir.path().join(".decisions/log").is_dir());
    let ignore = std::fs::read_to_string(dir.path().join(".gitignore")).expect("read");
    assert!(ignore.contains(".decisions/index/"), "{ignore}");

    let verified = ledger(dir.path(), &["verify"]);
    assert_eq!(verified.status.code(), Some(0), "{}", stderr(&verified));
    assert!(stdout(&verified).contains("conformant"), "{}", stdout(&verified));
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    ledger(dir.path(), &["init"]);
    let second = ledger(dir.path(), &["init"]);
    assert_eq!(second.status.code(), Some(0));
    assert!(stdout(&second).contains("already initialised"), "{}", stdout(&second));
}

#[test]
fn a_gate_that_could_not_run_exits_two_not_one() {
    // CI has to tell "the gate said no" apart from "the gate broke".
    let dir = tempfile::tempdir().expect("tempdir");
    ledger(dir.path(), &["init"]);
    let out = ledger(dir.path(), &["verify", "--gate", "release"]);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("is not a gate"), "{}", stderr(&out));
}

#[test]
fn a_malformed_date_exits_two() {
    let dir = tempfile::tempdir().expect("tempdir");
    ledger(dir.path(), &["init"]);
    let out = ledger(dir.path(), &["verify", "--today", "yesterday"]);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
}

#[test]
fn verifying_without_a_store_says_how_to_make_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.current_dir(dir.path()).arg("verify");
    let out = cmd.output().expect("run");
    // A tempdir may sit under a parent that has a store, so accept either
    // the missing-store error or a clean run; what must not happen is a
    // panic or a violation exit.
    assert!(matches!(out.status.code(), Some(0) | Some(2)), "{}", stderr(&out));
    if out.status.code() == Some(2) {
        assert!(stderr(&out).contains("ledger init"), "{}", stderr(&out));
    }
}

#[test]
fn the_surface_is_the_format_the_verbs_the_graph_and_merge() {
    // L1's verbs, L2's graph commands, and L3's merge/diff are here; L4's
    // federation is deliberately absent — adding a command is a milestone
    // decision, not a drive-by. The merge driver itself is plumbing and
    // stays out of the help.
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    let out = cmd.arg("--help").output().expect("run");
    let text = stdout(&out);
    for present in [
        "init", "verify", "accept", "add", "allocate", "revoke", "show", "status", "supersede",
        "diff", "merge",
    ] {
        assert!(text.contains(&format!("  {present}")), "{present} is missing: {text}");
    }
    for later in ["fetch", "instruments", "import", "upstreams"] {
        assert!(!text.contains(&format!("  {later}")), "{later} is not shipped yet: {text}");
    }
    assert!(!text.contains("merge-driver"), "the driver is plumbing, not surface: {text}");
}
