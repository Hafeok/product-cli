//! Tests for reachability through the resolver over the fixture inventory.

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
    assert_eq!(Root::parse("implements:T:X.I`1"), Some(Root::Implements("T:X.I`1".into())));
    assert_eq!(Root::parse("member:M:X.Main(System.String[])"), Some(Root::Member("M:X.Main(System.String[])".into())));
    assert_eq!(Root::parse("bogus"), None);
}

#[test]
fn entry_point_reaches_through_the_container() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &[]));
    assert_eq!(report.by_root[0].root_symbols, 1);
    let unreached = |t: &str| report.unreached.iter().any(|u| u == t);
    // Main resolves OrdersEndpoints, whose constructor takes IHandler<…>:
    // the registered handler is reached, and through it the repository.
    assert!(!unreached("T:Shop.Api.Orders.PlaceOrderHandler"));
    assert!(!unreached("T:Shop.Api.Persistence.OrderRepository"));
    assert!(!unreached("T:Shop.Api.Infrastructure.AlwaysValid`1"), "open-generic typeof pair");
    assert!(!unreached("T:Shop.Api.Infrastructure.GuidGenerator"), "factory lambda constructing one type");
    // Dead is referenced by nothing; CheckoutService by nothing either.
    assert!(unreached("T:Shop.Api.Persistence.Dead"));
    assert!(unreached("T:Shop.Api.Orders.CheckoutService"));
}

#[test]
fn unresolved_is_its_own_category() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &[]));
    let r = &report.resolution;
    assert_eq!(r.interface_edges, r.resolved + r.unresolved);
    assert_eq!(r.by_reason.get("conditional-registration"), Some(&1), "{:?}", r.by_reason);
    assert_eq!(r.by_reason.get("no-registration"), Some(&1), "{:?}", r.by_reason);
    // ConsoleAudit implements IAudit, whose edge was unresolved: unresolved, not unreached.
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Api.Infrastructure.ConsoleAudit"));
    assert!(report.total.unresolved >= 1);
    assert_eq!(report.total.types, report.total.reached + report.total.unresolved + report.total.unreached);
}

#[test]
fn tracked_sets_get_their_own_disposition() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &["implements:T:Shop.Api.Infrastructure.IHandler`1", "implements:T:Shop.Api.Infrastructure.IAudit"]));
    assert_eq!(report.tracked[0].reached, 1, "the handler");
    assert_eq!(report.tracked[1].unresolved, 1, "the conditionally registered audit");
    let text = render_reach(&report);
    assert!(text.starts_with("roots: entry-point\n"));
    assert!(text.contains("resolution coverage:"));
    assert!(text.contains("tracked:"));
}

#[test]
fn a_named_member_is_a_root() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["member:M:Shop.Api.Program.Main(System.String[])"], &[]));
    let entry = reach(&inv, &opts(&["entry-point"], &[]));
    assert_eq!(report.total.reached, entry.total.reached);
    assert_eq!(report.by_root[0].root_symbols, 1);
}

#[test]
fn attribute_and_implements_roots_select_declared_symbols() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["attribute:T:Shop.Api.Infrastructure.EndpointAttribute", "implements:T:Shop.Api.Infrastructure.IHandler`1"], &[]));
    assert_eq!(report.by_root[0].root_symbols, 1, "one [Endpoint] method");
    assert_eq!(report.by_root[1].root_symbols, 1, "one IHandler implementor");
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Domain.OrderPlaced"));
}
