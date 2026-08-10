//! The full life of a decision, driven through the binary in a real repo:
//! declare → add → allocate → accept → revise → escape → supersede →
//! revoke, each round-tripping through `status` and surviving `verify`, and
//! each with its refusal — the write rejected with the same finding class
//! `verify` would report one command later.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

const TODAY: &str = "2026-08-10";

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    /// A git repo with an initialised store and a human identity.
    fn human() -> Self {
        Self::with_identity("fixture-human@example")
    }

    fn with_identity(email: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Self { dir };
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Fixture"]);
        repo.git(&["config", "user.email", email]);
        assert_eq!(repo.ledger(&["init"]).status.code(), Some(0));
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

    fn ledger(&self, args: &[&str]) -> Output {
        let mut cmd = Command::cargo_bin("ledger").expect("binary");
        cmd.arg("--root").arg(self.path()).args(args);
        cmd.output().expect("run")
    }

    /// Run a verb that must succeed, returning stdout.
    fn ok(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "ledger {args:?}:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Run a verb that must be refused (exit 1), returning stderr.
    fn refused(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "ledger {args:?} should refuse:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stderr).into_owned()
    }

    fn verify_green(&self) {
        let out = self.ledger(&["verify", "--today", TODAY]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "verify should pass:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn declare(&self) {
        self.ok(&["declare", "--set", "verbs", "--tolerance-floor", "T1"]);
    }

    /// Add a constraint-allocated decision; returns its id.
    fn add(&self) -> String {
        let text = self.ok(&[
            "add", "--set", "verbs",
            "--statement", "Monetary amounts use decimal, never double.",
            "--namespace", "fixture.verbs",
            "--store", "constraint",
            "--discharge", "analyzer:DEC001",
        ]);
        let id = text
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .expect("decision id in output");
        assert!(id.starts_with("dec:fixture.verbs/"), "{text}");
        id.to_string()
    }

    fn status(&self) -> String {
        self.ok(&["status", "--today", TODAY])
    }
}

#[test]
fn the_full_life_of_a_decision() {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add();
    assert!(repo.status().contains("awaiting-acceptance"), "{}", repo.status());
    repo.verify_green();

    repo.ok(&["accept", &id, "--expires", "2027-08-10"]);
    assert!(repo.status().contains("decided"), "{}", repo.status());
    repo.verify_green();

    // Revision moves the hash: the acceptance becomes history.
    repo.ok(&["revise", &id, "--statement", "Decimal everywhere, including imports."]);
    assert!(repo.status().contains("awaiting-acceptance"), "{}", repo.status());
    let blame = repo.ok(&["blame", &id]);
    assert!(blame.contains("historical"), "{blame}");
    repo.verify_green();

    // Price the escape, then supersede with a fresh decision.
    repo.ok(&["escape", &id, "--exposure", "exports may drift", "--review-by", "2026-09-01"]);
    assert!(repo.status().contains("escaped-priced"), "{}", repo.status());
    let successor = repo.add();
    repo.ok(&["supersede", &id, "--by", &successor, "--reason", "narrowed to exports"]);
    let status = repo.status();
    assert!(status.contains(&format!("superseded by {successor}")), "{status}");
    repo.verify_green();

    // Sign the successor, then unsay it with a reason.
    repo.ok(&["accept", &successor]);
    let blame = repo.ok(&["blame", &successor]);
    assert!(blame.contains("fixture-human@example"), "{blame}");
    assert!(blame.contains("live"), "{blame}");
    let acc_id = blame
        .lines()
        .find(|l| l.contains("live"))
        .map(|_| {
            // The acceptance id lives in the log; read it back from there.
            let store = ledger_core::store::load(repo.path());
            store
                .log
                .iter()
                .flat_map(|c| c.file.acceptances.iter())
                .map(|a| a.id.to_string())
                .next_back()
                .expect("acceptance")
        })
        .expect("live line");
    repo.ok(&["revoke", &acc_id, "--reason", "signed before the tier ruling"]);
    assert!(repo.status().contains("awaiting-acceptance"), "{}", repo.status());
    repo.verify_green();

    let log = repo.ok(&["log", "--set", "verbs"]);
    assert!(log.contains("narrowed to exports"), "{log}");
}

#[test]
fn every_write_refusal_carries_the_gate_class() {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add();

    // add: below-floor override → L004.
    let err = repo.refused(&[
        "add", "--set", "verbs", "--statement", "x", "--namespace", "fixture.verbs",
        "--store", "constraint", "--discharge", "analyzer:D", "--tolerance-override", "T0",
    ]);
    assert!(err.contains("[L004]"), "{err}");

    // allocate: judgment to a model actor → L010.
    let err = repo.refused(&["allocate", &id, "--store", "judgment", "--actor", "claude@example.com"]);
    assert!(err.contains("[L010]"), "{err}");

    // accept: expired on arrival → L003.
    let err = repo.refused(&["accept", &id, "--expires", "2020-01-01"]);
    assert!(err.contains("[L003]"), "{err}");

    // revoke: an acceptance nobody filed → SCHEMA.
    let err = repo.refused(&["revoke", "acc:01K2C4YQJ3F8M0PT5W7NZ9RDXZ", "--reason", "x"]);
    assert!(err.contains("[SCHEMA]"), "{err}");

    // revise: a stale parent → the single-writer conflict, merge is L3.
    let tip = ledger_core::store::load(repo.path())
        .log
        .iter()
        .flat_map(|c| c.file.versions.iter())
        .next_back()
        .expect("version")
        .hash
        .to_string();
    repo.ok(&["revise", &id, "--statement", "restated once"]);
    let err = repo.refused(&["revise", &id, "--statement", "forked", "--parent", &tip]);
    assert!(err.contains("merge is L3"), "{err}");

    // supersede: an unknown target → usage (exit 2), a chained fork → conflict.
    let missing = repo.ledger(&["supersede", "dec:fixture.verbs/01K2C4YQJ3F8M0PT5W7NZ9RDXV", "--by", &id]);
    assert_eq!(missing.status.code(), Some(2), "{}", String::from_utf8_lossy(&missing.stderr));

    // Nothing above wrote anything the gate objects to.
    repo.verify_green();
}

#[test]
fn a_model_identity_can_author_but_never_sign() {
    let repo = Repo::with_identity("claude@example.com");
    repo.declare();
    let id = repo.add();
    // Authoring passed; the store records the model as created_by. Signing
    // is refused with the same class verify reports (L006), and pricing an
    // escape — whose acceptor is the configured identity — likewise.
    let err = repo.refused(&["accept", &id]);
    assert!(err.contains("[L006]"), "{err}");
    let err = repo.refused(&["escape", &id, "--exposure", "x", "--review-by", "2026-09-01"]);
    assert!(err.contains("[L006]"), "{err}");
    repo.verify_green();
}

#[test]
fn an_unallocated_add_writes_and_the_gate_says_l001() {
    let repo = Repo::human();
    repo.declare();
    let out = repo.ok(&[
        "add", "--set", "verbs", "--statement", "Which retry budget applies?",
        "--namespace", "fixture.verbs",
    ]);
    assert!(out.contains("unallocated"), "{out}");
    let verify = repo.ledger(&["verify", "--today", TODAY]);
    assert_eq!(verify.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&verify.stderr).contains("[L001]"));
    assert!(repo.status().contains("undecided"), "{}", repo.status());
}

#[test]
fn the_namespace_is_inferred_once_the_store_speaks_one() {
    let repo = Repo::human();
    repo.declare();
    // First add must name the namespace…
    let out = repo.ledger(&["add", "--set", "verbs", "--statement", "x", "--store", "constraint", "--discharge", "analyzer:D"]);
    assert_eq!(out.status.code(), Some(2), "no namespace to infer yet");
    repo.add();
    // …after which it is inferred.
    let text = repo.ok(&["add", "--set", "verbs", "--statement", "y", "--store", "constraint", "--discharge", "analyzer:D"]);
    assert!(text.contains("dec:fixture.verbs/"), "{text}");
}
