//! Tests for reachability over the fixture inventory: roots, the resolver, the per-edge criterion.

use super::super::csharp_inventory::load_inventory;
use crate::pf::csharp_reach::*;

const FIXTURE: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");

fn opts(roots: &[&str], track: &[&str]) -> ReachOptions {
    ReachOptions {
        roots: roots.iter().map(|r| Root::parse(r).expect("root spec")).collect(),
        track: track.iter().map(|r| Root::parse(r).expect("track spec")).collect(),
    }
}

#[test]
fn root_specs_parse() {
    assert_eq!(Root::parse("entry-point"), Some(Root::EntryPoint));
    assert_eq!(Root::parse("attribute:T:X.A"), Some(Root::Attribute("T:X.A".into())));
    assert_eq!(Root::parse("member:M:X.Main(System.String[])"), Some(Root::Member("M:X.Main(System.String[])".into())));
    assert_eq!(Root::parse("bogus"), None);
}

#[test]
fn entry_point_reaches_through_the_container() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &[]));
    let unreached = |t: &str| report.unreached.iter().any(|u| u == t);
    // Main resolves OrdersEndpoints (service locator), whose constructor takes
    // IHandler<…>: the registered handler is reached, and through it the repository.
    assert!(!unreached("T:Shop.Api.Orders.PlaceOrderHandler"));
    assert!(!unreached("T:Shop.Api.Persistence.OrderRepository"));
    assert!(!unreached("T:Shop.Api.Infrastructure.AlwaysValid`1"), "open-generic typeof pair");
    assert!(!unreached("T:Shop.Api.Infrastructure.MoneyComparer"), "an external abstraction with an in-solution registration");
    assert!(unreached("T:Shop.Api.Persistence.Dead"));
    assert!(unreached("T:Shop.Api.Orders.CheckoutService"));
}

#[test]
fn composition_edges_are_classified_by_role() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &[]));
    let r = &report.resolution;
    assert_eq!(r.in_denominator, r.resolved + r.partial + r.unresolved);
    assert!(r.partial >= 2, "IClockFactory and IServiceProvider are factory/provider: {:?}", r.by_role);
    assert_eq!(r.by_reason.get("conditional-registration"), Some(&1), "{:?}", r.by_reason);
    assert!(r.by_role.contains_key("generic-dispatch"), "IHandler<T>, IValidator<T>: {:?}", r.by_role);
    // A marker and a data contract arriving as constructor parameters are
    // composition edges the criterion excludes; a type test on the marker and
    // a data contract as a method parameter are not composition edges at all.
    assert_eq!(r.excluded_by_role.get("marker"), Some(&1), "{:?}", r.excluded_by_role);
    assert_eq!(r.excluded_by_role.get("data-contract"), Some(&1), "{:?}", r.excluded_by_role);
    assert_eq!(r.composition_edges, r.in_denominator + 2);
}

#[test]
fn unresolved_and_partial_are_their_own_categories() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &[]));
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Api.Infrastructure.ConsoleAudit"), "conditionally registered: unresolved, not unreached");
    assert!(report.total.unresolved >= 1);
    let t = &report.total;
    assert_eq!(t.types, t.reached + t.unresolved + t.partial + t.unreached);
}

#[test]
fn tracked_sets_get_their_own_disposition() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &["implements:T:Shop.Api.Infrastructure.IHandler`1", "implements:T:Shop.Api.Infrastructure.IAudit"]));
    assert_eq!(report.tracked[0].reached, 1, "the handler");
    assert_eq!(report.tracked[1].unresolved, 1, "the conditionally registered audit");
    let text = render_reach(&report);
    assert!(text.starts_with("roots: entry-point\n"));
    assert!(text.contains("resolution coverage:") && text.contains("role proxies"));
}

#[test]
fn a_named_member_is_a_root() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["member:M:Shop.Api.Program.Main(System.String[])"], &[]));
    let entry = reach(&inv, &opts(&["entry-point"], &[]));
    assert_eq!(report.total.reached, entry.total.reached);
}

#[test]
fn attribute_and_implements_roots_select_declared_symbols() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["attribute:T:Shop.Api.Infrastructure.EndpointAttribute", "implements:T:Shop.Api.Infrastructure.IHandler`1"], &[]));
    assert_eq!(report.by_root[0].root_symbols, 1, "one [Endpoint] method");
    assert_eq!(report.by_root[1].root_symbols, 1, "one IHandler implementor");
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Domain.OrderPlaced"));
}
