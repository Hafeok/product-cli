//! Unit tests for the DeployableUnit model (§4/§4.2 shape validation).

use super::*;
use crate::pf::model::DomainGraph;
use crate::pf::model_ui::System;

fn graph_with_system(id: &str) -> DomainGraph {
    let mut g = DomainGraph::default();
    g.systems.push(System { id: id.to_string(), label: id.to_string(), kind: "application".to_string(), ..Default::default() });
    g
}

fn valid_unit() -> DeployableUnit {
    DeployableUnit {
        id: "shop-ios".to_string(),
        built_from: "rn-hexagonal-app".to_string(),
        deploys_system: vec!["acme-shop".to_string()],
        environment: Some("production".to_string()),
        identity: DeploymentIdentity { bundle_id: Some("com.acme.shop".to_string()), ..Default::default() },
    }
}

#[test]
fn yaml_round_trips() {
    let du = valid_unit();
    let back = DeployableUnit::from_yaml(&du.to_yaml().expect("yaml")).expect("reparse");
    assert_eq!(du, back);
}

#[test]
fn a_well_formed_unit_is_conformant() {
    let g = graph_with_system("acme-shop");
    let out = validate_deployable_unit(&valid_unit(), Some(&g), &["rn-hexagonal-app".to_string()]);
    assert!(out.is_empty(), "expected no violations, got {out:?}");
}

#[test]
fn missing_built_from_is_a_violation() {
    let mut du = valid_unit();
    du.built_from = String::new();
    let out = validate_deployable_unit(&du, None, &[]);
    assert!(out.iter().any(|v| v.path == "built_from"));
}

#[test]
fn unknown_blueprint_is_a_violation() {
    let out = validate_deployable_unit(&valid_unit(), None, &["some-other-blueprint".to_string()]);
    assert!(out.iter().any(|v| v.path == "built_from"));
}

#[test]
fn no_system_is_a_violation() {
    let mut du = valid_unit();
    du.deploys_system.clear();
    let out = validate_deployable_unit(&du, None, &[]);
    assert!(out.iter().any(|v| v.path == "deploys_system"));
}

#[test]
fn deploys_system_must_resolve_to_a_system_node() {
    let g = graph_with_system("acme-shop");
    let mut du = valid_unit();
    du.deploys_system = vec!["not-a-system".to_string()];
    let out = validate_deployable_unit(&du, Some(&g), &[]);
    assert!(out.iter().any(|v| v.path == "deploys_system"));
}

#[test]
fn empty_identity_is_a_violation() {
    let mut du = valid_unit();
    du.identity = DeploymentIdentity::default();
    let out = validate_deployable_unit(&du, None, &[]);
    assert!(out.iter().any(|v| v.path == "identity"));
}

#[test]
fn legacy_units_with_no_blueprint_list_skip_the_resolve_check() {
    // When no blueprint list is supplied (offline), built_from resolution is skipped.
    let out = validate_deployable_unit(&valid_unit(), None, &[]);
    assert!(out.is_empty(), "expected no violations without a graph or blueprint list, got {out:?}");
}
