//! Round-trip tests for Turtle seed parsing.

use super::*;
use crate::pf::model::*;
use crate::pf::turtle::to_turtle;

fn sample() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), purpose: Some("track work".into()), glossary: vec![] });
    g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "a unit of work".into(), identity: Some("id".into()), is_aggregate_root: true, attributes: vec![] });
    g.relations.push(Relation { id: "rel".into(), label: Some("owns".into()), from: "Task".into(), to: "Task".into(), cardinality: "one-to-many".into(), rationale: "self ref".into() });
    g.events.push(Event { id: "Done".into(), label: "Done".into(), context: "Tasks".into(), changes: "Task".into() });
    g.commands.push(Command { id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Done".into()] });
    g.read_models.push(ReadModel { id: "Open".into(), label: "Open".into(), projects: vec!["Task".into(), "Done".into()] });
    g.flows.push(Flow { id: "Flow".into(), label: "Complete a task".into(), steps: vec!["Complete".into(), "Done".into(), "Open".into()], ..Default::default() });
    g
}

#[test]
fn turtle_seed_round_trips() {
    let original = sample();
    let ttl = to_turtle(&original, "demo");
    let parsed = from_turtle(&ttl).expect("parse seed");

    assert_eq!(parsed.contexts.len(), 1);
    assert_eq!(parsed.entities.len(), 1);
    let task = &parsed.entities[0];
    assert_eq!(task.id, "Task");
    assert_eq!(task.context, "Tasks");
    assert!(task.is_aggregate_root);
    assert_eq!(parsed.events[0].changes, "Task");
    assert_eq!(parsed.commands[0].targets, "Task");
    assert_eq!(parsed.commands[0].emits, vec!["Done".to_string()]);
    let mut projects = parsed.read_models[0].projects.clone();
    projects.sort();
    assert_eq!(projects, vec!["Done".to_string(), "Task".to_string()]);
    assert_eq!(parsed.flows[0].steps.len(), 3);
}

#[test]
fn seeded_graph_is_conformant() {
    let ttl = to_turtle(&sample(), "demo");
    let parsed = from_turtle(&ttl).expect("parse seed");
    assert_eq!(crate::pf::validate::validate_graph(&parsed), vec![]);
}

#[test]
fn malformed_turtle_errs() {
    assert!(from_turtle("this is not turtle <<<").is_err());
}
