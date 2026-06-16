//! Tests for the Decider interpreter + scenario simulation.

use super::*;
use crate::pf::decider::Decider;
use crate::pf::decider_logic::*;

fn order_logic() -> DeciderLogic {
    let placed = |status: &str| {
        let mut s = State::new();
        s.insert("status".into(), Scalar::Str(status.into()));
        s
    };
    DeciderLogic {
        initial: placed("none"),
        evolve: vec![
            EvolveRule { on: "OrderPlaced".into(), set: placed("placed") },
            EvolveRule { on: "OrderPaid".into(), set: placed("paid") },
        ],
        decide: vec![
            DecideRule {
                on: "PlaceOrder".into(),
                guards: vec![Guard { when: Predicate { field: "status".into(), eq: Some(Scalar::Str("none".into())), ne: None, any_of: None, exists: None }, else_reject: "no-double-place".into() }],
                emit: vec!["OrderPlaced".into()],
            },
            DecideRule {
                on: "PayOrder".into(),
                guards: vec![Guard { when: Predicate { field: "status".into(), eq: Some(Scalar::Str("placed".into())), ne: None, any_of: None, exists: None }, else_reject: "pay-only-placed".into() }],
                emit: vec!["OrderPaid".into()],
            },
        ],
    }
}

fn order_decider() -> Decider {
    Decider {
        id: "order-decider".into(),
        decides_for: "Order".into(),
        handles: vec!["PlaceOrder".into(), "PayOrder".into()],
        emits: vec!["OrderPlaced".into(), "OrderPaid".into()],
        logic: Some(order_logic()),
        scenarios: vec![
            Scenario { name: "place a fresh order".into(), given: vec![], when: "PlaceOrder".into(), then: Expectation::emit(vec!["OrderPlaced".into()]) },
            Scenario { name: "cannot pay before placing".into(), given: vec![], when: "PayOrder".into(), then: Expectation::reject("pay-only-placed") },
            Scenario { name: "place then pay".into(), given: vec!["OrderPlaced".into()], when: "PayOrder".into(), then: Expectation::emit(vec!["OrderPaid".into()]) },
        ],
        ..Default::default()
    }
}

#[test]
fn replay_folds_events_into_state() {
    let logic = order_logic();
    let s = replay(&logic, &["OrderPlaced".into()]);
    assert_eq!(s.get("status"), Some(&Scalar::Str("placed".into())));
}

#[test]
fn decide_accepts_valid_and_rejects_with_the_invariant() {
    let logic = order_logic();
    let fresh = replay(&logic, &[]);
    assert_eq!(decide(&logic, &fresh, "PayOrder"), Outcome::Rejected("pay-only-placed".into()));
    let placed = replay(&logic, &["OrderPlaced".into()]);
    assert_eq!(decide(&logic, &placed, "PayOrder"), Outcome::Accepted(vec!["OrderPaid".into()]));
}

#[test]
fn a_complete_sound_decider_simulates_clean() {
    let d = order_decider();
    let vs = simulate(&d);
    assert!(vs.is_empty(), "{vs:?}");
}

#[test]
fn a_wrong_expectation_fails_the_scenario() {
    let mut d = order_decider();
    // claim paying a fresh order succeeds — it must reject
    d.scenarios[1].then = Expectation::emit(vec!["OrderPaid".into()]);
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "scenario" && x.message.contains("cannot pay before placing")), "{vs:?}");
}

#[test]
fn an_uncovered_command_is_incomplete() {
    let mut d = order_decider();
    d.scenarios.retain(|s| s.when != "PlaceOrder");
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "completeness" && x.message.contains("PlaceOrder")), "{vs:?}");
}

#[test]
fn no_logic_is_a_violation() {
    let d = Decider { id: "x".into(), decides_for: "Order".into(), ..Default::default() };
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "logic"));
}
