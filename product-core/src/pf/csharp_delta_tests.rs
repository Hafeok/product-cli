//! Tests for the three-region delta over the fixture inventory + the shipped event model.

use super::super::csharp_inventory::load_inventory;
const FIXTURE: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");
use super::super::eventmodel::load_event_model;
use crate::pf::csharp_delta::*;

const ORDERING: &str =
    include_str!("../../../product-cli/tests/fixtures/csharp-inventory/ordering.eventmodel.yaml");

fn run() -> DeltaReport {
    let inv = load_inventory(FIXTURE).expect("loads");
    let model = load_event_model(ORDERING).expect("loads");
    delta(&inv, &model, &DeltaOptions::default())
}

fn row<'a>(r: &'a DeltaReport, address: &str) -> &'a ActRow {
    r.acts.iter().find(|a| a.address == address).expect(address)
}

#[test]
fn every_act_gets_exactly_one_region() {
    let r = run();
    assert_eq!(r.acts.len(), 6);
    assert_eq!(row(&r, "command:PlaceOrder").region, Region::Declared);
    assert_eq!(row(&r, "command:PlaceOrder").symbols, ["T:Shop.Api.Orders.PlaceOrderHandler"]);
    // CartService references Cart, ItemAddedToCart, CartEmptied — covered by the Cart read-model only.
    assert_eq!(row(&r, "read-model:Cart").region, Region::Declarable);
    assert!(row(&r, "read-model:Cart").symbols.contains(&"T:Shop.Api.Orders.CartService".to_string()));
    // ConfirmOrder's facts are touched only by the spanning CheckoutService.
    assert_eq!(row(&r, "command:ConfirmOrder").region, Region::Unstructured);
    assert_eq!(row(&r, "read-model:OrderSummary").region, Region::Unrealised);
}

#[test]
fn the_boundary_runs_through_checkout_service() {
    let r = run();
    let span = r.spanning_types.iter().find(|s| s.type_id == "T:Shop.Api.Orders.CheckoutService").expect("spanning");
    assert_eq!(span.facts, ["ActorIdentity", "Cart", "OrderConfirmed", "OrderPlaced"]);
    assert!(span.acts_touched.contains(&"command:PlaceOrder".to_string()));
    assert!(span.acts_touched.contains(&"command:ConfirmOrder".to_string()));
}

#[test]
fn unresolved_and_orphan_declarations_are_named() {
    let r = run();
    assert_eq!(r.declared_unresolved, [("T:Shop.Api.Orders.BogusHandler".to_string(), "Nonexistent".to_string())]);
    assert_eq!(r.fact_orphans, [("T:Shop.Domain.Ghost".to_string(), "Ghost".to_string())]);
    assert!(r.facts_unrealised.is_empty(), "{:?}", r.facts_unrealised);
}

#[test]
fn ratios_are_three_way_over_undeclared_types() {
    let r = run();
    // 33 types; 2 [Slice] + 8 [RealisesFact] carry declarations.
    assert_eq!(r.ratios.undeclared_types, 33 - 10);
    assert_eq!(r.ratios.reachable_undeclared + r.ratios.unresolved_undeclared + r.ratios.isolated_undeclared, r.ratios.undeclared_types);
    assert!(r.ratios.reachable_undeclared >= 4, "the handler reaches ICartReader/CartService and IOrderRepository/OrderRepository: {:?}", r.ratios);
    assert_eq!(r.attribution.grade, "authored");
    let text = render_delta(&r);
    assert!(text.contains("graded authored, CG-R-57"));
    assert!(text.contains("unresolved-undeclared:"));
}

#[test]
fn shared_value_object_is_neither_region() {
    // Money carries no fact and references none: it must not appear as spanning.
    let r = run();
    assert!(!r.spanning_types.iter().any(|s| s.type_id == "T:Shop.Domain.Money"));
    let text = render_delta(&r);
    assert!(text.contains("known divergence"));
    assert!(text.contains("Declared: 1 act(s)"));
}
