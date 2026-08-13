//! `ledger show` end to end: the screen, the selectors, the reopen edge.
//!
//! The reopen edge is exercised here rather than only in the library because
//! the ruling it implements is about what a *reader* sees — that ground and
//! tripwire never share a block — and that is a property of the rendered
//! screen, not of the struct behind it.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

const TODAY: &str = "2026-08-13";

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Self { dir };
        repo.git(&["init", "-q"]);
        repo.git(&["config", "user.email", "fixture-human@example"]);
        repo.git(&["config", "user.name", "Fixture Human"]);
        repo.ok(&["init"]);
        repo.ok(&["declare", "--set", "shown", "--tolerance-floor", "T1"]);
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) {
        Command::new("git")
            .current_dir(self.path())
            .args(args)
            .output()
            .expect("git");
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::cargo_bin("ledger").expect("binary");
        cmd.current_dir(self.path()).arg("--root").arg(self.path()).args(args);
        cmd.output().expect("run")
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "`{}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Add a decision and return its id.
    fn add(&self, statement: &str, extra: &[&str]) -> String {
        let mut args = vec![
            "add", "--set", "shown", "--namespace", "fixture.shown",
            "--statement", statement, "--store", "constraint",
            "--discharge", "analyzer:DEC001",
        ];
        args.extend_from_slice(extra);
        let text = self.ok(&args);
        text.lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .expect("decision id in output")
            .to_string()
    }
}

#[test]
fn the_screen_carries_what_a_signature_needs() {
    let repo = Repo::new();
    let id = repo.add("Monetary amounts use decimal, never double.", &[
        "--based-on", "prd:decision-ledger-prd#4.2.1",
        "--note", "the filing act",
    ]);
    let text = repo.ok(&["show", &id, "--today", TODAY]);
    for needle in [
        "awaiting-acceptance",
        "shown (floor T1)",
        "Monetary amounts use decimal, never double.",
        "constraint",
        "prd:decision-ledger-prd#4.2.1",
        "the filing act",
        "none on record",
    ] {
        assert!(text.contains(needle), "`{needle}` missing from:\n{text}");
    }
    assert!(text.contains(&format!("ledger accept {id}")), "{text}");
}

/// The ruling, at the surface a reader actually meets: the reopen edge has
/// its own block and never appears inside the ground.
#[test]
fn ground_and_reopen_never_share_a_block() {
    let repo = Repo::new();
    let id = repo.add("internal is not contract surface by default.", &[
        "--based-on", "mandate:dec/ddd/internal-not-surface",
        "--revisit-if", "claim:DDD-adapter-02@sha256:b333063d",
    ]);
    let text = repo.ok(&["show", &id, "--today", TODAY]);
    let ground = text.find("based on").expect("ground block");
    let reopen = text.find("revisit if").expect("reopen block");
    assert!(ground < reopen, "{text}");
    assert!(
        !text[ground..reopen].contains("DDD-adapter-02"),
        "a reopen edge rendered as ground:\n{text}"
    );
    assert!(text[reopen..].contains("DDD-adapter-02"), "{text}");

    // And the same fact in the machine form, under its own key.
    let json = repo.ok(&["show", &id, "--json", "--today", TODAY]);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("json");
    assert_eq!(parsed[0]["revisit_if"][0]["pointer"], "claim:DDD-adapter-02@sha256:b333063d");
    assert_eq!(parsed[0]["based_on"][0]["pointer"], "mandate:dec/ddd/internal-not-surface");
}

/// A store carrying a reopen edge declares the format that needs it, and a
/// store without one stays where it was.
#[test]
fn a_reopen_edge_declares_format_four_and_nothing_else_does() {
    let repo = Repo::new();
    repo.add("A decision with no reopen edge.", &[]);
    repo.add("A decision that states one.", &[
        "--revisit-if", "claim:DDD-gates-01@sha256:6f500ee8",
    ]);
    let mut formats: Vec<u32> = std::fs::read_dir(repo.path().join(".decisions/log"))
        .expect("log dir")
        .filter_map(|e| e.ok())
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|t| {
            t.lines()
                .find_map(|l| l.strip_prefix("format: ").and_then(|n| n.trim().parse().ok()))
        })
        .collect();
    formats.sort_unstable();
    assert_eq!(formats, vec![1, 4], "one file needs format 4, the other does not");
    assert_eq!(repo.run(&["verify", "--today", TODAY]).status.code(), Some(0));
}

#[test]
fn a_selector_reads_a_group_in_one_pass() {
    let repo = Repo::new();
    // A criterion discharged solely by the contract check is a
    // transcription; the analyzer-discharged constraints are not.
    repo.add("An individual read.", &[]);
    repo.add("Another individual read.", &[]);
    let mechanical = repo.ok(&[
        "add", "--set", "shown", "--namespace", "fixture.shown",
        "--statement", "pub fn canonical_json in ledger-core/src/canon.rs",
        "--store", "criterion", "--discharge", "contract:seam/ledger/canonical-form",
        "--stage", "pr",
    ]);
    assert!(mechanical.contains("added"), "{mechanical}");

    let all = repo.ok(&["show", "--today", TODAY]);
    assert!(all.starts_with("3 decision(s)"), "{all}");
    let group = repo.ok(&["show", "--group", "mechanical", "--today", TODAY]);
    assert!(group.starts_with("1 decision(s)"), "{group}");
    assert!(group.contains("[mechanical]"), "{group}");
    let individual = repo.ok(&["show", "--group", "individual", "--today", TODAY]);
    assert!(individual.starts_with("2 decision(s)"), "{individual}");
    let by_set = repo.ok(&["show", "--set", "shown", "--today", TODAY]);
    assert!(by_set.starts_with("3 decision(s)"), "{by_set}");
    assert_eq!(repo.ok(&["show", "--set", "absent", "--today", TODAY]), "no decisions match\n");
}

#[test]
fn an_unknown_id_is_an_error_never_a_blank_screen() {
    let repo = Repo::new();
    let out = repo.run(&["show", "dec:fixture.shown/01K2C4YQJ3F8M0PT5W7NZ9RDXV"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("has no filed version"), "{err}");
}

#[test]
fn two_selectors_at_once_are_refused_rather_than_silently_widened() {
    let repo = Repo::new();
    let out = repo.run(&["show", "--set", "shown", "--group", "mechanical"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("not more than one"), "{err}");
}
