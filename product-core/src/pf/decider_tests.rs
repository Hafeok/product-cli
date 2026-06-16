//! Tests for Decider derivation + signature conformance against the event model.

use super::*;
use crate::pf::model::*;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), ..Default::default() });
    g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "a task".into(), is_aggregate_root: true, ..Default::default() });
    g.events.push(Event { id: "TaskCompleted".into(), label: "completed".into(), context: "Tasks".into(), changes: "Task".into() });
    g.events.push(Event { id: "TaskReopened".into(), label: "reopened".into(), context: "Tasks".into(), changes: "Task".into() });
    g.commands.push(Command { id: "CompleteTask".into(), label: "complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["TaskCompleted".into()] });
    g.commands.push(Command { id: "ReopenTask".into(), label: "reopen".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["TaskReopened".into()] });
    g.invariants.push(Invariant { id: "not-deleted".into(), statement: "a completed task is not deleted".into(), context: Some("Tasks".into()), applies_to: Some("Task".into()) });
    g
}

#[test]
fn derive_builds_the_full_signature() {
    let d = derive_decider(&graph(), "Task").expect("derive");
    assert_eq!(d.decides_for, "Task");
    assert_eq!(d.handles, vec!["CompleteTask", "ReopenTask"]);
    assert_eq!(d.emits, vec!["TaskCompleted", "TaskReopened"]);
    assert_eq!(d.evolves_from, vec!["TaskCompleted", "TaskReopened"]);
    assert_eq!(d.rejects, vec!["not-deleted"]);
}

#[test]
fn derive_unknown_aggregate_errs() {
    assert!(derive_decider(&graph(), "Ghost").is_err());
}

#[test]
fn derived_decider_is_conformant() {
    let g = graph();
    let d = derive_decider(&g, "Task").expect("derive");
    let vs = validate_decider(&d, &g);
    assert!(vs.is_empty(), "{vs:?}");
}

#[test]
fn foreign_command_is_a_violation() {
    let g = graph();
    let mut d = derive_decider(&g, "Task").expect("derive");
    d.handles.push("SomeForeignCmd".into());
    let vs = validate_decider(&d, &g);
    assert!(vs.iter().any(|v| v.path == "handles" && v.message.contains("SomeForeignCmd")), "{vs:?}");
}

#[test]
fn missing_command_coverage_is_a_violation() {
    let g = graph();
    let mut d = derive_decider(&g, "Task").expect("derive");
    d.handles.retain(|c| c != "ReopenTask");
    d.emits.retain(|e| e != "TaskReopened");
    let vs = validate_decider(&d, &g);
    assert!(vs.iter().any(|v| v.path == "handles" && v.message.contains("ReopenTask")), "{vs:?}");
}

#[test]
fn emitting_an_unsanctioned_event_is_a_violation() {
    let g = graph();
    let mut d = derive_decider(&g, "Task").expect("derive");
    d.emits.push("GhostEvent".into());
    let vs = validate_decider(&d, &g);
    assert!(vs.iter().any(|v| v.path == "emits" && v.message.contains("GhostEvent")), "{vs:?}");
}

#[test]
fn yaml_round_trips() {
    let d = derive_decider(&graph(), "Task").expect("derive");
    let yaml = d.to_yaml().expect("to_yaml");
    assert_eq!(Decider::from_yaml(&yaml).expect("from_yaml"), d);
}
