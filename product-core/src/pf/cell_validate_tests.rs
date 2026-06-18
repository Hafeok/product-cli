//! Tests for the task-type (cell) checker + cross-validation.

use super::*;
use crate::pf::cell::*;
use crate::pf::how::HowContract;
use crate::pf::model::*;
use crate::pf::validate::Violation;

const EXAMPLE: &str = include_str!("../../../schema/examples/task-type-definition.example.yaml");
const HOW: &str = include_str!("../../../schema/examples/how-contract.example.yaml");

fn blocking(vs: &[Violation]) -> usize {
    vs.iter().filter(|v| v.severity == "violation").count()
}

#[test]
fn bundled_example_has_no_blocking_violations() {
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    // Without a What graph, domain-slot pointers still resolve (entity/fields…
    // are declared slots), so there are no blocking violations.
    let vs = validate_cell(&t, None, None);
    assert_eq!(blocking(&vs), 0, "unexpected blocking: {vs:?}");
}

#[test]
fn slot_without_inline_audit_is_a_violation() {
    let mut t = TaskType::from_yaml(EXAMPLE).expect("parse");
    t.slots.push(Slot {
        name: "orphan-slot".into(),
        kind: Some("domain".into()),
        dispatch: "x".into(),
        capture: "x".into(),
        audit: "".into(), // no backing audit
        required: true,
    });
    let vs = validate_cell(&t, None, None);
    assert!(vs.iter().any(|v| v.severity == "violation" && v.message.contains("orphan-slot")));
}

#[test]
fn slot_not_covered_by_top_level_audit_is_a_warning() {
    // The bundled example's `entity` + `validation` slots have inline audits
    // but no top-level audit naming them — a soft coverage warning.
    let t = TaskType::from_yaml(EXAMPLE).expect("parse");
    let vs = validate_cell(&t, None, None);
    assert!(vs.iter().any(|v| v.severity == "warning" && v.message.contains("entity")));
    assert_eq!(blocking(&vs), 0);
}

#[test]
fn empty_slots_or_audits_are_violations() {
    let mut t = TaskType::from_yaml(EXAMPLE).expect("parse");
    t.slots.clear();
    t.audits.clear();
    let vs = validate_cell(&t, None, None);
    assert!(vs.iter().any(|v| v.path == "slots"));
    assert!(vs.iter().any(|v| v.path == "audits"));
}

#[test]
fn concrete_domain_pointer_warns_when_absent_from_graph() {
    let mut t = TaskType::from_yaml(EXAMPLE).expect("parse");
    // a concrete domain pointer not present as a slot or in the graph
    t.cells[0].derived_from.push("domain:NoSuchEntity".into());
    let empty = DomainGraph::default();
    let vs = validate_cell(&t, Some(&empty), None);
    assert!(vs.iter().any(|v| v.severity == "warning" && v.message.contains("NoSuchEntity")));
}

#[test]
fn concrete_domain_pointer_resolves_against_the_what_graph() {
    let mut t = TaskType::from_yaml(EXAMPLE).expect("parse");
    t.cells[0].derived_from.push("domain:Order".into());
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Sales".into(), label: "Sales".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "d".into(), ..Default::default() });
    let vs = validate_cell(&t, Some(&g), None);
    assert!(!vs.iter().any(|x| x.message.contains("Order")), "Order should resolve: {vs:?}");
}

#[test]
fn applies_resolves_against_how_contract() {
    let mut t = TaskType::from_yaml(EXAMPLE).expect("parse");
    // result-type cell applies result-type pattern (exists in HOW)
    t.cells[1].applies = vec!["result-type".into(), "ghost-pattern".into()];
    let how = HowContract::from_yaml(HOW).expect("how");
    let vs = validate_cell(&t, None, Some(&how));
    assert!(!vs.iter().any(|x| x.message.contains("result-type'")), "known pattern must not warn");
    assert!(vs.iter().any(|x| x.message.contains("ghost-pattern")), "unknown pattern must warn");
}
