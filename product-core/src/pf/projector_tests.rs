//! Tests for the Projector (§3.4) — the read-model fold's derived signature.

use super::*;
use crate::pf::model::{DomainGraph, Entity, Event, ReadModel};

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "an order".into(), ..Default::default() });
    g.events.push(Event { id: "OrderPlaced".into(), label: "Order Placed".into(), context: "Sales".into(), changes: "Order".into() });
    g.events.push(Event { id: "OrderShipped".into(), label: "Order Shipped".into(), context: "Sales".into(), changes: "Order".into() });
    g.events.push(Event { id: "UserRenamed".into(), label: "User Renamed".into(), context: "Iam".into(), changes: "User".into() });
    g.read_models.push(ReadModel { id: "rm-orders".into(), label: "Orders".into(), projects: vec!["Order".into()] });
    g
}

#[test]
fn derives_the_fold_signature_from_projected_entities() {
    let p = derive_projector(&graph(), "rm-orders").expect("derive");
    assert_eq!(p.projects_for, "rm-orders");
    assert!(p.folds.contains(&"OrderPlaced".to_string()) && p.folds.contains(&"OrderShipped".to_string()));
    assert!(!p.folds.contains(&"UserRenamed".to_string()), "only events changing a projected entity");
    assert_eq!(p.over, vec!["Order".to_string()]);
}

#[test]
fn a_derived_projector_is_conformant() {
    let g = graph();
    let p = derive_projector(&g, "rm-orders").expect("derive");
    assert!(validate_projector(&p, &g).is_empty(), "{:?}", validate_projector(&p, &g));
}

#[test]
fn omitting_a_fed_event_fails_coverage() {
    let g = graph();
    let p = Projector { id: "rm-orders-projector".into(), projects_for: "rm-orders".into(), folds: vec!["OrderPlaced".into()], over: vec!["Order".into()], ..Default::default() };
    let vs = validate_projector(&p, &g);
    assert!(vs.iter().any(|x| x.message.contains("OrderShipped") && x.message.contains("does not fold it")), "{:?}", vs);
}

#[test]
fn folding_a_foreign_event_fails_no_foreign() {
    let g = graph();
    let p = Projector { id: "rm-orders-projector".into(), projects_for: "rm-orders".into(), folds: vec!["OrderPlaced".into(), "OrderShipped".into(), "UserRenamed".into()], over: vec!["Order".into()], ..Default::default() };
    let vs = validate_projector(&p, &g);
    assert!(vs.iter().any(|x| x.message.contains("UserRenamed") && x.message.contains("not fed by")), "{:?}", vs);
}

#[test]
fn unknown_read_model_is_rejected() {
    assert!(derive_projector(&graph(), "rm-ghost").is_err());
}
