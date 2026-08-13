//! `ledger accept --group` end to end: the batched act, the unbatched
//! signature.
//!
//! Every assertion here is about the same property from a different side —
//! that batching the invocation left the binding alone. N decisions produce
//! N acceptance records, each carrying its own version hash; the selection
//! is pinned before it is signed and refused if it moved; a held member
//! stops the run rather than being signed around; and nothing the batch
//! accepts would have been refused as a single accept.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn human() -> Self {
        Self::as_identity("fixture-human@example")
    }

    fn as_identity(email: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Self { dir };
        repo.git(&["init", "-q", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Fixture"]);
        repo.git(&["config", "user.email", email]);
        repo.ok(&["init"]);
        repo.ok(&["declare", "--set", "batched", "--tolerance-floor", "T1"]);
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

    /// A run the tool says no to (exit 1), returning stdout and stderr both:
    /// the enumeration prints on stdout even when the verdict is a refusal.
    fn refused(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "ledger {args:?} should refuse:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }

    /// A run the tool could not perform at all (exit 2).
    fn unusable(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        assert_eq!(out.status.code(), Some(2), "ledger {args:?} should be unusable");
        String::from_utf8_lossy(&out.stderr).into_owned()
    }

    /// Add a decision to the batched set; returns its id.
    fn add(&self, statement: &str, basis: &[&str]) -> String {
        let mut args: Vec<String> = vec![
            "add".into(), "--set".into(), "batched".into(),
            "--statement".into(), statement.into(),
            "--namespace".into(), "fixture.batched".into(),
            "--store".into(), "constraint".into(),
            "--discharge".into(), "analyzer:DEC001".into(),
        ];
        for b in basis {
            args.push("--based-on".into());
            args.push((*b).to_string());
        }
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        let text = self.ok(&borrowed);
        text.lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .expect("decision id in output")
            .to_string()
    }

    fn verify_green(&self) {
        let out = self.ledger(&["verify"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "verify should pass:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn log_files(&self) -> usize {
        std::fs::read_dir(self.path().join(".decisions/log"))
            .expect("log dir")
            .filter_map(Result::ok)
            .count()
    }

    fn acceptances(&self) -> Vec<(String, String)> {
        ledger_core::store::load(self.path())
            .log
            .iter()
            .flat_map(|c| c.file.acceptances.iter())
            .map(|a| (a.decision.to_string(), a.version.to_string()))
            .collect()
    }
}

/// The `manifest   sha256:…` line a dry run prints.
fn manifest_of(text: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix("manifest"))
        .map(|rest| rest.trim().to_string())
        .unwrap_or_else(|| panic!("no manifest line in:\n{text}"))
}

/// The whole point, stated as one test: batching the invocation changed the
/// act, not the binding.
#[test]
fn a_batch_files_one_acceptance_record_per_decision_each_on_its_own_hash() {
    let repo = Repo::human();
    let ids = [
        repo.add("Monetary amounts use decimal, never double.", &[]),
        repo.add("Identity comes from git config, never a flag.", &[]),
        repo.add("The log is append-only; a correction is a new version.", &[]),
    ];
    let before = repo.log_files();

    let read = repo.ok(&["accept", "--set", "batched"]);
    let manifest = manifest_of(&read);
    let signed = repo.ok(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert!(signed.contains("signed 3 decision(s)"), "{signed}");
    assert!(signed.contains("each signing its own version hash"), "{signed}");

    // One act: one new log file. Three signatures: three records.
    assert_eq!(repo.log_files(), before + 1, "the batched act is one change-set");
    let acceptances = repo.acceptances();
    assert_eq!(acceptances.len(), 3);
    for id in &ids {
        let record = acceptances.iter().find(|(d, _)| d == id).expect("one record per decision");
        let screen = repo.ok(&["show", id, "--json"]);
        assert!(screen.contains(&record.1), "the record signs this decision's own hash:\n{screen}");
    }
    let hashes: std::collections::BTreeSet<&String> = acceptances.iter().map(|(_, h)| h).collect();
    assert_eq!(hashes.len(), 3, "three distinct version hashes, not one shared record");

    assert!(repo.ok(&["status"]).matches("decided").count() >= 3);
    repo.verify_green();
}

#[test]
fn the_dry_run_is_the_default_posture_and_writes_nothing() {
    let repo = Repo::human();
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);
    let before = repo.log_files();

    let read = repo.ok(&["accept", "--set", "batched"]);
    assert!(read.contains("dry run — nothing written"), "{read}");
    assert!(read.contains(&id), "the enumeration names every member:\n{read}");
    assert!(read.contains("1 signable · 0 already signed · 0 held · 0 forked"), "{read}");
    assert!(read.contains("ledger accept --set batched --confirm sha256:"), "{read}");

    assert_eq!(repo.log_files(), before, "a dry run appends nothing");
    assert!(repo.acceptances().is_empty());
    assert!(repo.ok(&["status"]).contains("awaiting-acceptance"));
}

/// Signing takes the manifest a read produced, which is a value only a dry
/// run can hand you — so "confirm without a prior read" has no spelling.
/// A digest from nowhere is still enumerated afresh and still refused.
#[test]
fn a_confirmation_that_no_read_produced_is_refused() {
    let repo = Repo::human();
    repo.add("Monetary amounts use decimal, never double.", &[]);

    let mistyped = repo.unusable(&["accept", "--set", "batched", "--confirm", "yes"]);
    assert!(mistyped.contains("is not a manifest"), "{mistyped}");

    let stranger = format!("sha256:{}", "0".repeat(64));
    let refused = repo.refused(&["accept", "--set", "batched", "--confirm", &stranger]);
    assert!(refused.contains("the selection moved since it was read"), "{refused}");
    assert!(refused.contains("matches no state this log has passed through"), "{refused}");
    assert!(repo.acceptances().is_empty());
}

#[test]
fn an_entry_filed_between_the_read_and_the_run_refuses_the_run_by_name() {
    let repo = Repo::human();
    repo.add("Monetary amounts use decimal, never double.", &[]);
    let manifest = manifest_of(&repo.ok(&["accept", "--set", "batched"]));

    // The store moves under the pass: a new entry nobody read.
    let unread = repo.add("A late arrival, unread by the principal.", &[]);

    let refused = repo.refused(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert!(refused.contains("the selection moved since it was read"), "{refused}");
    assert!(refused.contains(&unread), "the refusal names what moved:\n{refused}");
    assert!(refused.contains("entered the selection, unread"), "{refused}");
    assert!(repo.acceptances().is_empty(), "nothing is signed on a drifted selection");
}

#[test]
fn a_revision_between_the_read_and_the_run_refuses_by_naming_both_hashes() {
    let repo = Repo::human();
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);
    let manifest = manifest_of(&repo.ok(&["accept", "--set", "batched"]));
    repo.ok(&["revise", &id, "--statement", "Decimal everywhere, including imports."]);

    let refused = repo.refused(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert!(refused.contains(&id), "{refused}");
    assert!(refused.contains("is no longer the tip"), "{refused}");
    assert!(repo.acceptances().is_empty());
}

/// A held entry does not drop quietly out of a batch. It stops it, by name,
/// so the judgment of whether to sign it stays a judgment somebody makes.
#[test]
fn a_held_member_stops_the_whole_batch_by_name() {
    let repo = Repo::human();
    repo.add("Monetary amounts use decimal, never double.", &[]);
    let held = repo.add(
        "The interceptor is not an extension point.",
        &["indeterminate:DDD-arch-03@sha256:8e7e53b6"],
    );

    // Even the read refuses: the selection cannot be signed as it stands.
    let read = repo.refused(&["accept", "--set", "batched"]);
    assert!(read.contains(&held), "{read}");
    assert!(read.contains("not for acceptance"), "{read}");
    assert!(read.contains("indeterminate"), "{read}");
    assert!(read.contains("1 signable · 0 already signed · 1 held · 0 forked"), "{read}");

    let manifest = manifest_of(&read);
    let refused = repo.refused(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert!(refused.contains(&held), "{refused}");
    assert!(
        repo.acceptances().is_empty(),
        "a held member stops the run rather than being signed around"
    );
}

/// The batch runs the same delta-gate a single accept runs. No class is
/// waived, and the refusal carries the same message the single verb gives.
#[test]
fn a_member_that_fails_the_gate_alone_fails_the_batch_with_the_same_class() {
    let repo = Repo::human();
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);
    let stale = "2020-01-01";

    let alone = repo.refused(&["accept", &id, "--expires", stale]);
    assert!(alone.contains("L003"), "the single verb's class:\n{alone}");

    let manifest = manifest_of(&repo.ok(&["accept", "--set", "batched", "--expires", stale]));
    let batched =
        repo.refused(&["accept", "--set", "batched", "--expires", stale, "--confirm", &manifest]);
    assert!(batched.contains("there is no batch-only leniency"), "{batched}");
    assert!(batched.contains("L003"), "the same class in the batch:\n{batched}");
    assert!(batched.contains(&id), "the failing member is named:\n{batched}");
    assert!(repo.acceptances().is_empty());
}

/// Identity is unchanged: the acceptor is whoever git config says, and a
/// model or CI identity is refused by `L006` in a batch exactly as alone.
#[test]
fn a_model_identity_is_refused_in_a_batch_exactly_as_it_is_alone() {
    let repo = Repo::as_identity("noreply@anthropic.com");
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);

    let alone = repo.refused(&["accept", &id]);
    assert!(alone.contains("L006"), "{alone}");

    let manifest = manifest_of(&repo.ok(&["accept", "--set", "batched"]));
    let batched = repo.refused(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert!(batched.contains("L006"), "{batched}");
    assert!(repo.acceptances().is_empty());
}

/// The invariant the whole design rests on: what was read is what is signed.
#[test]
fn show_and_accept_resolve_to_the_identical_set() {
    let repo = Repo::human();
    repo.add("Monetary amounts use decimal, never double.", &[]);
    repo.add("Identity comes from git config, never a flag.", &[]);
    repo.add("The interceptor is not an extension point.", &["indeterminate:DDD-arch-03"]);

    for selector in [["--group", "individual"], ["--set", "batched"]] {
        let shown: Vec<String> = json_field(&repo.ok(&["show", selector[0], selector[1], "--json"]));
        let out = repo.ledger(&["accept", selector[0], selector[1], "--json"]);
        let planned: Vec<String> = serde_json::from_slice::<serde_json::Value>(&out.stdout)
            .expect("json")
            .get("plan")
            .and_then(|p| p.get("members"))
            .and_then(|m| m.as_array())
            .expect("members")
            .iter()
            .filter_map(|m| m.get("decision").and_then(|d| d.as_str()).map(str::to_string))
            .collect();
        assert!(!shown.is_empty());
        assert_eq!(shown, planned, "`show {selector:?}` and `accept {selector:?}` must agree");
    }
}

/// Not a silent exclusion: an entry already signed is enumerated, labelled,
/// and left alone — visible in the read rather than absent from it.
#[test]
fn an_already_signed_member_is_enumerated_rather_than_dropped() {
    let repo = Repo::human();
    let first = repo.add("Monetary amounts use decimal, never double.", &[]);
    repo.add("Identity comes from git config, never a flag.", &[]);
    repo.ok(&["accept", &first]);

    let read = repo.ok(&["accept", "--set", "batched"]);
    assert!(read.contains(&first), "{read}");
    assert!(read.contains("already carries a live acceptance"), "{read}");
    assert!(read.contains("1 signable · 1 already signed · 0 held · 0 forked"), "{read}");

    let manifest = manifest_of(&read);
    repo.ok(&["accept", "--set", "batched", "--confirm", &manifest]);
    assert_eq!(repo.acceptances().len(), 2, "the signed member was not re-signed");
    repo.verify_green();
}

#[test]
fn a_selection_must_be_named_rather_than_implied() {
    let repo = Repo::human();
    repo.add("Monetary amounts use decimal, never double.", &[]);

    let implicit = repo.unusable(&["accept"]);
    assert!(implicit.contains("never covers the whole store implicitly"), "{implicit}");

    let two = repo.unusable(&["accept", "--set", "batched", "--group", "individual"]);
    assert!(two.contains("not more than one"), "{two}");

    let empty = repo.unusable(&["accept", "--set", "absent"]);
    assert!(empty.contains("matches no decision"), "{empty}");
}

fn json_field(text: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(text)
        .expect("json")
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|s| s.get("decision").and_then(|d| d.as_str()).map(str::to_string))
        .collect()
}
