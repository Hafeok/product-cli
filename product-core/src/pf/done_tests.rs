//! Tests for the §7.2 done + closed predicates.

use super::*;
use crate::pf::decider::Decider;
use crate::pf::decider_logic::*;
use crate::pf::deliverable::AcceptanceCriterion;
use crate::pf::model::*;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "an order".into(), is_aggregate_root: true, ..Default::default() });
    g.events.push(Event { id: "OrderPlaced".into(), label: "OrderPlaced".into(), context: "Sales".into(), changes: "Order".into() });
    g.commands.push(Command { id: "PlaceOrder".into(), label: "PlaceOrder".into(), context: "Sales".into(), targets: "Order".into(), emits: vec!["OrderPlaced".into()] });
    g
}

fn slice() -> Slice {
    Slice { id: "order-slice".into(), anchors: vec!["Order".into()], depth: Some(2) }
}

fn deliverable(status: &str) -> Deliverable {
    Deliverable {
        id: "place-order".into(),
        slice: "order-slice".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "an order can be placed".into(), status: status.into() }],
    }
}

#[test]
fn pending_acceptance_blocks_done() {
    let fd = feature_done(&deliverable("pending"), &slice(), &graph(), &[]);
    assert!(!fd.done);
    assert!(fd.checks.iter().any(|c| c.kind == "acceptance" && !c.passing));
    // domain checks for the in-scope elements pass
    assert!(fd.checks.iter().any(|c| c.kind == "domain" && c.passing));
}

#[test]
fn passing_acceptance_with_conformant_scope_is_done() {
    let fd = feature_done(&deliverable("passing"), &slice(), &graph(), &[]);
    assert!(fd.done, "{:?}", fd.checks);
    assert_eq!(fd.progress(), 1.0);
}

#[test]
fn an_unsound_decider_blocks_done() {
    // a decider over the in-scope Order aggregate that is incomplete (handles a
    // command but has no scenario for it) fails behavioural conformance
    let dec = Decider {
        id: "order-decider".into(),
        decides_for: "Order".into(),
        handles: vec!["PlaceOrder".into()],
        logic: Some(DeciderLogic::default()),
        scenarios: vec![],
        ..Default::default()
    };
    let fd = feature_done(&deliverable("passing"), &slice(), &graph(), std::slice::from_ref(&dec));
    assert!(!fd.done);
    assert!(fd.checks.iter().any(|c| c.kind == "behavioural" && !c.passing));
}

#[test]
fn a_closed_cut_has_no_open_edges() {
    // the whole graph is in scope → every dependency is included
    let scope = covered(&graph(), &["Order".into()], 3);
    assert!(cut_closed(&graph(), &scope).is_empty(), "{:?}", cut_closed(&graph(), &scope));
}

#[test]
fn an_open_cut_is_detected() {
    // scope is just the command; its targets/emits/context are excluded
    let scope = ["PlaceOrder".to_string()].into_iter().collect();
    let open = cut_closed(&graph(), &scope);
    assert!(open.iter().any(|(n, d)| n == "PlaceOrder" && d == "Order"));
}

#[test]
fn release_done_requires_members_done_and_closed() {
    let members = vec![(deliverable("passing"), slice())];
    let rd = release_done("R1", &members, &graph(), &[]);
    // member is done; the slice (depth 2 from Order) covers Sales/OrderPlaced/
    // PlaceOrder → closed
    assert!(rd.done, "members {:?} open {:?}", rd.members, rd.open_edges);
    assert!(rd.closed());
}
