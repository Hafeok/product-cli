//! Tests for DI resolution over the fixture's registration facts.

use super::super::csharp_inventory::load_inventory;
use crate::pf::csharp_di::*;

const FIXTURE: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");

fn resolver() -> Resolver {
    Resolver::build(&load_inventory(FIXTURE).expect("loads"))
}

fn all(_: &str) -> bool {
    true
}

#[test]
fn explicit_generic_registration_resolves() {
    let r = resolver();
    let got = r.resolve("T:Shop.Api.Persistence.IOrderRepository", &all).expect("resolved");
    assert_eq!(got[0].implementation, "T:Shop.Api.Persistence.OrderRepository");
    assert_eq!(got[0].via, Via::Generic);
}

#[test]
fn typeof_pair_and_factory_construct_resolve() {
    let r = resolver();
    let v = r.resolve("T:Shop.Api.Infrastructure.IValidator`1", &all).expect("open generic");
    assert_eq!((v[0].implementation.as_str(), v[0].via), ("T:Shop.Api.Infrastructure.AlwaysValid`1", Via::TypeofPair));
    let g = r.resolve("T:Shop.Api.Infrastructure.IIdGenerator", &all).expect("factory");
    assert_eq!((g[0].implementation.as_str(), g[0].via), ("T:Shop.Api.Infrastructure.GuidGenerator", Via::FactoryConstruct));
    let s = r.resolve("T:Shop.Api.Orders.OrdersEndpoints", &all).expect("self");
    assert_eq!(s[0].via, Via::SelfRegistration);
}

#[test]
fn unresolved_reasons_are_named() {
    let r = resolver();
    assert_eq!(r.resolve("T:Shop.Api.Infrastructure.IAudit", &all), Err(Reason::ConditionalRegistration));
    assert_eq!(r.resolve("T:Shop.Api.Infrastructure.IClock", &all), Err(Reason::NoRegistration));
    // The registration site was never reached: module configuration.
    assert_eq!(r.resolve("T:Shop.Api.Persistence.IOrderRepository", &|_| false), Err(Reason::ModuleConfiguration));
    assert_eq!(r.registrations_read, 7);
    assert_eq!(r.calls_ignored, 1, "BuildServiceProvider is a call, not a registration");
    assert_eq!(r.ignored_by_method.get("BuildServiceProvider"), Some(&1));
}
