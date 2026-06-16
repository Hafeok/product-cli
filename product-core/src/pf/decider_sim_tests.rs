//! Tests for the Decider interpreter + scenario simulation.

use super::*;
use crate::pf::decider::Decider;
use crate::pf::decider_logic::*;

fn status(s: &str) -> State {
    let mut m = State::new();
    m.insert("status".into(), Scalar::Str(s.into()));
    m
}

fn pred_eq(field: &str, value: &str) -> Predicate {
    Predicate { field: field.into(), eq: Some(Scalar::Str(value.into())), ne: None, any_of: None, exists: None }
}

fn order_logic() -> DeciderLogic {
    DeciderLogic {
        initial: status("none"),
        evolve: vec![
            EvolveRule { on: "OrderPlaced".into(), set: status("placed") },
            EvolveRule { on: "OrderPaid".into(), set: status("paid") },
        ],
        decide: vec![
            DecideRule {
                on: "PlaceOrder".into(),
                guards: vec![Guard { when: Some(pred_eq("status", "none")), expr: None, else_reject: "no-double-place".into() }],
                emit: vec!["OrderPlaced".into()],
            },
            DecideRule {
                on: "PayOrder".into(),
                guards: vec![Guard { when: Some(pred_eq("status", "placed")), expr: None, else_reject: "pay-only-placed".into() }],
                emit: vec!["OrderPaid".into()],
            },
        ],
    }
}

fn emitted(event: &str) -> EmittedEvent {
    EmittedEvent { event: event.into(), payload: Payload::new() }
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
    let s = replay(&logic, &["OrderPlaced".into()]).expect("replay");
    assert_eq!(s.get("status"), Some(&Scalar::Str("placed".into())));
}

#[test]
fn decide_accepts_valid_and_rejects_with_the_invariant() {
    let logic = order_logic();
    let fresh = replay(&logic, &[]).expect("replay");
    assert_eq!(decide(&logic, &fresh, &"PayOrder".into()).expect("decide"), Outcome::Rejected("pay-only-placed".into()));
    let placed = replay(&logic, &["OrderPlaced".into()]).expect("replay");
    assert_eq!(decide(&logic, &placed, &"PayOrder".into()).expect("decide"), Outcome::Accepted(vec![emitted("OrderPaid")]));
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
    d.scenarios[1].then = Expectation::emit(vec!["OrderPaid".into()]);
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "scenario" && x.message.contains("cannot pay before placing")), "{vs:?}");
}

#[test]
fn an_uncovered_command_is_incomplete() {
    let mut d = order_decider();
    d.scenarios.retain(|s| s.when.id() != "PlaceOrder");
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "completeness" && x.message.contains("PlaceOrder")), "{vs:?}");
}

#[test]
fn no_logic_is_a_violation() {
    let d = Decider { id: "x".into(), decides_for: "Order".into(), ..Default::default() };
    let vs = simulate(&d);
    assert!(vs.iter().any(|x| x.path == "logic"));
}

/// A single-field payload (keeps the CEL test under the function-length limit).
fn pay(field: &str, value: Scalar) -> Payload {
    let mut p = Payload::new();
    p.insert(field.into(), value);
    p
}

fn account_logic() -> DeciderLogic {
    DeciderLogic {
        initial: State::new(),
        evolve: vec![EvolveRule { on: "Opened".into(), set: pay("limit", Scalar::Str("=event.limit".into())) }],
        decide: vec![DecideRule {
            on: "Charge".into(),
            guards: vec![Guard { when: None, expr: Some("command.amount <= state.limit".into()), else_reject: "over-limit".into() }],
            emit: vec![EventRef::Data { event: "Charged".into(), with: pay("amount", Scalar::Str("=command.amount".into())) }],
        }],
    }
}

fn charge(amount: i64) -> CommandRef {
    CommandRef::Data { command: "Charge".into(), with: pay("amount", Scalar::Int(amount)) }
}

fn opened(limit: i64) -> EventRef {
    EventRef::Data { event: "Opened".into(), with: pay("limit", Scalar::Int(limit)) }
}

#[test]
fn cel_guard_and_payload_flow_through() {
    // A CEL guard over command payload + an emitted event carrying a computed
    // payload, with state derived from a prior event's payload.
    let d = Decider {
        id: "acct".into(),
        decides_for: "Account".into(),
        handles: vec!["Charge".into()],
        emits: vec!["Charged".into()],
        logic: Some(account_logic()),
        scenarios: vec![
            Scenario {
                name: "charge within limit".into(),
                given: vec![opened(100)],
                when: charge(40),
                then: Expectation::emit(vec![EventRef::Data { event: "Charged".into(), with: pay("amount", Scalar::Int(40)) }]),
            },
            Scenario {
                name: "charge over limit".into(),
                given: vec![opened(100)],
                when: charge(250),
                then: Expectation::reject("over-limit"),
            },
        ],
        ..Default::default()
    };
    let vs = simulate(&d);
    assert!(vs.is_empty(), "{vs:?}");
}
