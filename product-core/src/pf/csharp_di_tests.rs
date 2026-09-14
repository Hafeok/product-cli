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
    assert_eq!(r.registrations_read, 14, "incl. AddCheck<PingCheck> on the chained IHealthChecksBuilder (R-2) and TryAddEnumerable(ServiceDescriptor.Singleton<,>()) (O-18)");
    assert_eq!(r.resolve("T:Shop.Api.Infrastructure.INotifier", &all).map(|v| v[0].implementation.as_str()), Ok("T:Shop.Api.Infrastructure.ConsoleNotifier"));
    assert_eq!(r.provider_of("T:Microsoft.Extensions.Logging.ILogger`1", &all), Some("host builder"), "CG-R-87: implicit host-builder registrations once the entry point is reached");
    assert_eq!(r.provider_of("T:Microsoft.Extensions.Logging.ILogger`1", &|_| false), None);
    assert_eq!(r.calls_ignored, 3, "BuildServiceProvider, AddMemoryCache, AddHealthChecks are calls the resolver does not parse");
    assert_eq!(r.ignored_by_method.get("BuildServiceProvider"), Some(&1));
    assert_eq!(r.test_sites_skipped, 3, "the test project's registrations are not production composition (CG-R-75)");
    assert_eq!(r.provider_of("T:Microsoft.Extensions.Caching.Memory.IMemoryCache", &all), Some("AddMemoryCache"));
    assert!(r.unlearned(&all).is_empty(), "every reached external call is parsed or known: {:?}", r.unlearned(&all));
}
