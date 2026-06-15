//! Tests for the How-contract conformance checker.

use super::*;
use crate::pf::how::*;

const EXAMPLE: &str = include_str!("../../../schema/examples/how-contract.example.yaml");

fn example() -> HowContract {
    HowContract::from_yaml(EXAMPLE).expect("parse")
}

#[test]
fn bundled_example_has_no_blocking_violations() {
    // The example carries one soft warning (a decision licenses a principle id
    // it doesn't define), but no blocking violation.
    let results = validate_how(&example());
    assert!(!has_blocking(&results), "unexpected blocking violations: {results:?}");
}

#[test]
fn top_decision_without_rationale_is_rejected() {
    let mut c = example();
    c.top_decisions[0].rationale = String::new();
    let vs = validate_how(&c);
    assert!(vs.iter().any(|x| x.path == "rationale" && x.message.contains("§4.1")));
}

#[test]
fn pattern_without_principle_is_rejected() {
    let mut c = example();
    c.patterns[0].realizes.clear();
    assert!(validate_how(&c).iter().any(|x| x.path == "realizes"));
}

#[test]
fn pattern_realizing_unknown_principle_is_rejected() {
    let mut c = example();
    c.patterns[0].realizes = vec!["ghost-principle".into()];
    assert!(validate_how(&c).iter().any(|x| x.path == "realizes"));
}

#[test]
fn infra_not_satisfying_app_contract_is_rejected() {
    let mut c = example();
    if let Some(infra) = c.infrastructure_contract.as_mut() {
        infra.satisfies = "wrong-contract".into();
    }
    assert!(validate_how(&c).iter().any(|x| x.path == "conformsTo"));
}

#[test]
fn interface_without_derived_from_is_rejected() {
    let mut c = example();
    c.interface_contracts[0].derived_from.clear();
    assert!(validate_how(&c).iter().any(|x| x.path == "derivedFrom"));
}

#[test]
fn crown_rule_applied_principle_must_be_enforced() {
    let mut c = example();
    // result-type is realised by the result-type pattern, which is applied_by
    // [add-crud-resource] → it is "applied". Strip its enforcement → trace lie.
    for p in c.principles.iter_mut() {
        if p.id == "explicit-error-handling" {
            p.enforced_by.clear();
        }
    }
    let vs = validate_how(&c);
    assert!(
        vs.iter().any(|x| x.path == "trace" && x.message.contains("trace must be true")),
        "expected a trace-truth violation, got {vs:?}"
    );
}

#[test]
fn unapplied_unenforced_principle_fails_earn_their_place() {
    let mut c = example();
    c.principles.push(Principle {
        id: "orphan".into(),
        statement: "a principle nobody applies or enforces".into(),
        ..Default::default()
    });
    assert!(validate_how(&c).iter().any(|x| x.path == "earn-their-place"));
}
