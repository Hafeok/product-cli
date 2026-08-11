//! Two-writer repositories, genuinely divergent, one per conflict class.
//!
//! Each case builds a real repository, forks two branches, works the same
//! ledger on both, merges, and asserts: mechanical divergence merges and
//! stays conformant; every judgment case is surfaced by name and none is
//! resolved automatically; an arbitration through `ledger merge --resolve`
//! is an ordinary recorded act; the reconciled version is unaccepted; and
//! the index rebuild stays byte-identical after the whole affair.

use std::path::{Path, PathBuf};
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
        repo.ok(&["init"]);
        repo.ok(&["declare", "--set", "merge-fixture", "--tolerance-floor", "T1"]);
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) -> Output {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        out
    }

    /// Git where failure is the expected outcome.
    fn git_may_fail(&self, args: &[&str]) -> Output {
        std::process::Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args(args)
            .output()
            .expect("git")
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

    fn ledger_stdin(&self, args: &[&str], stdin: &str) -> Output {
        let mut cmd = Command::cargo_bin("ledger").expect("binary");
        cmd.arg("--root").arg(self.path()).args(args).write_stdin(stdin);
        cmd.output().expect("run")
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "ledger {args:?}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn add_decision(&self, statement: &str) -> String {
        let out = self.ok(&[
            "add", "--set", "merge-fixture", "--namespace", "fixture.merge",
            "--statement", statement,
            "--store", "constraint", "--discharge", "analyzer:DEC001",
        ]);
        out.lines
            ()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .expect("decision id")
            .to_string()
    }

    fn verify(&self) -> Output {
        self.ledger(&["verify", "--no-blame"])
    }

    fn reindex_bytes(&self) -> Vec<u8> {
        self.ok(&["reindex"]);
        std::fs::read(self.path().join(".decisions/index/ledger.ttl")).expect("index")
    }
}

/// Fork `a` and `b` off main, run each closure on its branch, come back to
/// `a`. The merge itself is each test's own business.
fn diverge(repo: &Repo, on_a: impl Fn(&Repo), on_b: impl Fn(&Repo)) {
    repo.git(&["checkout", "-b", "a"]);
    on_a(repo);
    repo.commit("branch a's work");
    repo.git(&["checkout", "-B", "b", "main"]);
    on_b(repo);
    repo.commit("branch b's work");
    repo.git(&["checkout", "a"]);
}

#[test]
fn disjoint_decisions_and_interleaved_logs_merge_mechanically() {
    let repo = Repo::new();
    repo.add_decision("The decision on main.");
    repo.commit("main: one decision");
    diverge(
        &repo,
        |r| {
            r.add_decision("Branch a's own decision.");
        },
        |r| {
            r.add_decision("Branch b's own decision.");
        },
    );
    let plan = repo.ok(&["merge", "b"]);
    assert!(plan.contains("no judgment required"), "{plan}");
    repo.git(&["merge", "b", "--no-edit"]);
    let out = repo.verify();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("3 decision(s)"));
}

#[test]
fn a_divergent_revision_is_named_never_auto_resolved_and_arbitrated_on_record() {
    let repo = Repo::new();
    let id = repo.add_decision("The statement both writers started from.");
    repo.commit("main: the shared decision");
    diverge(
        &repo,
        |r| {
            r.ok(&["revise", &id, "--statement", "The left writer's revision."]);
        },
        |r| {
            r.ok(&["revise", &id, "--statement", "The right writer's revision."]);
        },
    );

    // Before the merge, the plan names the conflict and exits non-zero.
    let plan = repo.ledger(&["merge", "b"]);
    assert_eq!(plan.status.code(), Some(1));
    let plan_text = String::from_utf8_lossy(&plan.stdout).into_owned();
    assert!(plan_text.contains("CONFLICT [divergent-revision]"), "{plan_text}");
    assert!(plan_text.contains("common parent"), "{plan_text}");

    // The git merge unions the append-only logs; the store is now forked.
    repo.git(&["merge", "b", "--no-edit"]);
    let broken = repo.verify();
    assert_eq!(broken.status.code(), Some(1), "an unarbitrated fork must not pass");
    assert!(String::from_utf8_lossy(&broken.stderr).contains("[G004]"));

    // No verb can bury the conflict; nothing resolves by default.
    for verb in [
        vec!["revise", id.as_str(), "--statement", "Extending a tip."],
        vec!["accept", id.as_str()],
        vec!["allocate", id.as_str(), "--store", "judgment", "--actor", "fixture-human@example"],
    ] {
        let out = repo.ledger(&verb);
        assert_eq!(out.status.code(), Some(1), "verb {verb:?} must refuse on a fork");
    }
    let skipped = repo.ledger_stdin(&["merge", "--resolve"], "s\n");
    assert_eq!(skipped.status.code(), Some(1), "skipping resolves nothing");
    assert_eq!(repo.verify().status.code(), Some(1), "still forked after a skip");

    // The arbitration: tip 1 stands. An ordinary act, on the record.
    let resolved = repo.ledger_stdin(&["merge", "--resolve"], "1\n");
    assert_eq!(
        resolved.status.code(),
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&resolved.stdout),
        String::from_utf8_lossy(&resolved.stderr)
    );
    let text = String::from_utf8_lossy(&resolved.stdout).into_owned();
    assert!(text.contains("unaccepted"), "{text}");
    repo.commit("arbitrate the fork");

    let after = repo.verify();
    assert_eq!(after.status.code(), Some(0), "{}", String::from_utf8_lossy(&after.stderr));
    assert!(
        String::from_utf8_lossy(&after.stdout).contains("awaiting acceptance"),
        "the reconciled version is pending, not accepted: {}",
        String::from_utf8_lossy(&after.stdout)
    );
    // The arbitration act is in the log, attributed and self-describing.
    let log = repo.ok(&["log"]);
    assert!(log.contains("merge arbitration: divergent-revision"), "{log}");

    // The rebuild stays byte-identical over the merged, arbitrated store.
    let first = repo.reindex_bytes();
    std::fs::remove_dir_all(repo.path().join(".decisions/index")).expect("delete");
    assert_eq!(first, repo.reindex_bytes(), "state leaked into the cache");
}

#[test]
fn a_divergent_allocation_is_its_own_named_class() {
    let repo = Repo::new();
    let id = repo.add_decision("Allocated differently by each writer.");
    repo.commit("main: the shared decision");
    diverge(
        &repo,
        |r| {
            r.ok(&["allocate", &id, "--store", "judgment", "--actor", "fixture-human@example"]);
        },
        |r| {
            r.ok(&["allocate", &id, "--store", "criterion", "--discharge", "test:SomeSuite",
                   "--stage", "pr"]);
        },
    );
    let plan = repo.ledger(&["merge", "b"]);
    assert_eq!(plan.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&plan.stdout).contains("CONFLICT [divergent-allocation]"),
        "{}",
        String::from_utf8_lossy(&plan.stdout)
    );
    repo.git(&["merge", "b", "--no-edit"]);
    let resolved = repo.ledger_stdin(&["merge", "--resolve"], "2\n");
    assert_eq!(resolved.status.code(), Some(0), "{}", String::from_utf8_lossy(&resolved.stderr));
    repo.commit("arbitrate the allocation");
    assert_eq!(repo.verify().status.code(), Some(0));
}

#[test]
fn a_competing_supersession_is_surfaced_and_settled_by_withdrawal() {
    let repo = Repo::new();
    let d = repo.add_decision("The decision both claim to replace.");
    let x = repo.add_decision("The first claimant.");
    let y = repo.add_decision("The second claimant.");
    repo.commit("main: three decisions");
    diverge(
        &repo,
        |r| {
            r.ok(&["supersede", &d, "--by", &x, "--reason", "narrowed"]);
        },
        |r| {
            r.ok(&["supersede", &d, "--by", &y, "--reason", "generalised"]);
        },
    );
    let plan = repo.ledger(&["merge", "b"]);
    assert_eq!(plan.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&plan.stdout).contains("CONFLICT [competing-supersession]"),
        "{}",
        String::from_utf8_lossy(&plan.stdout)
    );

    repo.git(&["merge", "b", "--no-edit"]);
    let broken = repo.verify();
    assert_eq!(broken.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&broken.stderr).contains("[G005]"));

    // Arbitrate: claimant 1 stands, the other claim is withdrawn on record.
    let resolved = repo.ledger_stdin(&["merge", "--resolve"], "1\n");
    assert_eq!(resolved.status.code(), Some(0), "{}", String::from_utf8_lossy(&resolved.stderr));
    repo.commit("arbitrate the supersession");
    let after = repo.verify();
    assert_eq!(after.status.code(), Some(0), "{}", String::from_utf8_lossy(&after.stderr));
    let status = repo.ok(&["status"]);
    assert!(status.contains("superseded"), "{status}");
    let log = repo.ok(&["log"]);
    assert!(log.contains("competing supersession"), "{log}");
}

#[test]
fn a_divergent_floor_stops_at_the_driver_and_resolves_by_choice() {
    let repo = Repo::new();
    // A member-free set, so a floor move cannot strand anyone.
    repo.ok(&["declare", "--set", "ops", "--tolerance-floor", "T0"]);
    repo.ok(&["merge", "--install"]);
    // The test must pin the driver to this build's binary, not PATH.
    let bin: PathBuf = assert_cmd::cargo::cargo_bin("ledger");
    repo.git(&[
        "config",
        "merge.ledger.driver",
        &format!("{} merge-driver %O %A %B %P", bin.display()),
    ]);
    repo.commit("main: the ops set, driver installed");

    let set_file = repo.path().join(".decisions/sets/ops.yml");
    let rewrite_floor = |floor: &str| {
        let text = std::fs::read_to_string(&set_file).expect("read");
        std::fs::write(&set_file, text.replace("tolerance_floor: T0", floor)).expect("write");
    };
    diverge(
        &repo,
        |_| rewrite_floor("tolerance_floor: T1"),
        |_| rewrite_floor("tolerance_floor: T2"),
    );

    let merge = repo.git_may_fail(&["merge", "b", "--no-edit"]);
    assert!(!merge.status.success(), "the driver must refuse the divergent floor");
    let noise = format!(
        "{}{}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert!(noise.contains("[divergent-floor]"), "{noise}");
    assert!(noise.contains("merge --resolve"), "{noise}");

    // The arbitration: theirs stands. Written, staged, on the record.
    let resolved = repo.ledger_stdin(&["merge", "--resolve"], "t\n");
    assert_eq!(
        resolved.status.code(),
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&resolved.stdout),
        String::from_utf8_lossy(&resolved.stderr)
    );
    let text = std::fs::read_to_string(&set_file).expect("read");
    assert!(text.contains("tolerance_floor: T2"), "{text}");
    repo.git(&["commit", "--no-edit"]);
    assert_eq!(repo.verify().status.code(), Some(0));
}

#[test]
fn an_acceptance_the_other_side_revised_past_is_surfaced_not_transferred() {
    let repo = Repo::new();
    let id = repo.add_decision("Accepted on one side, revised on the other.");
    repo.commit("main: the shared decision");
    diverge(
        &repo,
        |r| {
            r.ok(&["accept", &id]);
        },
        |r| {
            r.ok(&["revise", &id, "--statement", "Revised past the signature."]);
        },
    );
    let plan = repo.ok(&["merge", "b"]);
    assert!(plan.contains("overtaken"), "{plan}");
    assert!(plan.contains("awaits fresh acceptance"), "{plan}");
    assert!(plan.contains("no judgment required"), "nothing to arbitrate here: {plan}");

    repo.git(&["merge", "b", "--no-edit"]);
    let out = repo.verify();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    // The acceptance did not follow the content: the tip awaits a fresh one.
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("awaiting acceptance"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let blame = repo.ok(&["blame", &id]);
    assert!(blame.contains("historical"), "{blame}");
}
