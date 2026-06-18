//! Tests for the Projector simulation (§3.4) — sound + complete before realisation.

use super::*;
use crate::pf::decider_logic::{EventRef, EvolveRule, Scalar, State};
use crate::pf::projector::Projector;
use crate::pf::projector_logic::{ProjectorLogic, ProjectorScenario};

fn ev(id: &str) -> EventRef {
    id.into()
}

fn rule(on: &str, key: &str, val: &str) -> EvolveRule {
    let mut set = State::new();
    set.insert(key.to_string(), Scalar::Str(val.to_string()));
    EvolveRule { on: on.to_string(), set }
}

fn status(val: &str) -> State {
    let mut m = State::new();
    m.insert("status".to_string(), Scalar::Str(val.to_string()));
    m
}

fn status_projector() -> Projector {
    Projector {
        id: "rm-orders-projector".into(),
        projects_for: "rm-orders".into(),
        folds: vec!["OrderPlaced".into(), "OrderShipped".into()],
        over: vec!["Order".into()],
        logic: Some(ProjectorLogic {
            initial: State::new(),
            apply: vec![rule("OrderPlaced", "status", "placed"), rule("OrderShipped", "status", "shipped")],
        }),
        scenarios: vec![ProjectorScenario {
            name: "placed then shipped".into(),
            given: vec![ev("OrderPlaced"), ev("OrderShipped")],
            then: status("shipped"),
        }],
    }
}

#[test]
fn a_sound_and_complete_projector_simulates_clean() {
    let p = status_projector();
    assert!(simulate(&p).is_empty(), "{:?}", simulate(&p));
}

#[test]
fn project_folds_events_into_the_view() {
    let p = status_projector();
    let logic = p.logic.as_ref().expect("logic");
    let st = project(logic, &[ev("OrderPlaced")]).expect("project");
    assert_eq!(st, status("placed"));
}

#[test]
fn a_wrong_expectation_fails_soundness() {
    let mut p = status_projector();
    p.scenarios[0].then = status("placed"); // the projection actually ends "shipped"
    assert!(simulate(&p).iter().any(|x| x.message.contains("failed")));
}

#[test]
fn an_unexercised_folded_event_fails_completeness() {
    let mut p = status_projector();
    p.folds.push("OrderCancelled".into()); // folded but no scenario exercises it
    assert!(simulate(&p).iter().any(|x| x.message.contains("OrderCancelled") && x.message.contains("incomplete")));
}
