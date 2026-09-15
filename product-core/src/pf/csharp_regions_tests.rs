//! Tests for the regions delta: the vocabulary from accepted rows, the four regions, the split with its bound.

use super::super::csharp_candidates::tests::inventory;
use super::{regions, Ratification, Region};

fn ratification(yaml: &str) -> Ratification {
    serde_yaml::from_str(yaml).expect("ratification")
}

const ACCEPT_GET: &str = "vocabulary_grade: stand-in (test)\ncandidates:\n  - {id: 'controller-action:OrdersController@Web.Get#GET', decision: accept, act: PlaceOrder, channel: web}\n  - {id: 'controller-action:OrdersController@Web.Post#POST', decision: reject, reject_reason: 'renders'}\n  - {id: 'health-check:MapHealthChecks#GET (x)', decision: reject, reject_reason: 'probe'}\n";

#[test]
fn the_vocabulary_is_the_accepted_rows_with_proxy_positions() {
    let r = regions(&inventory(), &ratification(ACCEPT_GET));
    assert_eq!(r.grade, "stand-in (test)");
    assert_eq!(r.acts.len(), 1);
    let a = &r.acts[0];
    assert_eq!((a.name.as_str(), a.entry_points.len(), a.channels.as_slice()), ("PlaceOrder", 1, &["web".to_string()][..]));
    assert_eq!(a.reads, vec!["T:Web.Order"]);
    assert_eq!(a.writes, vec!["T:Web.Order"]);
    assert_eq!(r.rows_without_path, vec!["health-check:MapHealthChecks#GET (x)"]);
}

#[test]
fn regions_separate_declared_declarable_and_no_facts() {
    let r = regions(&inventory(), &ratification(ACCEPT_GET));
    let by = |id: &str| r.entries.iter().find(|e| e.id == id).expect(id);
    let get = by("controller-action:OrdersController@Web.Get#GET");
    assert_eq!((get.region, get.disposition.as_str(), get.acts.as_slice()), (Region::Declared, "accept", &["PlaceOrder".to_string()][..]));
    let post = by("controller-action:OrdersController@Web.Post#POST");
    assert_eq!((post.region, post.disposition.as_str()), (Region::Declarable, "reject"));
    assert_eq!(post.acts, vec!["PlaceOrder"], "the rejected POST's path is covered by the declared act — declarable for it");
    let checkout = by("razor-page-handler:CheckoutModel@Basket.OnGet#GET");
    assert_eq!((checkout.region, checkout.disposition.as_str()), (Region::NoFactsUnderProxy, "not in ratification"));
    assert_eq!(r.by_region["declared"], 1);
    assert_eq!(r.by_region["declarable"], 1);
    assert_eq!(r.by_region["no-facts-under-proxy"], 6);
    assert!(r.by_region.get("unstructured").is_none());
    assert!(r.spanning_types.is_empty(), "every type with facts is covered by PlaceOrder");
    assert!(r.separator_incidence.starts_with("0 spanning of 1 path types with facts"), "{}", r.separator_incidence);
}

#[test]
fn an_uncovered_fact_makes_a_type_spanning_and_its_entry_points_unstructured() {
    // Accept the endpoint as an act with no facts: OrderService's Order is then covered by nothing.
    let rat = ratification("candidates:\n  - {id: 'fastendpoints:CreateOrderEndpoint@Web.(type)#POST', decision: accept, act: CreateOrder}\n");
    let r = regions(&inventory(), &rat);
    assert_eq!(r.spanning_types.len(), 1);
    assert_eq!(r.spanning_types[0].type_id, "T:Web.OrderService");
    let get = r.entries.iter().find(|e| e.id.ends_with("Get#GET")).expect("Get");
    assert_eq!(get.region, Region::Unstructured);
    assert_eq!(get.spanning_types, vec!["T:Web.OrderService"]);
    assert_eq!(get.disposition, "not in ratification");
}

#[test]
fn the_split_walks_from_accepted_roots_with_its_bound() {
    let r = regions(&inventory(), &ratification(ACCEPT_GET));
    let x = &r.ratios;
    assert_eq!((x.accepted_entry_points, x.roots), (1, 2), "the handler plus the constructor");
    assert_eq!(x.undeclared_types, 10, "eleven production types minus the accepted entry point's own");
    assert_eq!(x.reachable_undeclared, 3, "IOrderService, OrderService and the IRepository abstraction its constructor names: {:?}", x.by_namespace);
    assert_eq!(x.unresolved_undeclared, 0);
    assert_eq!(x.isolated_undeclared, 7);
    assert_eq!(x.composition_edges, 2, "IOrderService on the controller, IRepository<Order> on the service");
    assert_eq!(x.scored + x.unscored, x.composition_edges);
    assert_eq!((x.unresolved_edges, x.not_read_edges), (1, 0), "IRepository<Order> has no registration: unfollowed, inside the CG-R-120 bound");
    assert_eq!(x.unfollowed, 1);
    assert!((x.error_bound_percent - 50.0).abs() < 1e-9, "1 unfollowed of 2 edges");
    assert_eq!(x.unscored, 0, "the prior form counts nothing here");
    assert!(r.twelve_one.bound_smaller_than_distance);
    assert!(r.twelve_one.statement.contains("discriminates"));
    assert!(r.labels.iter().any(|l| l.contains("stand-in")));
}
