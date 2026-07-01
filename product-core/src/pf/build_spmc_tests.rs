//! Tests for the SPMC emit (build_spmc).

use super::*;
use crate::pf::deliverable::AcceptanceCriterion;
use crate::pf::feature::Feature;
use crate::pf::work_unit::{Context, Produces, WorkUnit};

fn deliverable(runner: Option<&str>) -> Deliverable {
    Deliverable {
        id: "place-order".into(),
        feature: "place-order".into(),
        acceptance: vec![AcceptanceCriterion {
            id: "handler-exists".into(),
            statement: "a handler writes OrderPlaced".into(),
            status: "pending".into(),
            runner: runner.map(String::from),
            runner_args: runner.map(|_| "tc_handler".into()),
        }],
    }
}

fn unit(id: &str, path: &str) -> WorkUnit {
    WorkUnit {
        id: id.into(),
        schema: "spmc/1".into(),
        prompt: format!("implement {id}"),
        model: None,
        context: Context::default(),
        produces: Produces { artifact: "code".into(), path: path.into() },
        applies: vec!["slice-adapter".into()],
        trace: None,
    }
}

fn spmc_with(runner: Option<&str>, units: &[WorkUnit]) -> String {
    let feature = Feature { id: "place-order".into(), anchors: vec![], depth: None };
    emit_session_spmc(&deliverable(runner), &feature, &DomainGraph::default(), None, &[], units, "bookstore")
}

#[test]
fn emits_the_operating_contract_and_build_plan() {
    let s = spmc_with(Some("cargo-test"), &[unit("handler-order", "src/handler.rs")]);
    assert!(s.contains("⟦Ω:SPMC⟧"), "carries the SPMC marker");
    assert!(s.contains("Operating contract"));
    // the work unit and its exact path appear in the build plan
    assert!(s.contains("`handler-order` → write `src/handler.rs`"));
    // a bound runner becomes a concrete verify command
    assert!(s.contains("`handler-exists`: `cargo test tc_handler`"));
    assert!(s.contains("## Done when"));
}

#[test]
fn unbound_acceptance_falls_back_to_manual_with_a_hint() {
    let s = spmc_with(None, &[unit("handler-order", "src/handler.rs")]);
    assert!(s.contains("no acceptance criterion carries a runner"));
    assert!(s.contains("handler-exists: a handler writes OrderPlaced"));
}

#[test]
fn no_units_says_dispatch_first() {
    let s = spmc_with(Some("shell"), &[]);
    assert!(s.contains("no work units"));
    assert!(s.contains("cell dispatch"));
}
