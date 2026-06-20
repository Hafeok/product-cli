//! Unit tests for the conformance mirror.

use super::*;
use crate::pf::model::*;

fn ctx(id: &str) -> BoundedContext {
    BoundedContext { id: id.into(), label: id.into(), ..Default::default() }
}

fn entity(id: &str, ctx: &str) -> Entity {
    Entity { id: id.into(), label: id.into(), context: ctx.into(), definition: "a thing".into(), ..Default::default() }
}

#[test]
fn conformant_what_graph_has_no_violations() {
    let mut g = DomainGraph::default();
    g.contexts.push(ctx("Tasks"));
    g.entities.push(entity("Task", "Tasks"));
    g.events.push(Event { id: "TaskDone".into(), label: "TaskDone".into(), context: "Tasks".into(), changes: "Task".into() });
    g.commands.push(Command { id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["TaskDone".into()] });
    g.read_models.push(ReadModel { id: "Open".into(), label: "Open".into(), projects: vec!["Task".into()], ..Default::default() });
    assert_eq!(validate_graph(&g), vec![]);
}

#[test]
fn event_changing_nothing_is_rejected() {
    let mut g = DomainGraph::default();
    g.contexts.push(ctx("Tasks"));
    g.events.push(Event { id: "Ghost".into(), label: "Ghost".into(), context: "Tasks".into(), changes: "Nope".into() });
    let vs = validate_node(&g, "Ghost");
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].path, "changes");
    assert!(vs[0].message.contains("§3.2"));
}

#[test]
fn entity_without_real_context_is_rejected() {
    let mut g = DomainGraph::default();
    g.entities.push(entity("Task", "MissingCtx"));
    let vs = validate_node(&g, "Task");
    assert!(vs.iter().any(|v| v.path == "inContext"));
}

#[test]
fn entity_without_definition_is_rejected() {
    let mut g = DomainGraph::default();
    g.contexts.push(ctx("Tasks"));
    let mut e = entity("Task", "Tasks");
    e.definition = String::new();
    g.entities.push(e);
    let vs = validate_node(&g, "Task");
    assert!(vs.iter().any(|v| v.path == "definition"));
}

#[test]
fn relation_without_rationale_is_rejected() {
    let mut g = DomainGraph::default();
    g.relations.push(Relation { id: "r".into(), label: None, from: "A".into(), to: "B".into(), cardinality: "one-to-many".into(), rationale: "".into() });
    let vs = validate_node(&g, "r");
    assert!(vs.iter().any(|v| v.path == "rationale"));
}

#[test]
fn command_without_event_is_rejected() {
    let mut g = DomainGraph::default();
    g.contexts.push(ctx("Tasks"));
    g.entities.push(entity("Task", "Tasks"));
    g.commands.push(Command { id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Nope".into()] });
    let vs = validate_node(&g, "Complete");
    assert!(vs.iter().any(|v| v.path == "emits"));
}

#[test]
fn context_mapping_needs_two_sides_and_rationale() {
    let mut g = DomainGraph::default();
    g.context_mappings.push(ContextMapping { id: "m".into(), concept_a: "A".into(), concept_b: "".into(), kind: None, rationale: "".into() });
    let vs = validate_node(&g, "m");
    assert!(vs.iter().any(|v| v.path == "mapsTo"));
    assert!(vs.iter().any(|v| v.path == "rationale"));
}
