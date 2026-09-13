//! Tests for the event-model loader over the binding's shipped example.

use crate::pf::eventmodel::*;

const ORDERING: &str =
    include_str!("../../../product-cli/tests/fixtures/csharp-inventory/ordering.eventmodel.yaml");

#[test]
fn shipped_example_loads() {
    let m = load_event_model(ORDERING).expect("loads");
    assert_eq!(m.context, "ordering");
    assert_eq!(m.facts.len(), 7);
    assert_eq!(m.slices.len(), 6);
    assert!(m.has_fact("Cart"));
    assert!(!m.has_fact("Ghost"));
}

#[test]
fn positions_are_relative_to_the_act() {
    let m = load_event_model(ORDERING).expect("loads");
    let place = m.slices_named("PlaceOrder");
    assert_eq!(place.len(), 1);
    assert_eq!(place[0].address(), "command:PlaceOrder");
    let facts = place[0].facts();
    assert!(facts.contains("Cart") && facts.contains("ActorIdentity") && facts.contains("OrderPlaced"));
    // Cart is read by three commands and written by the read-model.
    let touching: Vec<String> = m.acts_touching("Cart").iter().map(|s| s.address()).collect();
    assert_eq!(touching, ["command:AddToCart", "read-model:Cart", "command:PlaceOrder", "command:EmptyCart"]);
}

#[test]
fn duplicate_address_is_refused() {
    let text = "slices:\n  - {type: command, name: A}\n  - {type: command, name: A}\n";
    assert!(load_event_model(text).is_err());
}
