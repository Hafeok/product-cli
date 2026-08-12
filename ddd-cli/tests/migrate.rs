//! `ddd migrate-ledger` acceptance: historical ids resolve through the
//! permanent concordance, migrated entries await acceptance, both gates
//! stay conformant, and the migration never runs twice.

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

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, text).expect("write");
}

/// A miniature governed repo: one claim, one decision pinning it, one
/// seam, a ledger store, all committed.
fn repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    write(
        dir,
        ".ddd/claims/DDD-t-01.yaml",
        "format: 1\nid: DDD-t-01\nstatement: T holds.\nstatus: reported\nevidence: seen\nfalsifier: T fails\nowner: none\nchanged: 2026-08-01\n",
    );
    write(
        dir,
        ".ddd/decisions/t-rules.yaml",
        "format: 2\nid: dec/t/rules\ntitle: T is ruled\nrationale: because T holds\nprincipal: Emil (Context&)\nbased_on:\n  - claim: DDD-t-01\n    status: reported\n    changed: 2026-08-01\n",
    );
    write(
        dir,
        ".ddd/seams/seam-t-api.yaml",
        "format: 1\nid: seam/t/api\nboundary: fn t in src/t.rs\nverdict_knowledge: callers learn t\ncontract_location: src/t.rs#t\n",
    );
    write(dir, ".decisions/sets/.keep", "");
    std::fs::create_dir_all(dir.join(".decisions/log")).expect("mkdir");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", "init"]);
    tmp
}

#[test]
fn migration_files_entries_writes_the_concordance_and_refuses_a_second_run() {
    let tmp = repo();
    let dir = tmp.path();
    ddd(dir)
        .args(["migrate-ledger", "--namespace", "t.ddd", "--set", "t-governance"])
        .assert()
        .success()
        .stdout(predicate::str::contains("awaiting acceptance"))
        .stdout(predicate::str::contains("<- dec/t/rules (decision)"))
        .stdout(predicate::str::contains("<- seam/t/api (seam)"));

    // The concordance is written and resolves both ways through why.
    let concordance = std::fs::read_to_string(dir.join(".ddd/concordance.yaml")).expect("read");
    assert!(concordance.contains("historical: dec/t/rules"), "{concordance}");
    let ledger_id = concordance
        .lines()
        .skip_while(|l| !l.contains("historical: dec/t/rules"))
        .find_map(|l| l.trim().strip_prefix("ledger: "))
        .expect("ledger id")
        .to_string();
    ddd(dir)
        .args(["why", "dec/t/rules"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ledger identity: dec:t.ddd/"));
    ddd(dir)
        .args(["why", &ledger_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("resolved from dec:t.ddd/"));

    // Both gates conformant; entries pending, none accepted.
    ddd(dir).arg("validate").assert().success();
    let ledger = Command::cargo_bin("ledger").expect("binary").current_dir(dir).arg("verify").assert().success();
    let out = String::from_utf8_lossy(&ledger.get_output().stdout).to_string();
    assert!(out.contains("awaiting acceptance"), "{out}");
    let log_text: String = std::fs::read_dir(dir.join(".decisions/log"))
        .expect("log")
        .flatten()
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect();
    assert!(!log_text.contains("acceptances:"), "migration must file no acceptances");
    assert!(log_text.contains("claim:DDD-t-01@sha256:"), "pins upgrade to content hashes");
    assert!(log_text.contains("contract:seam/t/api"), "seams ride the contract discharge");

    // Permanent means never re-run over it.
    ddd(dir)
        .args(["migrate-ledger", "--namespace", "t.ddd", "--set", "t-governance"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already"));
}
