//! The gate's failure classes, one passing store against nine violating ones.
//!
//! Eight classes have a checked-in fixture store under `tests/fixtures/`,
//! readable as documentation of the format. `L009` cannot: blame consistency
//! needs commits, and a checked-in fixture's history belongs to whoever
//! committed the fixture, not to the identity it names — so it is built in a
//! temporary repository instead.
//!
//! Fixture hashes are real. When the canonical form changes — which is a
//! `CANONICAL_FORM` bump and a migration note, never a quiet fix — refresh
//! them with:
//!
//! ```text
//! UPDATE_FIXTURES=1 cargo test -p ledger-cli --test gate
//! ```

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;

/// The date the fixtures are judged against, so expiry never depends on
/// when the suite happens to run.
const TODAY: &str = "2026-08-10";

/// The one fixture whose stored hash is deliberately stale.
const STALE_BY_DESIGN: &str = "l007";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Run the gate over a fixture store. Blame is off: these stores live inside
/// this repository, so their history attributes to whoever committed them.
fn gate(fixture: &str, extra: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root")
        .arg(fixtures().join(fixture))
        .args(["verify", "--today", TODAY, "--no-blame"])
        .args(extra);
    cmd.output().expect("run")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Assert the fixture fails, for this class and no other.
fn fails_only_with(fixture: &str, code: &str) {
    let out = gate(fixture, &[]);
    let text = stderr(&out);
    assert_eq!(out.status.code(), Some(1), "{fixture} should fail the gate:\n{text}");
    assert!(text.contains(&format!("[{code}]")), "{fixture} should report {code}:\n{text}");
    for other in
        ["L001", "L002", "L003", "L004", "L005", "L006", "L007", "L008", "L009", "L010", "SCHEMA"]
    {
        if other != code {
            assert!(
                !text.contains(&format!("[{other}]")),
                "{fixture} reported {other} as well — the fixture is not isolating {code}:\n{text}"
            );
        }
    }
}

#[test]
fn the_conformant_fixture_passes() {
    let out = gate("pass", &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("conformant"), "{}", stdout(&out));
}

#[test]
fn l001_a_decision_with_no_allocation() {
    fails_only_with("l001", "L001");
}

#[test]
fn l002_an_escape_without_exposure_acceptor_and_review_date() {
    fails_only_with("l002", "L002");
}

#[test]
fn l003_an_expired_acceptance() {
    fails_only_with("l003", "L003");
}

#[test]
fn l004_a_tolerance_override_below_the_set_floor() {
    fails_only_with("l004", "L004");
}

#[test]
fn l005_a_member_stranded_below_a_raised_floor() {
    fails_only_with("l005", "L005");
}

#[test]
fn l006_an_acceptor_resolving_to_a_model_identity() {
    fails_only_with("l006", "L006");
}

#[test]
fn l007_a_stored_hash_that_does_not_match_recomputed_content() {
    fails_only_with("l007", "L007");
}

#[test]
fn l008_an_acceptance_signing_a_hash_no_version_carries() {
    fails_only_with("l008", "L008");
}

#[test]
fn l010_a_judgment_exercised_by_a_model_identity() {
    fails_only_with("l010", "L010");
}

#[test]
fn pendency_is_visible_status_rather_than_a_failure() {
    // The l007 store has no acceptance at all. It fails for its own reason,
    // but never because nobody has signed yet.
    let text = stderr(&gate(STALE_BY_DESIGN, &[]));
    assert!(text.contains("awaiting acceptance"), "{text}");
}

#[test]
fn the_readiness_gate_lets_a_stale_acceptance_through() {
    // Readiness blocks produce; a stale acceptance blocks release.
    let out = gate("l003", &["--gate", "readiness"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let completeness = gate("l003", &["--gate", "completeness"]);
    assert_eq!(completeness.status.code(), Some(1));
}

#[test]
fn the_json_report_carries_the_classes_machine_readably() {
    let out = gate("l001", &["--json"]);
    let text = stdout(&out);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(parsed["findings"][0]["class"], "L001");
    assert_eq!(parsed["decisions"], 1);
}

// ---------------------------------------------------------------------------
// L009 — blame consistency, over real commits.
// ---------------------------------------------------------------------------

mod blame {
    use super::*;

    struct Repo {
        dir: tempfile::TempDir,
    }

    impl Repo {
        /// A repository with a conformant store, committed by `author`.
        fn with_acceptance_committed_by(author: &str) -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = Self { dir };
            repo.git(&["init", "--initial-branch=main"]);
            repo.git(&["config", "user.name", "Fixture Human"]);
            repo.git(&["config", "user.email", "fixture-human@example"]);
            repo.copy_pass_fixture();
            repo.git(&["add", "."]);
            repo.git(&[
                "-c",
                &format!("user.email={author}"),
                "commit",
                "-m",
                "file the acceptance",
                "--no-gpg-sign",
            ]);
            repo
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }

        fn copy_pass_fixture(&self) {
            let from = fixtures().join("pass/.decisions");
            let to = self.path().join(".decisions");
            std::fs::create_dir_all(to.join("sets")).expect("mkdir");
            std::fs::create_dir_all(to.join("log")).expect("mkdir");
            for sub in ["sets", "log"] {
                for entry in std::fs::read_dir(from.join(sub)).expect("read").flatten() {
                    let target = to.join(sub).join(entry.file_name());
                    std::fs::copy(entry.path(), target).expect("copy");
                }
            }
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

        fn verify(&self) -> Output {
            let mut cmd = Command::cargo_bin("ledger").expect("binary");
            cmd.arg("--root").arg(self.path()).args(["verify", "--today", TODAY]);
            cmd.output().expect("run")
        }
    }

    #[test]
    fn an_acceptance_committed_by_the_actor_it_names_passes() {
        let repo = Repo::with_acceptance_committed_by("fixture-human@example");
        let out = repo.verify();
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    }

    #[test]
    fn l009_an_acceptance_committed_by_somebody_else() {
        let repo = Repo::with_acceptance_committed_by("someone-else@example");
        let out = repo.verify();
        let text = stderr(&out);
        assert_eq!(out.status.code(), Some(1), "{text}");
        assert!(text.contains("[L009]"), "{text}");
        assert!(text.contains("someone-else@example"), "{text}");
    }

    #[test]
    fn an_uncommitted_acceptance_is_skipped_with_the_skip_reported() {
        let repo = Repo::with_acceptance_committed_by("fixture-human@example");
        // Re-file the acceptance under a new id, uncommitted.
        let log = repo.path().join(".decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml");
        let text = std::fs::read_to_string(&log).expect("read");
        std::fs::write(
            &log,
            text.replace("acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX", "acc:01K2C4YQJ3F8M0PT5W7NZ9RDXY"),
        )
        .expect("write");
        let out = repo.verify();
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert!(
            stdout(&out).contains("skipped for 1 uncommitted acceptance"),
            "a skipped check must never read as a passing one: {}",
            stdout(&out)
        );
    }
}

// ---------------------------------------------------------------------------
// Fixture maintenance.
// ---------------------------------------------------------------------------

/// Recompute every fixture's stored hashes. Runs only under
/// `UPDATE_FIXTURES=1`; otherwise it asserts the fixtures are current, which
/// is what makes them extra conformance vectors rather than stale copies.
#[test]
fn fixture_hashes_are_current() {
    let update = std::env::var("UPDATE_FIXTURES").is_ok();
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(fixtures()).expect("read fixtures").flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == STALE_BY_DESIGN {
            continue;
        }
        for log in std::fs::read_dir(entry.path().join(".decisions/log")).expect("log").flatten() {
            if refresh(&log.path(), update) {
                stale.push(format!("{name}/{}", log.file_name().to_string_lossy()));
            }
        }
    }
    assert!(
        stale.is_empty() || update,
        "stale fixture hashes in {stale:?} — run `UPDATE_FIXTURES=1 cargo test -p ledger-cli --test gate`"
    );
}

/// Rewrite the stored hashes in one log file. Returns whether it was stale.
fn refresh(path: &Path, write: bool) -> bool {
    let original = std::fs::read_to_string(path).expect("read");
    let parsed: ledger_core::changeset::ChangeSet =
        serde_yaml::from_str(&original).expect("parse fixture");
    let mut text = original.clone();
    for version in &parsed.versions {
        let computed = ledger_core::hash::version_hash(version).to_string();
        text = text.replace(&version.hash.to_string(), &computed);
    }
    if text != original && write {
        std::fs::write(path, &text).expect("write");
    }
    text != original
}
