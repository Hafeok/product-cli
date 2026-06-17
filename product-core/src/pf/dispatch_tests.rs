//! Tests for cell dispatch (task type → frozen SPMC work units).

use super::*;
use crate::pf::cell::TaskType;
use crate::pf::model::*;
use crate::pf::work_unit_validate::validate_work_unit;

const EXAMPLE: &str = include_str!("../../../schema/examples/task-type-definition.example.yaml");

fn graph_with_order() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "an order".into(), ..Default::default() });
    g
}

fn bind(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn dispatch_instantiates_frozen_work_units() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    // the example's required slots: entity, fields, operations, validation, views
    let bindings = bind(&[("entity", "Order"), ("fields", "title,total"), ("operations", "CRUD"), ("validation", "non-empty"), ("views", "list")]);
    let d = dispatch(&t, &bindings, Some(&graph_with_order()));
    assert!(!d.violations.iter().any(|x| x.severity == "violation"), "{:?}", d.violations);
    assert_eq!(d.work_units.len(), t.cells.len());
    for wu in &d.work_units {
        assert!(wu.context.frozen);
        assert!(wu.context.hash.is_some());
        // the entity slot resolved to the bound concrete entity
        // (the contract cell derives from domain:entity → domain:Order)
    }
    // the 'contract' cell's domain:entity became domain:Order
    let contract = d.work_units.iter().find(|w| w.id.starts_with("contract-")).expect("contract cell");
    assert!(contract.context.derived_from.iter().any(|p| p == "domain:Order"), "{:?}", contract.context.derived_from);
    assert_eq!(contract.trace.as_ref().unwrap().what.as_deref(), Some("Order"));
}

#[test]
fn dispatched_work_units_are_valid() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    let bindings = bind(&[("entity", "Order"), ("fields", "f"), ("operations", "R"), ("validation", "v"), ("views", "l")]);
    let d = dispatch(&t, &bindings, Some(&graph_with_order()));
    for wu in &d.work_units {
        let vs = validate_work_unit(wu, Some(&graph_with_order()), None);
        assert!(!vs.iter().any(|x| x.severity == "violation"), "wu {} invalid: {:?}", wu.id, vs);
    }
}

#[test]
fn binding_to_unknown_entity_is_rejected() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    let bindings = bind(&[("entity", "Ghost"), ("fields", "f"), ("operations", "R"), ("validation", "v"), ("views", "l")]);
    let d = dispatch(&t, &bindings, Some(&graph_with_order()));
    assert!(d.violations.iter().any(|x| x.severity == "violation" && x.message.contains("Ghost")));
    assert!(d.work_units.is_empty(), "must not instantiate against invalid bindings");
}

#[test]
fn missing_required_binding_is_rejected() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    let d = dispatch(&t, &bind(&[("entity", "Order")]), Some(&graph_with_order()));
    assert!(d.violations.iter().any(|x| x.message.contains("required slot")));
}

#[test]
fn edits_cell_becomes_an_edit_work_unit() {
    use crate::pf::cell::{Cell, Slot};
    let t = TaskType {
        id: "wire".into(),
        name: "wire".into(),
        applies_when: "x".into(),
        slots: vec![Slot {
            name: "entity".into(),
            kind: Some("domain".into()),
            dispatch: "name it".into(),
            capture: "which?".into(),
            audit: "exists".into(),
            required: true,
        }],
        cells: vec![Cell {
            id: "wire-mod".into(),
            artifact: "add `pub mod casing;` in sorted order".into(),
            model: Some("code".into()),
            derived_from: vec!["domain:entity".into()],
            applies: vec![],
            edits: Some("src/pf/mod.rs".into()),
        }],
        ..Default::default()
    };
    let d = dispatch(&t, &bind(&[("entity", "Order")]), Some(&graph_with_order()));
    assert!(!d.violations.iter().any(|x| x.severity == "violation"), "{:?}", d.violations);
    let wu = &d.work_units[0];
    assert_eq!(wu.produces.path_hint.as_deref(), Some("src/pf/mod.rs"));
    assert!(wu.prompt.starts_with("Edit the existing file 'src/pf/mod.rs'"), "{}", wu.prompt);
}

#[test]
fn binding_unknown_slot_is_rejected() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    let bindings = bind(&[("entity", "Order"), ("fields", "f"), ("operations", "R"), ("validation", "v"), ("views", "l"), ("nope", "x")]);
    let d = dispatch(&t, &bindings, Some(&graph_with_order()));
    assert!(d.violations.iter().any(|x| x.message.contains("names no declared slot")));
}
