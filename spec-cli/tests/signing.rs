//! Signing through the `spec` binary.
//!
//! The scenarios that matter are the adoption edges: signing off by default,
//! on for the whole repo once a key is filed, and graced for acts nobody could
//! have signed.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;

struct Fixture {
    repo: tempfile::TempDir,
    keys: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let repo = tempfile::tempdir().expect("tempdir");
        for args in [
            vec!["init", "-q", "."],
            vec!["config", "user.email", "emil@example.com"],
            vec!["config", "user.name", "Emil"],
            vec!["commit", "-q", "--allow-empty", "-m", "init"],
        ] {
            Command::new("git").current_dir(repo.path()).args(&args).status().expect("git runs");
        }
        Self { repo, keys: tempfile::tempdir().expect("tempdir") }
    }

    fn root(&self) -> &Path {
        self.repo.path()
    }

    fn key_path(&self, id: &str) -> std::path::PathBuf {
        self.keys.path().join(format!("{id}.key"))
    }

    fn run(&self, args: &[&str]) -> (i32, String) {
        let mut cmd = Command::cargo_bin("spec").expect("the spec binary builds");
        cmd.arg("--root").arg(self.root()).args(args);
        let out = cmd.output().expect("spec runs");
        let mut text = String::from_utf8_lossy(&out.stdout).to_string();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        (out.status.code().unwrap_or(-1), text)
    }

    /// Open a record and return its id.
    fn open_record(&self, slice: &str) -> String {
        self.run(&["implement", "--slice", slice, "--act", "act/a", "--by", "agent@example.invalid"]);
        let (_, listed) = self.run(&["records", "--open", "--json"]);
        let parsed: serde_json::Value = serde_json::from_str(&listed).expect("json");
        parsed[0]["record"].as_str().expect("a record id").to_string()
    }

    fn adopt_signing(&self, id: &str) -> String {
        let path = self.key_path(id);
        let (code, text) = self.run(&[
            "trust", "generate", "--id", id,
            "--principal", "emil@example.com",
            "--out", path.to_str().expect("path"),
        ]);
        assert_eq!(code, 0, "{text}");
        path.to_str().expect("path").to_string()
    }
}

#[test]
fn signing_is_off_until_a_key_is_filed() {
    let f = Fixture::new();
    let (code, text) = f.run(&["trust", "list"]);
    assert_eq!(code, 0);
    assert!(text.contains("signing is off"), "{text}");
}

#[test]
fn a_secret_key_inside_the_repo_is_refused() {
    let f = Fixture::new();
    let inside = f.root().join("secret.key");
    let (code, text) = f.run(&[
        "trust", "generate", "--id", "emil", "--principal", "emil@example.com",
        "--out", inside.to_str().expect("path"),
    ]);
    assert_eq!(code, 2);
    assert!(text.contains("one commit from being public"), "{text}");
    assert!(!inside.exists(), "nothing must have been written");
}

#[test]
fn filing_a_key_turns_signing_on_for_the_repo() {
    let f = Fixture::new();
    f.adopt_signing("emil");
    let (_, text) = f.run(&["trust", "list"]);
    assert!(text.contains("signing is on"), "{text}");
}

#[test]
fn a_close_without_a_key_is_refused_once_signing_is_on() {
    let f = Fixture::new();
    f.adopt_signing("emil");
    let id = f.open_record("s");

    let (code, text) = f.run(&["close", &id, "--principal", "emil@example.com", "--nothing-arose"]);
    assert_eq!(code, 1);
    assert!(text.contains("S014"), "{text}");
}

#[test]
fn a_close_with_a_key_signs_and_clears_the_gate() {
    let f = Fixture::new();
    let key = f.adopt_signing("emil");
    let id = f.open_record("s");

    let (code, _) = f.run(&[
        "close", &id, "--principal", "emil@example.com", "--nothing-arose", "--key-file", &key,
    ]);
    assert_eq!(code, 0);
    assert_eq!(f.run(&["check", "--ci"]).0, 0);

    let stored = std::fs::read_to_string(f.root().join(".spec/records").join(format!("{id}.yml")))
        .expect("read record");
    assert!(stored.contains("signature: "), "{stored}");
}

#[test]
fn a_ratification_must_be_signed_too() {
    let f = Fixture::new();
    let key = f.adopt_signing("emil");

    let (unsigned, text) = f.run(&[
        "accept", "cand/x", "--name", "Settle", "--settles", "What is owed.",
        "--principal", "emil@example.com",
    ]);
    assert_eq!(unsigned, 1, "{text}");
    assert!(text.contains("S014"), "{text}");

    let (signed, text) = f.run(&[
        "accept", "cand/x", "--name", "Settle", "--settles", "What is owed.",
        "--principal", "emil@example.com", "--key-file", &key,
    ]);
    assert_eq!(signed, 0, "{text}");
}

#[test]
fn an_act_nobody_could_have_signed_is_graced() {
    let f = Fixture::new();
    let id = f.open_record("before-adoption");
    assert_eq!(
        f.run(&["close", &id, "--principal", "emil@example.com", "--nothing-arose"]).0,
        0
    );

    f.adopt_signing("emil");

    let (code, text) = f.run(&["check", "--ci"]);
    assert_eq!(code, 0, "adopting signing must not invalidate a history nobody could sign: {text}");
}

#[test]
fn a_forged_signature_on_a_graced_record_still_fails() {
    let f = Fixture::new();
    let id = f.open_record("before-adoption");
    f.run(&["close", &id, "--principal", "emil@example.com", "--nothing-arose"]);
    f.adopt_signing("emil");

    let path = f.root().join(".spec/records").join(format!("{id}.yml"));
    let stored = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, format!("{}\n  signature: {}\n", stored.trim_end(), "de".repeat(64)))
        .expect("forge");

    let (code, text) = f.run(&["check", "--ci"]);
    assert_eq!(code, 1, "the grace is for absence, never for a signature that does not verify");
    assert!(text.contains("S015"), "{text}");
}

#[test]
fn tampering_with_a_signed_record_breaks_the_signature() {
    let f = Fixture::new();
    let key = f.adopt_signing("emil");
    let id = f.open_record("checkout-totals");
    f.run(&["close", &id, "--principal", "emil@example.com", "--nothing-arose", "--key-file", &key]);

    let path = f.root().join(".spec/records").join(format!("{id}.yml"));
    let stored = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, stored.replace("checkout-totals", "something-else")).expect("tamper");

    let (code, text) = f.run(&["check", "--ci"]);
    assert_eq!(code, 1);
    assert!(text.contains("S015"), "the signature, not only the binding: {text}");
}

#[test]
fn the_environment_can_carry_the_key_instead_of_a_flag() {
    let f = Fixture::new();
    let key_path = f.adopt_signing("emil");
    let secret = std::fs::read_to_string(&key_path).expect("read key");
    let id = f.open_record("s");

    let out = Command::cargo_bin("spec")
        .expect("binary")
        .arg("--root")
        .arg(f.root())
        .args(["close", &id, "--principal", "emil@example.com", "--nothing-arose"])
        .env("SPEC_SIGNING_KEY", secret.trim())
        .output()
        .expect("spec runs");

    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn a_key_id_cannot_be_trusted_twice() {
    let f = Fixture::new();
    f.adopt_signing("emil");
    let second = f.keys.path().join("emil-again.key");
    let (code, text) = f.run(&[
        "trust", "generate", "--id", "emil", "--principal", "emil@example.com",
        "--out", second.to_str().expect("path"),
    ]);
    assert_eq!(code, 2);
    assert!(text.contains("rotate under a new id"), "{text}");
}
