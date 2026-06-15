//! Tests for What-graph context-bundle assembly.

use super::*;
use crate::pf::model::*;

fn sample() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), purpose: Some("orders".into()), glossary: vec![] });
    g.contexts.push(BoundedContext { id: "Billing".into(), label: "Billing".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "a customer order".into(), is_aggregate_root: true, ..Default::default() });
    g.entities.push(Entity { id: "Invoice".into(), label: "Invoice".into(), context: "Billing".into(), definition: "a bill".into(), ..Default::default() });
    g.relations.push(Relation { id: "orderBilled".into(), label: Some("order billed".into()), from: "Order".into(), to: "Invoice".into(), cardinality: "one-to-one".into(), rationale: "one invoice per order".into() });
    g.events.push(Event { id: "OrderPlaced".into(), label: "OrderPlaced".into(), context: "Sales".into(), changes: "Order".into() });
    g.commands.push(Command { id: "PlaceOrder".into(), label: "PlaceOrder".into(), context: "Sales".into(), targets: "Order".into(), emits: vec!["OrderPlaced".into()] });
    g.read_models.push(ReadModel { id: "OpenOrders".into(), label: "OpenOrders".into(), projects: vec!["Order".into()] });
    g
}

#[test]
fn unknown_node_returns_none() {
    assert!(bundle(&sample(), "ghost", 2, "demo").is_none());
}

#[test]
fn depth_one_pulls_direct_neighbours_only() {
    let g = sample();
    let b = bundle(&g, "Order", 1, "demo").expect("bundle");
    // header + focus
    assert!(b.contains("Domain Context Bundle: Order"));
    assert!(b.contains("focus≜Order:Entity"));
    assert!(b.contains("a customer order"));
    // direct neighbours of Order: Sales (context), orderBilled (relation),
    // OrderPlaced (event), PlaceOrder (command), OpenOrders (read model)
    assert!(b.contains("PlaceOrder"));
    assert!(b.contains("OrderPlaced"));
    assert!(b.contains("OpenOrders"));
    assert!(b.contains("Sales"));
    // Invoice is 2 hops away (via orderBilled) — excluded at depth 1
    assert!(!b.contains("a bill"));
}

#[test]
fn depth_two_reaches_across_a_relation() {
    let g = sample();
    let b = bundle(&g, "Order", 2, "demo").expect("bundle");
    // Invoice (and its context Billing) now reachable via orderBilled
    assert!(b.contains("Invoice"));
    assert!(b.contains("a bill"));
}

#[test]
fn context_focus_lists_its_members() {
    let g = sample();
    let b = bundle(&g, "Sales", 1, "demo").expect("bundle");
    assert!(b.contains("focus≜Sales:BoundedContext"));
    // members of Sales reachable at depth 1
    assert!(b.contains("Order"));
    assert!(b.contains("## Entities"));
}
