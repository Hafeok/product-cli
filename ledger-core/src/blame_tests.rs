//! Blame cases against real git repositories built in a temp directory.

use std::path::PathBuf;
use std::process::Command;

use super::*;

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        run(dir.path(), &["init", "--initial-branch=main"]);
        run(dir.path(), &["config", "user.name", "Fixture Human"]);
        run(dir.path(), &["config", "user.email", "fixture-human@example"]);
        Self { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn commit(&self, name: &str, body: &str, email: &str) -> PathBuf {
        let file = self.path().join(name);
        std::fs::write(&file, body).expect("write");
        run(self.path(), &["add", "."]);
        run(
            self.path(),
            &["-c", &format!("user.email={email}"), "commit", "-m", "entry", "--no-gpg-sign"],
        );
        file
    }
}

fn run(dir: &Path, args: &[&str]) {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn the_author_of_the_introducing_commit_is_returned() {
    let repo = Repo::new();
    let file = repo.commit("log.yml", "id: acc:AAA\n", "fixture-human@example");
    assert_eq!(
        introducing_author(repo.path(), &file, "acc:AAA"),
        Attribution::Author("fixture-human@example".into())
    );
}

#[test]
fn the_earliest_commit_wins_when_a_later_one_touches_the_same_file() {
    let repo = Repo::new();
    repo.commit("log.yml", "id: acc:AAA\n", "first@example");
    let file = repo.commit("log.yml", "id: acc:AAA\nid: acc:BBB\n", "second@example");
    // Each acceptance is attributed to the commit that introduced *it*.
    assert_eq!(
        introducing_author(repo.path(), &file, "acc:AAA"),
        Attribution::Author("first@example".into())
    );
    assert_eq!(
        introducing_author(repo.path(), &file, "acc:BBB"),
        Attribution::Author("second@example".into())
    );
}

#[test]
fn an_uncommitted_acceptance_is_skipped_not_failed() {
    let repo = Repo::new();
    let file = repo.commit("log.yml", "id: acc:AAA\n", "fixture-human@example");
    std::fs::write(&file, "id: acc:AAA\nid: acc:NEW\n").expect("write");
    // Staged-but-uncommitted is the state every acceptance passes through.
    assert_eq!(introducing_author(repo.path(), &file, "acc:NEW"), Attribution::NotCommitted);
}

#[test]
fn a_directory_that_is_not_a_repository_reports_no_repository() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("log.yml");
    std::fs::write(&file, "id: acc:AAA\n").expect("write");
    assert_eq!(introducing_author(dir.path(), &file, "acc:AAA"), Attribution::NoRepository);
}

#[test]
fn author_emails_are_lowercased_so_comparison_matches_a_normalised_identity() {
    let repo = Repo::new();
    let file = repo.commit("log.yml", "id: acc:AAA\n", "Fixture-Human@Example");
    assert_eq!(
        introducing_author(repo.path(), &file, "acc:AAA"),
        Attribution::Author("fixture-human@example".into())
    );
}
