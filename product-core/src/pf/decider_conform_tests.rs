//! Tests for behavioural conformance — realised outcomes vs the Decider oracle.

use super::*;
use crate::pf::decider::Decider;
use crate::pf::decider_logic::*;

fn status(s: &str) -> State {
    let mut m = State::new();
    m.insert("status".into(), Scalar::Str(s.into()));
    m
}

fn guard(value: &str, reject: &str) -> Guard {
    Guard {
        when: Some(Predicate { field: "status".into(), eq: Some(Scalar::Str(value.into())), ne: None, any_of: None, exists: None }),
        expr: None,
        else_reject: reject.into(),
    }
}

fn order_decider() -> Decider {
    Decider {
        id: "order-decider".into(),
        decides_for: "Order".into(),
        handles: vec!["PlaceOrder".into(), "PayOrder".into()],
        emits: vec!["OrderPlaced".into(), "OrderPaid".into()],
        logic: Some(DeciderLogic {
            initial: status("none"),
            evolve: vec![
                EvolveRule { on: "OrderPlaced".into(), set: status("placed") },
                EvolveRule { on: "OrderPaid".into(), set: status("paid") },
            ],
            decide: vec![
                DecideRule { on: "PlaceOrder".into(), guards: vec![guard("none", "no-double-place")], emit: vec!["OrderPlaced".into()] },
                DecideRule { on: "PayOrder".into(), guards: vec![guard("placed", "pay-only-placed")], emit: vec!["OrderPaid".into()] },
            ],
        }),
        scenarios: vec![
            Scenario { name: "place fresh".into(), given: vec![], when: "PlaceOrder".into(), then: Expectation::emit(vec!["OrderPlaced".into()]) },
            Scenario { name: "cannot pay unplaced".into(), given: vec![], when: "PayOrder".into(), then: Expectation::reject("pay-only-placed") },
            Scenario { name: "place then pay".into(), given: vec!["OrderPlaced".into()], when: "PayOrder".into(), then: Expectation::emit(vec!["OrderPaid".into()]) },
        ],
        ..Default::default()
    }
}

#[test]
fn requests_are_built_per_scenario() {
    let reqs = requests(&order_decider());
    assert_eq!(reqs.len(), 3);
    assert_eq!(reqs[2].when.id(), "PayOrder");
    assert_eq!(reqs[2].given[0].id(), "OrderPlaced");
}

#[test]
fn realised_matching_the_oracle_is_conformant() {
    let realised = vec![
        Expectation::emit(vec!["OrderPlaced".into()]),
        Expectation::reject("pay-only-placed"),
        Expectation::emit(vec!["OrderPaid".into()]),
    ];
    assert!(check_conformance(&order_decider(), &realised).is_empty());
}

#[test]
fn realised_diverging_from_the_oracle_is_flagged() {
    // realised wrongly accepts paying an unplaced order
    let realised = vec![
        Expectation::emit(vec!["OrderPlaced".into()]),
        Expectation::emit(vec!["OrderPaid".into()]),
        Expectation::emit(vec!["OrderPaid".into()]),
    ];
    let vs = check_conformance(&order_decider(), &realised);
    assert!(vs.iter().any(|x| x.path == "conformance" && x.message.contains("cannot pay unplaced")), "{vs:?}");
}

#[test]
fn wrong_response_count_is_flagged() {
    let realised = vec![Expectation::emit(vec!["OrderPlaced".into()])];
    let vs = check_conformance(&order_decider(), &realised);
    assert!(vs.iter().any(|x| x.path == "runner"));
}
