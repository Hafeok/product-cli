//! Tests for build-verify planning (acceptance runners → concrete commands).

use super::*;
use crate::pf::deliverable::{AcceptanceCriterion, Deliverable};

fn crit(id: &str, runner: Option<&str>, args: Option<&str>) -> AcceptanceCriterion {
    AcceptanceCriterion {
        id: id.to_string(),
        statement: "x".to_string(),
        status: "pending".to_string(),
        runner: runner.map(String::from),
        runner_args: args.map(String::from),
    }
}

fn deliv(acceptance: Vec<AcceptanceCriterion>) -> Deliverable {
    Deliverable { id: "d1".to_string(), feature: "s1".to_string(), acceptance }
}

#[test]
fn cargo_test_runner_splits_args() {
    let d = deliv(vec![crit("a1", Some("cargo-test"), Some("pf::casing other"))]);
    let steps = plan(&d);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].criterion, "a1");
    assert_eq!(steps[0].program, "cargo");
    assert_eq!(steps[0].args, vec!["test", "pf::casing", "other"]);
}

#[test]
fn shell_runner_passes_one_command_string() {
    let d = deliv(vec![crit("a1", Some("shell"), Some("cargo build && cargo t"))]);
    let steps = plan(&d);
    assert_eq!(steps[0].program, "sh");
    assert_eq!(steps[0].args, vec!["-c", "cargo build && cargo t"]);
}

#[test]
fn criteria_without_a_runner_are_skipped() {
    let d = deliv(vec![crit("a1", None, None), crit("a2", Some("shell"), Some("true"))]);
    let steps = plan(&d);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].criterion, "a2");
}

#[test]
fn unknown_runner_is_reported_not_planned() {
    let d = deliv(vec![crit("a1", Some("make"), Some("all"))]);
    assert!(plan(&d).is_empty());
    assert_eq!(unknown_runners(&d), vec!["a1"]);
}
