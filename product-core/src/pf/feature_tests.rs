//! Tests for delivery-feature validation + context assembly.

use super::*;
use crate::pf::model::*;

fn sample() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "a customer order".into(), is_aggregate_root: true, ..Default::default() });
    g.events.push(Event { fields: vec![], id: "OrderPlaced".into(), label: "OrderPlaced".into(), context: "Sales".into(), changes: "Order".into() });
    g.commands.push(Command { fields: vec![], id: "PlaceOrder".into(), label: "PlaceOrder".into(), context: "Sales".into(), targets: "Order".into(), emits: vec!["OrderPlaced".into()] });
    g.flows.push(Flow { id: "PlaceOrderFlow".into(), label: "place an order".into(), steps: vec!["PlaceOrder".into(), "OrderPlaced".into()], ..Default::default() });
    g
}

#[test]
fn yaml_round_trips() {
    let s = Feature { id: "place-order".into(), anchors: vec!["PlaceOrderFlow".into()], depth: Some(3) };
    assert_eq!(Feature::from_yaml(&s.to_yaml().expect("to")).expect("from"), s);
}

#[test]
fn an_empty_anchor_list_is_a_violation() {
    let s = Feature { id: "x".into(), anchors: vec![], depth: None };
    assert!(validate_feature(&s, &sample()).iter().any(|v| v.path == "anchors"));
}

#[test]
fn a_dangling_anchor_is_a_violation() {
    let s = Feature { id: "x".into(), anchors: vec!["Ghost".into()], depth: None };
    assert!(validate_feature(&s, &sample()).iter().any(|v| v.path == "anchors" && v.message.contains("Ghost")));
}

#[test]
fn a_resolving_feature_passes() {
    let s = Feature { id: "po".into(), anchors: vec!["PlaceOrderFlow".into()], depth: None };
    assert!(validate_feature(&s, &sample()).is_empty());
}

#[test]
fn context_assembles_the_flow_closure() {
    let s = Feature { id: "po".into(), anchors: vec!["PlaceOrderFlow".into()], depth: None };
    let bundle = feature_context(&s, &sample(), s.depth(), "demo").expect("bundle");
    // the flow is the focus; its steps + their entities/contexts are pulled in
    assert!(bundle.contains("PlaceOrderFlow"));
    assert!(bundle.contains("PlaceOrder"));
    assert!(bundle.contains("OrderPlaced"));
    assert!(bundle.contains("a customer order")); // Order's definition, reached via PlaceOrder
}

#[test]
fn multiple_anchors_union_into_one_bundle() {
    let s = Feature { id: "two".into(), anchors: vec!["Sales".into(), "Order".into()], depth: Some(1) };
    let bundle = feature_context(&s, &sample(), s.depth(), "demo").expect("bundle");
    assert!(bundle.contains("focus≜Sales:BoundedContext, Order:Entity"));
}
