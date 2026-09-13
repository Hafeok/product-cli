//! Tests for reachability over the fixture inventory.

use std::collections::BTreeSet;

use super::super::csharp_inventory::load_inventory;
const FIXTURE: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");
use crate::pf::csharp_reach::*;

fn opts(roots: &[&str], di: bool) -> ReachOptions {
    ReachOptions {
        roots: roots.iter().map(|r| Root::parse(r).expect("root spec")).collect(),
        through_implementations: di,
    }
}

#[test]
fn root_specs_parse() {
    assert_eq!(Root::parse("entry-point"), Some(Root::EntryPoint));
    assert_eq!(Root::parse("attribute:T:X.A"), Some(Root::Attribute("T:X.A".into())));
    assert_eq!(Root::parse("implements:T:X.I`1"), Some(Root::Implements("T:X.I`1".into())));
    assert_eq!(Root::parse("bogus"), None);
}

#[test]
fn entry_point_without_di_stops_at_interfaces() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], false));
    assert_eq!(report.by_root[0].root_symbols, 1);
    let reached: BTreeSet<&str> = inv.types.iter().filter(|t| !report.unreached.contains(&t.id)).map(|t| t.id.as_str()).collect();
    // Main constructs the concrete services itself, so they are reached …
    assert!(reached.contains("T:Shop.Api.Orders.PlaceOrderHandler"));
    // … but Dead is reached by nothing, and Handle's body runs only through
    // the resolved interface, so CheckoutService stays unreached either way.
    assert!(report.unreached.contains(&"T:Shop.Api.Persistence.Dead".to_string()));
    assert!(report.unreached.contains(&"T:Shop.Api.Orders.CheckoutService".to_string()));
}

#[test]
fn di_traversal_reaches_more_never_less() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let without = reach(&inv, &opts(&["entry-point"], false));
    let with = reach(&inv, &opts(&["entry-point"], true));
    assert!(with.types_reached >= without.types_reached);
    assert!(with.through_implementations && !without.through_implementations);
}

#[test]
fn attribute_and_implements_roots_select_declared_symbols() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["attribute:T:Shop.Api.Infrastructure.EndpointAttribute", "implements:T:Shop.Api.Infrastructure.IHandler`1"], false));
    assert_eq!(report.by_root[0].root_symbols, 1, "one [Endpoint] method");
    assert_eq!(report.by_root[1].root_symbols, 1, "one IHandler implementor");
    assert!(!report.unreached.contains(&"T:Shop.Domain.OrderPlaced".to_string()));
}

#[test]
fn public_root_covers_the_public_surface() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["public"], false));
    assert!(report.percent_reached > 80.0, "{}", report.percent_reached);
    let text = render_reach(&report, &opts(&["public"], false));
    assert!(text.starts_with("roots: public\n"));
    assert!(text.contains("by namespace:"));
}
