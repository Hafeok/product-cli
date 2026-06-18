//! Tests for the repository layout-model checker.

use super::*;

const EXAMPLE: &str = include_str!("../../../schema/examples/layout-model.example.yaml");

fn example() -> LayoutModel {
    LayoutModel::from_yaml(EXAMPLE).expect("parse")
}

#[test]
fn bundled_example_is_conformant() {
    let m = example();
    assert_eq!(m.layout.len(), 6);
    assert_eq!(validate_layout(&m), vec![]);
}

#[test]
fn round_trips() {
    let m = example();
    let back = LayoutModel::from_yaml(&m.to_yaml().expect("yaml")).expect("reparse");
    assert_eq!(m, back);
}

#[test]
fn rule_without_enforces_is_rejected() {
    let mut m = example();
    m.layout[0].enforces.clear();
    assert!(validate_layout(&m).iter().any(|x| x.path == "enforces"));
}

#[test]
fn must_exist_without_cardinality_is_rejected() {
    let mut m = example();
    m.layout[0].cardinality = None; // apphost-required is a must_exist
    assert!(validate_layout(&m).iter().any(|x| x.path == "cardinality"));
}

#[test]
fn rule_with_two_kinds_is_rejected() {
    let mut m = example();
    m.layout[0].no_orphans = Some("src/**".into()); // now has must_exist AND no_orphans
    assert!(validate_layout(&m).iter().any(|x| x.path == "rule-kind"));
}

#[test]
fn prohibition_without_rationale_is_rejected() {
    let mut m = example();
    // index 4 is no-secrets-in-source (must_not_exist)
    m.layout[4].rationale = None;
    assert!(validate_layout(&m).iter().any(|x| x.path == "rationale"));
}

#[test]
fn scaffold_is_conformant() {
    let m = LayoutModel::scaffold("rest-api");
    assert_eq!(validate_layout(&m), vec![]);
}
