//! Tests for SPMC build-context assembly.

use super::*;
use crate::pf::deliverable::{AcceptanceCriterion, Deliverable};
use crate::pf::model::*;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "an order".into(), ..Default::default() });
    g.commands.push(Command { id: "PlaceOrder".into(), label: "PlaceOrder".into(), context: "Sales".into(), targets: "Order".into(), emits: vec!["OrderPlaced".into()] });
    g
}

#[test]
fn assembles_the_spmc_sections() {
    let slice = Slice { id: "order-slice".into(), anchors: vec!["Order".into()], depth: Some(2) };
    let d = Deliverable {
        id: "place-order".into(),
        slice: "order-slice".into(),
        acceptance: vec![AcceptanceCriterion { id: "a1".into(), statement: "an order can be placed".into(), status: "pending".into(), runner: None, runner_args: None }],
    };
    let ctx = assemble(&d, &slice, &graph(), None, &[], "demo");
    assert!(ctx.contains("Build Context: place-order"));
    assert!(ctx.contains("## What"));
    assert!(ctx.contains("PlaceOrder"));      // the slice subgraph is included
    assert!(ctx.contains("## How"));
    assert!(ctx.contains("## Behaviour"));
    assert!(ctx.contains("## Acceptance"));
    assert!(ctx.contains("a1: an order can be placed"));
}
