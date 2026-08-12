//! `ddd diff-contracts` acceptance: the repo-diff half of the shared
//! classifier plus the discharge gate (M8), through the real binary.
//!
//! The load-bearing case is bypass-catch (spec invariant 5, success
//! criterion 3): an out-of-band contract-surface change — made in an
//! editor and committed with git, never touching MCP — produces a CI
//! finding, and a stored declaration signing that exact transition
//! discharges it.

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

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
    std::fs::create_dir_all(dir.join(".ddd/seams")).expect("mkdir");
}

fn write(dir: &Path, rel: &str, text: &str) {
    std::fs::write(dir.join(rel), text).expect("write");
}

fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", msg]);
}

const BEFORE: &str = ":root { --ink: #111; }\n";
const AFTER: &str = ":root { --ink: #111; --gap: 4px; }\n";

#[test]
fn an_out_of_band_contract_change_is_a_ci_finding() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "page.css", BEFORE);
    commit_all(dir, "before");
    // The out-of-band change: an editor edit committed with git — the
    // governed path never sees it.
    write(dir, "page.css", AFTER);
    commit_all(dir, "after");

    ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "UNDISCHARGED  contract/htmlcss/page.css#--gap@added",
        ))
        .stderr(predicate::str::contains("undischarged contract-surface change(s)"));
}

#[test]
fn a_signed_binding_discharges_exactly_its_transition() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "page.css", BEFORE);
    commit_all(dir, "before");
    let base = rev(dir, "HEAD");
    write(dir, "page.css", AFTER);
    commit_all(dir, "after");

    let binding = ddd_core::binding::SeamBinding::seal(
        "--gap",
        "page.css",
        &ddd_core::contracts::content_hash(BEFORE.as_bytes()),
        &ddd_core::contracts::content_hash(AFTER.as_bytes()),
        &base,
    );
    write_seam(dir, &binding);

    ddd(dir).args(["diff-contracts", "HEAD~1..HEAD"]).assert().success().stdout(
        predicate::str::contains("discharged  contract/htmlcss/page.css#--gap@added"),
    );
}

/// The reuse refusal (M8 ruling 1): a declaration signed for one
/// transition does not discharge a *different* edit that lands on
/// identical content — the before-hash differs, so it does not bind.
#[test]
fn a_binding_is_refused_against_a_different_change_landing_on_identical_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    // A different parent state (extra token) than the one the binding signs.
    write(dir, "page.css", ":root { --ink: #111; --old: red; }\n");
    commit_all(dir, "different-before");
    let base = rev(dir, "HEAD");
    write(dir, "page.css", AFTER);
    commit_all(dir, "after");

    // The binding signs the BEFORE -> AFTER transition, not this one.
    let binding = ddd_core::binding::SeamBinding::seal(
        "--gap",
        "page.css",
        &ddd_core::contracts::content_hash(BEFORE.as_bytes()),
        &ddd_core::contracts::content_hash(AFTER.as_bytes()),
        &base,
    );
    write_seam(dir, &binding);

    ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("no signed binding chain composes"));
}

/// Chain composition: two signed hops (A->B, B->C) discharge the
/// aggregate A->C transition a squashed diff shows.
#[test]
fn a_chain_of_bindings_composes_across_the_aggregate_diff() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    let middle = ":root { --ink: #111; --gap: 4px; }\n";
    let last = ":root { --ink: #111; --gap: 4px; --pad: 2px; }\n";
    write(dir, "page.css", BEFORE);
    commit_all(dir, "a");
    let base = rev(dir, "HEAD");
    write(dir, "page.css", middle);
    commit_all(dir, "b");
    let mid_rev = rev(dir, "HEAD");
    write(dir, "page.css", last);
    commit_all(dir, "c");

    let hop1 = ddd_core::binding::SeamBinding::seal(
        "--gap",
        "page.css",
        &ddd_core::contracts::content_hash(BEFORE.as_bytes()),
        &ddd_core::contracts::content_hash(middle.as_bytes()),
        &base,
    );
    let hop2 = ddd_core::binding::SeamBinding::seal(
        "--pad",
        "page.css",
        &ddd_core::contracts::content_hash(middle.as_bytes()),
        &ddd_core::contracts::content_hash(last.as_bytes()),
        &mid_rev,
    );
    write_seams(dir, &[("seam-css-gap", "seam/css/gap", hop1), ("seam-css-pad", "seam/css/pad", hop2)]);

    ddd(dir).args(["diff-contracts", "HEAD~2..HEAD"]).assert().success();
}

#[test]
fn json_report_carries_stable_finding_ids_and_hashes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "page.css", ":root { --ink: #111; }\n");
    commit_all(dir, "before");
    // A token changing type (color -> length) is a signature change;
    // a value-only change within one type is not contract surface.
    write(dir, "page.css", ":root { --ink: 4px; }\n");
    commit_all(dir, "after");

    let out = ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&out).expect("json");
    let files = report["report"]["files"].as_array().expect("files");
    assert_eq!(files.len(), 1);
    assert!(files[0]["before"].as_str().expect("before").starts_with("sha256:"));
    assert!(files[0]["after"].as_str().expect("after").starts_with("sha256:"));
    let ids: Vec<&str> = files[0]["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter_map(|e| e["id"].as_str())
        .collect();
    assert!(ids.contains(&"contract/htmlcss/page.css#--ink@signature-changed"), "{ids:?}");
    assert!(report["undischarged"].as_array().is_some_and(|u| !u.is_empty()));
}

#[test]
fn clean_range_reports_no_contract_surface() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    init_repo(dir);
    write(dir, "notes.md", "prose\n");
    commit_all(dir, "before");
    write(dir, "notes.md", "more prose\n");
    commit_all(dir, "after");

    ddd(dir)
        .args(["diff-contracts", "HEAD~1..HEAD"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no contract-surface changes"));
}

fn rev(dir: &Path, name: &str) -> String {
    let out = Proc::new("git").arg("-C").arg(dir).args(["rev-parse", name]).output().expect("git");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write_seam(dir: &Path, binding: &ddd_core::binding::SeamBinding) {
    write_seams(dir, &[("seam-css-gap", "seam/css/gap", binding.clone())]);
}

fn write_seams(dir: &Path, seams: &[(&str, &str, ddd_core::binding::SeamBinding)]) {
    for (slug, id, binding) in seams {
        let seam = ddd_core::seam::SeamDeclaration {
            format: 2,
            id: id.to_string(),
            boundary: format!("{} in page.css", binding.symbol),
            verdict_knowledge: "consumers learn the pair's token vocabulary".to_string(),
            contract_location: format!("page.css#{}", binding.symbol),
            obligations: Vec::new(),
            metadata: Default::default(),
            bindings: vec![binding.clone()],
            notes: None,
        };
        let yaml = serde_yaml::to_string(&seam).expect("yaml");
        std::fs::write(dir.join(".ddd/seams").join(format!("{slug}.yaml")), yaml).expect("seam");
    }
}
