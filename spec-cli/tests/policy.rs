//! The check policy through the `spec` binary.
//!
//! The scenario that matters is the degeneration path: quietly lower a
//! number, leave the argument that justified the old one. B-2 is what catches
//! it, and these tests are what prove B-2 is wired to the write.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for args in [
        vec!["init", "-q", "."],
        vec!["config", "user.email", "emil@example.com"],
        vec!["config", "user.name", "Emil"],
        vec!["commit", "-q", "--allow-empty", "-m", "init"],
    ] {
        Command::new("git").current_dir(dir.path()).args(&args).status().expect("git runs");
    }
    dir
}

fn spec(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("spec").expect("the spec binary builds");
    cmd.arg("--root").arg(root);
    cmd
}

fn run(root: &Path, args: &[&str]) -> (i32, String) {
    let out = spec(root).args(args).output().expect("spec runs");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

const ZERO: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn draft(root: &Path, fires_when: &str, basis: &str, binds: &str) -> std::path::PathBuf {
    let path = root.join("policy-draft.yml");
    std::fs::write(
        &path,
        format!(
            "policy_verdicts:\n  \
             - metric: records_open\n    \
               fires_when: \"{fires_when}\"\n    \
               basis: \"{basis}\"\n    \
               basis_binds: \"{binds}\"\n    \
               principal: {{ kind: team, identifier: platform }}\n\
             not_gated:\n  \
             - metric: mapping_coverage\n    \
               reason: Coverage is a metric, not a goal.\n"
        ),
    )
    .expect("write draft");
    path
}

/// File a policy, resolving the basis digest the way a hand-author would:
/// write zeroes, read the digest back off the finding, paste it in.
fn file_policy(root: &Path, fires_when: &str, basis: &str) -> (i32, String) {
    let path = draft(root, fires_when, basis, ZERO);
    let (_, refused) = run(root, &["policy", "set", path.to_str().expect("path"), "--principal", "emil@example.com"]);
    let digest = refused
        .split("basis_binds: ")
        .nth(1)
        .and_then(|rest| rest.split('`').next())
        .unwrap_or_default()
        .trim()
        .to_string();
    let path = draft(root, fires_when, basis, &digest);
    run(root, &["policy", "set", path.to_str().expect("path"), "--principal", "emil@example.com"])
}

#[test]
fn the_default_is_structural_only() {
    let dir = repo();
    let (code, text) = run(dir.path(), &["policy", "show"]);
    assert_eq!(code, 0);
    assert!(text.contains("no policy filed"), "{text}");
    assert!(text.contains("would presume a basis nobody stated"), "{text}");
}

#[test]
fn a_basis_that_does_not_bind_its_threshold_is_refused() {
    let dir = repo();
    let path = draft(dir.path(), "count > 0", "Because we said so.", ZERO);
    let (code, text) = run(dir.path(), &[
        "policy", "set", path.to_str().expect("path"), "--principal", "emil@example.com",
    ]);
    assert_eq!(code, 1);
    assert!(text.contains("S009"), "{text}");
    assert!(text.contains("set `basis_binds: sha256:"), "the digest must be reported: {text}");
}

#[test]
fn a_bound_basis_files() {
    let dir = repo();
    let (code, text) = file_policy(dir.path(), "count > 0", "An open record is an unmade write.");
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("filed policy"), "{text}");
}

#[test]
fn moving_a_threshold_without_the_argument_fails() {
    let dir = repo();
    file_policy(dir.path(), "count > 0", "An open record is an unmade write.");

    // The number moves; the basis and its digest do not.
    let path = draft(dir.path(), "count > 99", "An open record is an unmade write.", ZERO);
    let (code, text) = run(dir.path(), &[
        "policy", "set", path.to_str().expect("path"), "--principal", "emil@example.com",
    ]);
    assert_eq!(code, 1);
    assert!(text.contains("the threshold moved, the argument did not"), "{text}");
}

#[test]
fn a_policy_with_no_uncovered_set_is_refused() {
    let dir = repo();
    let path = dir.path().join("bare.yml");
    std::fs::write(&path, "policy_verdicts: []\nnot_gated: []\n").expect("write");
    let (code, text) = run(dir.path(), &[
        "policy", "set", path.to_str().expect("path"), "--principal", "emil@example.com",
    ]);
    assert_eq!(code, 1);
    assert!(text.contains("S011"), "{text}");
}

#[test]
fn a_policy_verdict_fires_and_is_reported_apart_from_the_structural_ones() {
    let dir = repo();
    file_policy(dir.path(), "count > 0", "An open record is an unmade write.");
    run(dir.path(), &["implement", "--slice", "s", "--act", "act/a", "--by", "agent@example.invalid"]);

    let (code, text) = run(dir.path(), &["check", "--ci"]);
    assert_eq!(code, 1);
    assert!(text.contains("S001"), "the structural verdict: {text}");
    assert!(text.contains("policy: records_open"), "the project's own verdict: {text}");
}

#[test]
fn filing_a_second_version_supersedes_the_first() {
    let dir = repo();
    file_policy(dir.path(), "count > 0", "The first argument.");
    let (code, text) = file_policy(dir.path(), "count > 5", "A second, different argument.");
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("supersedes"), "{text}");

    let (_, shown) = run(dir.path(), &["policy", "show"]);
    assert!(shown.contains("count > 5"), "the tip is in force: {shown}");
    assert!(!shown.contains("The first argument"), "{shown}");
}

#[test]
fn the_superseded_version_stays_readable_on_disk() {
    let dir = repo();
    file_policy(dir.path(), "count > 0", "The first argument.");
    file_policy(dir.path(), "count > 5", "A second, different argument.");

    let filed: Vec<_> = std::fs::read_dir(dir.path().join(".spec/policy"))
        .expect("policy dir")
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(filed.len(), 2, "a policy loosened twice is evidence about the original claim");
}

#[test]
fn metrics_say_which_of_them_can_fail_a_build() {
    let dir = repo();
    file_policy(dir.path(), "count > 0", "An open record is an unmade write.");
    let (_, text) = run(dir.path(), &["check"]);
    assert!(text.contains("only those marked `gated` can fail a build"), "{text}");
    assert!(
        text.lines().any(|l| l.contains("records_open") && l.contains("gated")),
        "{text}"
    );
}
