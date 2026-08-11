//! `ledger diff <ref>..<ref>` over a real repository: two commits, the
//! change between them reported decision-level, never textually.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let repo = Self { dir: tempfile::tempdir().expect("tempdir") };
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Fixture Human"]);
        repo.git(&["config", "user.email", "fixture-human@example"]);
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "."]);
        self.git(&["commit", "-m", message, "--no-gpg-sign"]);
    }

    fn ledger(&self, args: &[&str]) -> Output {
        let mut cmd = Command::cargo_bin("ledger").expect("binary");
        cmd.arg("--root").arg(self.path()).args(args);
        cmd.output().expect("run")
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "ledger {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }
}

/// A repo with two commits: `first` holds one allocated decision; `second`
/// adds a revision of it, an acceptance of the revision, and a new set.
fn two_commit_repo() -> (Repo, String) {
    let repo = Repo::new();
    repo.ok(&["init"]);
    repo.ok(&[
        "declare", "--set", "diff-fixture", "--tolerance-floor", "T1",
    ]);
    let add = repo.ok(&[
        "add", "--set", "diff-fixture", "--namespace", "fixture.diff",
        "--statement", "Monetary amounts use decimal, never double.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]);
    let decision = add
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .expect("decision id")
        .to_string();
    repo.commit("first: one allocated decision");

    repo.ok(&[
        "revise", &decision,
        "--statement", "Monetary amounts use decimal everywhere, including imports.",
    ]);
    repo.ok(&["accept", &decision]);
    repo.ok(&[
        "declare", "--set", "second-set", "--tolerance-floor", "T2",
    ]);
    repo.commit("second: revision, acceptance, new set");
    (repo, decision)
}

#[test]
fn the_diff_between_two_commits_is_decision_level() {
    let (repo, decision) = two_commit_repo();
    let text = repo.ok(&["diff", "HEAD~1..HEAD"]);
    assert!(text.contains(&format!("revised {decision}")), "{text}");
    assert!(text.contains("fast-forward"), "a revision of the tip fast-forwards: {text}");
    assert!(text.contains("acceptance gained: fixture-human@example"), "{text}");
    assert!(text.contains("sets declared: second-set (floor T2)"), "{text}");
    assert!(!text.contains("DIVERGED"), "{text}");
}

#[test]
fn the_reverse_diff_states_losses_rather_than_hiding_them() {
    let (repo, _) = two_commit_repo();
    let text = repo.ok(&["diff", "HEAD..HEAD~1"]);
    assert!(text.contains("acceptance lost"), "{text}");
    assert!(text.contains("DIVERGED"), "the old tip does not descend from the new: {text}");
}

#[test]
fn a_single_ref_compares_against_the_working_tree() {
    let (repo, decision) = two_commit_repo();
    // Nothing uncommitted: HEAD vs worktree is empty.
    let clean = repo.ok(&["diff", "HEAD"]);
    assert!(clean.contains("no decision-level change"), "{clean}");
    // An uncommitted escape shows up, with its allocation transition.
    repo.ok(&[
        "escape", &decision,
        "--exposure", "imports may drift silently",
        "--review-by", "2099-09-01",
    ]);
    let dirty = repo.ok(&["diff", "HEAD"]);
    assert!(dirty.contains("constraint -> escaped"), "{dirty}");
}

#[test]
fn the_json_report_is_machine_readable() {
    let (repo, decision) = two_commit_repo();
    let text = repo.ok(&["diff", "HEAD~1..HEAD", "--json"]);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(parsed["tips_moved"][0]["decision"], decision.as_str());
    assert_eq!(parsed["tips_moved"][0]["fast_forward"], true);
    assert_eq!(parsed["acceptances_gained"][0]["actor"], "fixture-human@example");
}

#[test]
fn an_unknown_revision_exits_two_with_the_reason() {
    let (repo, _) = two_commit_repo();
    let out = repo.ledger(&["diff", "no-such-rev..HEAD"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("does not name a commit"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
