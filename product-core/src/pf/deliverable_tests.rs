//! Tests for deliverable validation.

use super::*;
use std::collections::BTreeSet;

fn features() -> BTreeSet<String> {
    ["place-order".to_string()].into_iter().collect()
}

#[test]
fn yaml_round_trips() {
    let d = Deliverable {
        id: "place-order".into(),
        feature: "place-order".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "an order can be placed".into(), status: "pending".into(), runner: None, runner_args: None }],
    };
    assert_eq!(Deliverable::from_yaml(&d.to_yaml().expect("to")).expect("from"), d);
}

#[test]
fn a_resolving_feature_passes() {
    let d = Deliverable { id: "po".into(), feature: "place-order".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &features()).is_empty());
}

#[test]
fn a_missing_feature_is_a_violation() {
    let d = Deliverable { id: "po".into(), feature: "ghost".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &features()).iter().any(|v| v.path == "feature" && v.message.contains("ghost")));
}

#[test]
fn an_empty_feature_is_a_violation() {
    let d = Deliverable { id: "po".into(), feature: "".into(), acceptance: vec![] };
    assert!(validate_deliverable(&d, &features()).iter().any(|v| v.path == "feature"));
}

#[test]
fn an_empty_acceptance_statement_is_a_violation() {
    let d = Deliverable {
        id: "po".into(),
        feature: "place-order".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "".into(), status: "pending".into(), runner: None, runner_args: None }],
    };
    assert!(validate_deliverable(&d, &features()).iter().any(|v| v.path == "acceptance"));
}
