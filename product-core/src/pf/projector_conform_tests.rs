//! Unit tests for projection conformance — oracle equality, mismatch diffs.


use super::*;
use crate::pf::decider_logic::{EvolveRule, Scalar};
use crate::pf::projector_logic::{ProjectorLogic, ProjectorScenario};

fn state(pairs: &[(&str, Scalar)]) -> State {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn fixture() -> Projector {
    Projector {
        id: "ordersummary-projector".into(),
        projects_for: "OrderSummary".into(),
        folds: vec!["OrderPlaced".into()],
        over: vec!["Order".into()],
        logic: Some(ProjectorLogic {
            initial: state(&[("count", Scalar::Int(0))]),
            apply: vec![EvolveRule {
                on: "OrderPlaced".into(),
                set: state(&[("count", Scalar::Str("=view.count + 1".into()))]),
            }],
        }),
        scenarios: vec![ProjectorScenario {
            name: "one order counted".into(),
            given: vec![EventRef::Id("OrderPlaced".into())],
            then: state(&[("count", Scalar::Int(1))]),
        }],
    }
}

#[test]
fn requests_mirror_the_scenarios_in_order() {
    let reqs = requests(&fixture());
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].given.len(), 1);
}

#[test]
fn matching_views_are_conformant() {
    let realised = vec![state(&[("count", Scalar::Int(1))])];
    assert!(check_conformance(&fixture(), &realised).is_empty());
}

#[test]
fn a_differing_view_is_a_finding_with_the_oracle_diff() {
    let realised = vec![state(&[("count", Scalar::Int(2))])];
    let findings = check_conformance(&fixture(), &realised);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("oracle"));
    // Full-state equality: an extra field is also non-conformant.
    let extra = vec![state(&[("count", Scalar::Int(1)), ("ghost", Scalar::Bool(true))])];
    assert_eq!(check_conformance(&fixture(), &extra).len(), 1);
}

#[test]
fn wrong_arity_and_missing_logic_are_findings() {
    assert_eq!(check_conformance(&fixture(), &[]).len(), 1);
    let mut bare = fixture();
    bare.logic = None;
    assert_eq!(check_conformance(&bare, &[state(&[])]).len(), 1);
}
