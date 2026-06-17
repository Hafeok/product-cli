//! Tests for release membership validation.

use super::*;
use std::collections::BTreeSet;

fn deliverables() -> BTreeSet<String> {
    ["place-order".to_string(), "cancel-order".to_string()].into_iter().collect()
}

#[test]
fn yaml_round_trips() {
    let r = Release { id: "R1".into(), features: vec!["place-order".into()] };
    assert_eq!(Release::from_yaml(&r.to_yaml().expect("to")).expect("from"), r);
}

#[test]
fn resolving_members_pass() {
    let r = Release { id: "R1".into(), features: vec!["place-order".into(), "cancel-order".into()] };
    assert!(validate_release(&r, &deliverables()).is_empty());
}

#[test]
fn an_empty_release_is_a_violation() {
    let r = Release { id: "R1".into(), features: vec![] };
    assert!(validate_release(&r, &deliverables()).iter().any(|v| v.path == "features"));
}

#[test]
fn a_dangling_member_is_a_violation() {
    let r = Release { id: "R1".into(), features: vec!["ghost".into()] };
    assert!(validate_release(&r, &deliverables()).iter().any(|v| v.path == "features" && v.message.contains("ghost")));
}
