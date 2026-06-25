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
fn system_requires_kind_and_purpose() {
    let mut g = DomainGraph::default();
    g.systems.push(System { id: "sys".into(), label: "Sys".into(), ..Default::default() });
    let vs = validate_node(&g, "sys");
    assert_eq!(vs.len(), 2, "missing kind and purpose: {vs:?}");
    assert!(vs.iter().any(|x| x.path == "kind"));
    assert!(vs.iter().any(|x| x.path == "purpose"));
}

#[test]
fn system_root_must_resolve_and_flow_system_must_resolve() {
    let mut g = DomainGraph::default();
    g.systems.push(System {
        id: "sys".into(), label: "Sys".into(), kind: "cli".into(), purpose: "a tool".into(),
        root: Some("ghost-root".into()), ..Default::default()
    });
    let vs = validate_node(&g, "sys");
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].path, "root");

    // a flow pointing at a real system is fine; a dangling one is a finding.
    g.systems[0].root = None;
    g.flows.push(Flow { id: "f-ok".into(), label: "Ok".into(), system: Some("sys".into()), ..Default::default() });
    g.flows.push(Flow { id: "f-bad".into(), label: "Bad".into(), system: Some("nope".into()), ..Default::default() });
    assert_eq!(validate_node(&g, "f-ok"), vec![]);
    let bad = validate_node(&g, "f-bad");
    assert_eq!(bad.len(), 1);
    assert_eq!(bad[0].path, "system");
}

fn cmd(id: &str, ctx: &str) -> Command {
    Command { id: id.into(), label: id.into(), context: ctx.into(), targets: "Order".into(), emits: vec![] }
}

#[test]
fn trigger_requires_source_and_a_resolvable_command() {
    let mut g = DomainGraph::default();
    g.triggers.push(Trigger { id: "t".into(), label: "T".into(), ..Default::default() });
    let vs = validate_node(&g, "t");
    assert!(vs.iter().any(|x| x.path == "source"), "missing source: {vs:?}");
    assert!(vs.iter().any(|x| x.path == "issues"), "missing/unresolved issues: {vs:?}");

    // A user trigger issuing a real command is conformant.
    g.contexts.push(ctx("Orders"));
    g.commands.push(cmd("Place", "Orders"));
    g.triggers[0] = Trigger { id: "t".into(), label: "T".into(), source: "user".into(), issues: "Place".into(), ..Default::default() };
    assert_eq!(validate_node(&g, "t"), vec![]);
}

#[test]
fn automation_and_translation_patterns_are_checked() {
    let mut g = DomainGraph::default();
    g.contexts.push(ctx("Orders"));
    g.commands.push(cmd("Place", "Orders"));
    // An automated trigger that watches no View is a §3.2.0 finding.
    g.triggers.push(Trigger { id: "t".into(), label: "Auto".into(), source: "automated".into(), issues: "Place".into(), ..Default::default() });
    let vs = validate_node(&g, "t");
    assert!(vs.iter().any(|x| x.path == "watches"), "automation must watch a view: {vs:?}");

    // A Translation reading from an undeclared system is a finding.
    g.read_models.push(ReadModel { id: "Todo".into(), label: "Todo".into(), projects: vec!["Order".into()], ..Default::default() });
    g.triggers[0] = Trigger {
        id: "t".into(), label: "Xlate".into(), source: "automated".into(), issues: "Place".into(),
        watches: Some("Todo".into()), translates_from: Some("ghost-system".into()),
    };
    let vs = validate_node(&g, "t");
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].path, "translates_from");
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
