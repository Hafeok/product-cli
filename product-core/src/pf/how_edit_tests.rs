//! Tests for granular How-contract mutation.

use super::*;
use crate::pf::how_validate::{has_blocking, validate_how};

fn empty() -> HowContract {
    HowContract { blueprint: "demo".into(), ..Default::default() }
}

#[test]
fn build_a_conformant_how_from_scratch() {
    let mut c = empty();
    set_app_contract(&mut c, ApplicationContract {
        id: "app".into(), language: "Rust".into(),
        statements: vec![ContractStatement { id: "s1".into(), statement: "deps point inward".into(), enforced_by: Some("layering-audit".into()) }],
        ..Default::default()
    });
    add_decision(&mut c, TopDecision { id: "slices".into(), decision: "vertical slices".into(), rationale: "change locality".into(), licenses: vec!["cohesion".into()], ..Default::default() }).expect("dec");
    add_principle(&mut c, Principle { id: "cohesion".into(), statement: "code that changes together lives together".into(), enforced_by: vec!["layout-audit".into()], ..Default::default() }).expect("prin");
    add_pattern(&mut c, Pattern { id: "slice-folder".into(), shape: "one folder per feature".into(), realizes: vec!["cohesion".into()], ..Default::default() }).expect("pat");
    set_infra_contract(&mut c, InfrastructureContract { id: "infra".into(), satisfies: "app".into(), frozen: true, resources: vec![] });
    add_resource(&mut c, Resource { id: "compute".into(), kind: "compute".into(), choice: "Container Apps".into(), ..Default::default() }).expect("res");
    add_interface(&mut c, InterfaceContract { id: "api".into(), surface: "rest".into(), standard: "OpenAPI".into(), derived_from: vec!["domain:Task".into()] }).expect("iface");

    assert!(!has_blocking(&validate_how(&c)), "{:?}", validate_how(&c));
    assert_eq!(c.top_decisions.len(), 1);
    assert_eq!(c.application_contract.statements.len(), 1);
    assert_eq!(c.infrastructure_contract.as_ref().unwrap().resources.len(), 1);
}

#[test]
fn duplicate_id_across_why_cascade_is_rejected() {
    let mut c = empty();
    add_principle(&mut c, Principle { id: "x".into(), statement: "s".into(), ..Default::default() }).expect("p");
    // a pattern reusing the principle's id is rejected (they reference each other by id)
    assert!(add_pattern(&mut c, Pattern { id: "x".into(), shape: "s".into(), realizes: vec!["x".into()], ..Default::default() }).is_err());
}

#[test]
fn app_statement_requires_a_contract() {
    let mut c = empty();
    assert!(add_app_statement(&mut c, ContractStatement { id: "s".into(), statement: "x".into(), enforced_by: None }).is_err());
    set_app_contract(&mut c, ApplicationContract { id: "app".into(), language: "Rust".into(), ..Default::default() });
    assert!(add_app_statement(&mut c, ContractStatement { id: "s".into(), statement: "x".into(), enforced_by: None }).is_ok());
    // duplicate statement id rejected
    assert!(add_app_statement(&mut c, ContractStatement { id: "s".into(), statement: "y".into(), enforced_by: None }).is_err());
}

#[test]
fn resource_requires_infra_contract() {
    let mut c = empty();
    assert!(add_resource(&mut c, Resource { id: "r".into(), kind: "compute".into(), choice: "x".into(), ..Default::default() }).is_err());
    set_infra_contract(&mut c, InfrastructureContract { id: "infra".into(), satisfies: "app".into(), frozen: true, resources: vec![] });
    assert!(add_resource(&mut c, Resource { id: "r".into(), kind: "compute".into(), choice: "x".into(), ..Default::default() }).is_ok());
}
