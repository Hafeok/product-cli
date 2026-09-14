//! Tests for reachability over the fixture inventory: roots, the resolver, the per-edge criterion, the report form.

use super::super::csharp_ground_truth::GroundTruth;
use super::super::csharp_inventory::load_inventory;
use crate::pf::csharp_reach::*;

const FIXTURE: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/inventory.json");
const GROUND_TRUTH: &str = include_str!("../../../product-cli/tests/fixtures/csharp-inventory/ground-truth.yaml");
const COMPONENT: &str = "implements:T:Microsoft.AspNetCore.Components.ComponentBase";

fn opts(roots: &[&str], track: &[&str]) -> ReachOptions {
    ReachOptions {
        roots: roots.iter().map(|r| Root::parse(r).expect("root spec")).collect(),
        track: track.iter().map(|r| Root::parse(r).expect("track spec")).collect(),
        ground_truth: None,
    }
}

fn run(roots: &[&str]) -> ReachReport {
    reach(&load_inventory(FIXTURE).expect("loads"), &opts(roots, &[]))
}

fn edge<'a>(report: &'a ReachReport, from: &str, target: &str) -> &'a super::super::csharp_walk::CompositionEdge {
    report.resolution.edges.iter().find(|e| e.from.ends_with(from) && e.target.ends_with(target)).unwrap_or_else(|| panic!("edge {from} -> {target}"))
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
    let report = run(&["entry-point"]);
    let unreached = |t: &str| report.unreached.iter().any(|u| u == t);
    // Main resolves OrdersEndpoints (service locator), whose constructor takes
    // IHandler<…>: the registered handler is reached, and through it the repository.
    assert!(!unreached("T:Shop.Api.Orders.PlaceOrderHandler"));
    assert!(!unreached("T:Shop.Api.Persistence.OrderRepository"));
    assert!(!unreached("T:Shop.Api.Infrastructure.AlwaysValid`1"), "open-generic typeof pair");
    assert!(!unreached("T:Shop.Api.Infrastructure.MoneyComparer"), "an external abstraction with an in-solution registration");
    assert!(unreached("T:Shop.Api.Persistence.Dead"));
    assert!(unreached("T:Shop.Api.Orders.CheckoutService"), "registered by nothing: not container-constructed");
}

#[test]
fn composition_edges_are_classified_by_role() {
    let report = run(&["entry-point"]);
    let r = &report.resolution;
    assert_eq!(r.scored, r.resolved + r.unresolved, "the denominator is resolved + unresolved (CG-R-73)");
    assert_eq!(r.composition_edges, r.scored + r.partial + r.boundary + r.registration_not_read + r.excluded, "every edge in exactly one state");
    assert_eq!(r.partial, 2, "Func<IAudit> at AuditSink and IServiceProvider at OrdersEndpoints are the provider ids; IClockFactory is a service under P-5: {:?}", r.by_role);
    assert_eq!(r.by_reason.get("conditional-registration"), Some(&2), "IAudit at OrdersEndpoints and AuditSink: {:?}", r.by_reason);
    assert!(r.by_role.contains_key("generic-dispatch"), "IHandler<T>, IValidator<T>: {:?}", r.by_role);
    // A marker arriving as a constructor parameter is a composition edge the
    // criterion excludes; a type test on it is not a composition edge at all.
    assert_eq!(r.excluded_by_role.get("marker"), Some(&1), "{:?}", r.excluded_by_role);
    // IReadOnlyList<string> is registered (an instance), so it is a single
    // dependency, not collection injection; with arity it reads as generic-dispatch
    // (informational, same resolution path) — no longer a data contract (P-7).
    let routes = edge(&report, "OrdersEndpoints", "IReadOnlyList`1");
    assert_eq!((routes.role.label(), routes.injection), ("generic-dispatch", super::super::csharp_walk::Injection::Single));
    // IEnumerable<IIdGenerator> is collection injection: the edge is to IIdGenerator (P-6).
    let generators = edge(&report, "AuditSink", "IIdGenerator");
    assert_eq!((generators.injection, generators.state), (super::super::csharp_walk::Injection::Collection, super::super::csharp_walk::EdgeState::Resolved));
    assert_eq!(r.collection_edges, 1);
}

#[test]
fn inherited_members_are_counted_over_the_chain() {
    // IAuditStore declares nothing and inherits Load from IReadStore<T>: a service, not a marker (P-1, O-13).
    let report = run(&["entry-point"]);
    let e = edge(&report, "OrdersEndpoints", "IAuditStore");
    assert_eq!(e.role.label(), "service");
    assert_eq!(e.state, super::super::csharp_walk::EdgeState::Resolved);
}

#[test]
fn boundary_and_registration_not_read_are_their_own_states() {
    use super::super::csharp_walk::EdgeState;
    let report = run(&["entry-point", COMPONENT]);
    let r = &report.resolution;
    let logger = r.boundary_surface.iter().find(|b| b.target == "T:Microsoft.Extensions.Logging.ILogger`1").expect("ILogger<T> is boundary");
    assert_eq!(logger.assembly, "Microsoft.Extensions.Logging.Abstractions");
    assert_eq!(logger.edges, 2, "the constructor and the [Inject] property");
    assert_eq!(edge(&report, "OrdersEndpoints", "IServiceProvider").state, EdgeState::Partial, "a provider id is partial wherever it is declared");
    assert_eq!(r.boundary, 2);
    // IMemoryCache is registered by AddMemoryCache(), a call the resolver does not
    // parse: registration-not-read with the call, never boundary (CG-R-75).
    assert_eq!(edge(&report, "OrdersEndpoints", "IMemoryCache").state, EdgeState::RegistrationNotRead("AddMemoryCache"));
    assert_eq!(r.registration_not_read_surface[0].call.as_deref(), Some("AddMemoryCache"));
    assert!(!r.boundary_surface.iter().any(|b| b.target.ends_with("IMemoryCache")));
    // An external abstraction with an in-solution registration is resolved, not boundary.
    assert!(!r.boundary_surface.iter().any(|b| b.target == "T:System.Collections.Generic.IComparer`1"));
    let text = render_reach(&report);
    assert!(text.contains("boundary — the used library surface") && text.contains("via AddMemoryCache"));
}

#[test]
fn the_registration_list_decides_container_construction() {
    // AuditSink is registered and resolved by no edge; PingCheck is registered by a
    // call chained off AddHealthChecks(). Both are container-constructed (O-17, R-2).
    let report = run(&["entry-point"]);
    assert_eq!(edge(&report, "AuditSink", "IAuditStore").state, super::super::csharp_walk::EdgeState::Resolved);
    assert_eq!(edge(&report, "PingCheck", "IClockFactory").state, super::super::csharp_walk::EdgeState::Resolved, "a solution's own factory interface is a service under P-5");
    assert!(!report.unreached.iter().any(|u| u.ends_with("PingCheck")));
}

#[test]
fn property_injection_is_a_composition_edge() {
    let entry = run(&["entry-point"]);
    let with_component = run(&["entry-point", COMPONENT]);
    // OrdersPanel's two [Inject] properties add one resolved edge (ICartReader,
    // through the registration Main reaches) and one boundary edge (ILogger<T>).
    assert_eq!(with_component.resolution.resolved, entry.resolution.resolved + 1);
    assert_eq!(with_component.resolution.boundary, entry.resolution.boundary + 1);
    assert_eq!(entry.resolution.boundary, 1, "ILogger<T> at the constructor");
}

#[test]
fn implements_root_walks_the_inherit_chain() {
    // MemoryAuditStore implements IAuditStore, which extends IReadStore<T> (P-4).
    let report = run(&["implements:T:Shop.Api.Infrastructure.IReadStore`1"]);
    assert_eq!(report.by_root[0].root_symbols, 2, "IAuditStore and MemoryAuditStore");
}

#[test]
fn test_projects_are_outside_the_primary_convention() {
    let report = run(&["entry-point", "implements:T:Shop.Api.Infrastructure.IClock"]);
    assert_eq!(report.test_projects, vec!["P:Shop.Tests".to_string()]);
    assert_eq!(report.test_sites_skipped, 3, "the test's AddSingleton×2 and BuildServiceProvider");
    assert_eq!(report.by_root[1].root_symbols, 1, "SystemClock, not the test project's FakeClock");
    assert_eq!(report.test_project_types.types, 4);
    assert_eq!(report.total.types, 45, "production types only");
    assert!(!report.unreached.iter().any(|u| u.starts_with("T:Shop.Tests.")));
}

#[test]
fn unresolved_and_partial_are_their_own_categories() {
    let report = run(&["entry-point"]);
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Api.Infrastructure.ConsoleAudit"), "conditionally registered: unresolved, not unreached");
    assert!(report.total.unresolved >= 1);
    let t = &report.total;
    assert_eq!(t.types, t.reached + t.unresolved + t.partial + t.unreached);
}

#[test]
fn coverage_prints_its_population() {
    let report = run(&["entry-point"]);
    let text = render_reach(&report);
    let r = &report.resolution;
    assert!(text.contains(&format!("resolution coverage: {}/{} of the denominator", r.resolved, r.scored)));
    assert!(text.contains(&format!("{}/{} of composition edges", r.scored, r.composition_edges)));
    assert!(text.contains("partial: 2 (held:"), "{text}");
    assert!(text.contains(&format!("with registration-not-read inside the denominator (the rule from run 7, CG-R-83): {}/{} (", r.resolved, r.scored + r.registration_not_read)), "{text}");
    assert!(text.contains("registration-knowledge table (CG-R-79): of 16 reached external registration calls the resolver parses 13, the table knows 3, 0 are unknown"), "{text}");
    assert!(text.contains("blind spot (CG-R-78)") && text.contains("incidence:"), "{text}");
}

#[test]
fn ground_truth_gives_reader_recall_and_walk_precision() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let mut o = opts(&["entry-point", COMPONENT], &[]);
    o.ground_truth = Some(serde_yaml::from_str::<GroundTruth>(GROUND_TRUTH).expect("ground truth parses"));
    let report = reach(&inv, &o);
    let g = report.ground_truth.as_ref().expect("measured");
    assert_eq!((g.edges, g.reader_present, g.walk_hits, g.walk_edges), (22, 22, 22, 22), "reader missing {:?} / walk missing {:?} / extra {:?}", g.reader_missing, g.walk_missing, g.walk_extra);
    assert!(render_reach(&report).contains("ground truth (Shop.Api, 22 edges") && render_reach(&report).contains("over C# source, Razor views not covered (CG-R-78)"));
}

#[test]
fn tracked_sets_get_their_own_disposition() {
    let inv = load_inventory(FIXTURE).expect("loads");
    let report = reach(&inv, &opts(&["entry-point"], &["implements:T:Shop.Api.Infrastructure.IHandler`1", "implements:T:Shop.Api.Infrastructure.IAudit"]));
    assert_eq!(report.tracked[0].reached, 1, "the handler");
    assert_eq!(report.tracked[1].unresolved, 1, "the conditionally registered audit (the test project's FakeAudit is not selected)");
    let text = render_reach(&report);
    assert!(text.starts_with("roots: entry-point\n"));
    assert!(text.contains("resolution coverage:") && text.contains("role proxies"));
}

#[test]
fn a_named_member_is_a_root() {
    let report = run(&["member:M:Shop.Api.Program.Main(System.String[])"]);
    let entry = run(&["entry-point"]);
    assert_eq!(report.total.reached, entry.total.reached);
}

#[test]
fn attribute_and_implements_roots_select_declared_symbols() {
    let report = run(&["attribute:T:Shop.Api.Infrastructure.EndpointAttribute", "implements:T:Shop.Api.Infrastructure.IHandler`1"]);
    assert_eq!(report.by_root[0].root_symbols, 1, "one [Endpoint] method");
    assert_eq!(report.by_root[1].root_symbols, 1, "one IHandler implementor");
    assert!(!report.unreached.iter().any(|u| u == "T:Shop.Domain.OrderPlaced"));
}
