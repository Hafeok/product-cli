//! Tests for delivery-feature (deliverable) validation.

use super::*;
use std::collections::BTreeSet;

fn slices() -> BTreeSet<String> {
    ["place-order".to_string()].into_iter().collect()
}

#[test]
fn yaml_round_trips() {
    let d = Deliverable {
        id: "place-order".into(),
        slice: "place-order".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "an order can be placed".into(), status: "pending".into(), runner: None, runner_args: None }],
    };
    assert_eq!(Deliverable::from_yaml(&d.to_yaml().expect("to")).expect("from"), d);
}

#[test]
fn a_resolving_slice_passes() {
    let d = Deliverable { id: "po".into(), slice: "place-order".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &slices()).is_empty());
}

#[test]
fn a_missing_slice_is_a_violation() {
    let d = Deliverable { id: "po".into(), slice: "ghost".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &slices()).iter().any(|v| v.path == "slice" && v.message.contains("ghost")));
}

#[test]
fn an_empty_slice_is_a_violation() {
    let d = Deliverable { id: "po".into(), slice: "".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &slices()).iter().any(|v| v.path == "slice"));
}

#[test]
fn an_empty_acceptance_statement_is_a_violation() {
    let d = Deliverable {
        id: "po".into(),
        slice: "place-order".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "".into(), status: "pending".into(), runner: None, runner_args: None }],
    };
    assert!(validate_deliverable(&d, &slices()).iter().any(|v| v.path == "acceptance"));
}
