//! Reviewing an imported codebase through the `spec` binary.
//!
//! The scenarios that matter are the refusals: a machine ratifying, a machine
//! refusing, and an endpoint nobody has looked at reaching the gate.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;

const SETTLE: &str = "cand/shop-api-basketcontroller-settle-httppost";
const HEALTH: &str = "cand/shop-api-basketcontroller-health-httpget";

const INVENTORY: &str = r#"{
  "form": "spec.inventory.v1",
  "root": ".",
  "revision": "abc123",
  "scanned_at": "2026-09-15T00:00:00+00:00",
  "symbols": [],
  "composition_edges": [],
  "entry_points": [
    {"id":"Shop.Api.BasketController.Settle#HttpPost","kind":"http-route",
     "symbol":"Shop.Api.BasketController.Settle","transport":"HttpPost /baskets/{id}/settle",
     "file":"src/Api.cs","line":4},
    {"id":"Shop.Api.BasketController.Health#HttpGet","kind":"http-route",
     "symbol":"Shop.Api.BasketController.Health","transport":"HttpGet /health",
     "file":"src/Api.cs","line":6}
  ],
  "candidates": [
    {"id":"cand/shop-api-basketcontroller-settle-httppost",
     "entry_point":"Shop.Api.BasketController.Settle#HttpPost",
     "observed":{"transport":"HttpPost /baskets/{id}/settle"},
     "unfilled_slots":["name","settles"]},
    {"id":"cand/shop-api-basketcontroller-health-httpget",
     "entry_point":"Shop.Api.BasketController.Health#HttpGet",
     "observed":{"transport":"HttpGet /health"},
     "unfilled_slots":["name","settles"]}
  ]
}"#;

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
    let path = dir.path().join(".spec/inventory.json");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, INVENTORY).expect("write inventory");
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

fn ratify_settle(root: &Path, principal: &str) -> (i32, String) {
    run(root, &[
        "accept", SETTLE,
        "--name", "Settle a basket",
        "--settles", "What the customer owes when the basket closes.",
        "--principal", principal,
    ])
}

#[test]
fn an_unreviewed_endpoint_fails_the_gate() {
    let dir = repo();
    let (code, text) = run(dir.path(), &["check", "--ci"]);
    assert_eq!(code, 1);
    assert!(text.contains("S005"), "{text}");
}

#[test]
fn a_machine_cannot_ratify_a_candidate() {
    let dir = repo();
    let (code, text) = ratify_settle(dir.path(), "claude@example.com");
    assert_eq!(code, 1, "a refused write reports a finding, not a crash");
    assert!(text.contains("S002"), "{text}");

    let (_, listed) = run(dir.path(), &["candidates", "--unreviewed"]);
    assert!(listed.contains(SETTLE), "nothing must have landed");
}

#[test]
fn a_machine_cannot_refuse_a_candidate_either() {
    let dir = repo();
    let (code, text) = run(dir.path(), &[
        "reject", HEALTH, "--reason", "Infrastructure.", "--principal", "ci@example.com",
    ]);
    assert_eq!(code, 1);
    assert!(text.contains("S002"), "{text}");
}

#[test]
fn a_principal_ratifies_and_the_drift_clears() {
    let dir = repo();
    assert_eq!(ratify_settle(dir.path(), "emil@example.com").0, 0);

    let (code, text) = run(dir.path(), &["check", "--ci"]);
    assert_eq!(code, 1, "the health probe is still unreviewed");
    assert!(!text.contains("Settle"), "the ratified act is covered: {text}");
}

#[test]
fn a_reasoned_refusal_covers_an_entry_point_too() {
    let dir = repo();
    ratify_settle(dir.path(), "emil@example.com");
    let (code, _) = run(dir.path(), &[
        "reject", HEALTH,
        "--reason", "A liveness probe settles nothing; it is infrastructure.",
        "--principal", "emil@example.com",
    ]);
    assert_eq!(code, 0);

    let (gate, text) = run(dir.path(), &["check", "--ci"]);
    assert_eq!(gate, 0, "a principal looked and decided — that is coverage: {text}");
}

#[test]
fn ratifying_an_unknown_candidate_is_refused_as_a_typo() {
    let dir = repo();
    let (code, text) = run(dir.path(), &[
        "accept", "cand/typo", "--name", "X", "--settles", "Y", "--principal", "emil@example.com",
    ]);
    assert_eq!(code, 2);
    assert!(text.contains("no candidate"), "{text}");
}

#[test]
fn a_record_naming_an_unratified_act_is_a_dangling_reference() {
    let dir = repo();
    ratify_settle(dir.path(), "emil@example.com");
    run(dir.path(), &[
        "implement", "--slice", "s", "--act", "act/does-not-exist", "--by", "agent@example.invalid",
    ]);

    let (code, text) = run(dir.path(), &["check", "--ci"]);
    assert_eq!(code, 1);
    assert!(text.contains("S006"), "{text}");
}

#[test]
fn map_reports_a_split_when_two_acts_claim_one_entry_point() {
    let dir = repo();
    ratify_settle(dir.path(), "emil@example.com");
    run(dir.path(), &[
        "accept", SETTLE, "--id", "act/second-claim",
        "--name", "Something else", "--settles", "A second claim on one entry point.",
        "--principal", "emil@example.com",
    ]);

    let (_, text) = run(dir.path(), &["map"]);
    assert!(text.contains("split"), "{text}");
}

#[test]
fn the_assessment_surface_adds_the_join_beneath_the_verdicts() {
    let dir = repo();
    let (_, plain) = run(dir.path(), &["check"]);
    let (_, assessment) = run(dir.path(), &["check", "--assessment"]);

    assert!(plain.contains("metrics"), "{plain}");
    assert!(!plain.contains("act ↔ entry point"), "{plain}");
    assert!(assessment.contains("act ↔ entry point"), "{assessment}");
}

#[test]
fn coverage_is_reported_and_never_gated() {
    let dir = repo();
    let (code, text) = run(dir.path(), &["check", "--json"]);
    assert_eq!(code, 1, "the findings are what fail, not the coverage");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(parsed["metrics"]["mapping_coverage"], "0%");
}
