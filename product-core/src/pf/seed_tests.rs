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
    g.read_models.push(ReadModel { id: "Open".into(), label: "Open".into(), projects: vec!["Task".into(), "Done".into()], ..Default::default() });
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

#[test]
fn system_round_trips_with_flow_ownership() {
    let mut g = DomainGraph::default();
    g.systems.push(System {
        id: "sys-shop".into(), label: "Acme Shop".into(), kind: "application".into(),
        purpose: "consumer e-commerce".into(), target_platforms: vec!["ios".into(), "web".into()],
        target_classes: vec!["gui".into()], root: Some("root-shop".into()),
    });
    g.flows.push(Flow { id: "checkout".into(), label: "Checkout".into(), steps: vec![], system: Some("sys-shop".into()), ..Default::default() });

    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    let s = &parsed.systems[0];
    assert_eq!(s.id, "sys-shop");
    assert_eq!(s.kind, "application");
    assert_eq!(s.purpose, "consumer e-commerce");
    assert_eq!(s.root.as_deref(), Some("root-shop"));
    let mut platforms = s.target_platforms.clone();
    platforms.sort();
    assert_eq!(platforms, vec!["ios".to_string(), "web".to_string()]);
    assert_eq!(s.target_classes, vec!["gui".to_string()]);
    assert_eq!(parsed.flows[0].system.as_deref(), Some("sys-shop"));
}

#[test]
fn triggers_round_trip() {
    let mut g = DomainGraph::default();
    g.triggers.push(Trigger { id: "t-user".into(), label: "Place".into(), source: "user".into(), issues: "PlaceOrder".into(), ..Default::default() });
    g.triggers.push(Trigger {
        id: "t-auto".into(), label: "Restock".into(), source: "automated".into(),
        issues: "Restock".into(), watches: Some("LowStock".into()), translates_from: Some("sys-wms".into()),
    });
    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    assert_eq!(parsed.triggers.len(), 2);
    let user = parsed.triggers.iter().find(|t| t.id == "t-user").expect("user");
    assert_eq!(user.source, "user");
    assert_eq!(user.issues, "PlaceOrder");
    let auto = parsed.triggers.iter().find(|t| t.id == "t-auto").expect("auto");
    assert_eq!(auto.source, "automated");
    assert_eq!(auto.watches.as_deref(), Some("LowStock"));
    assert_eq!(auto.translates_from.as_deref(), Some("sys-wms"));
}

#[test]
fn unreifiable_rules_round_trip() {
    let mut g = DomainGraph::default();
    g.unreifiable_rules.push(UnreifiableRule {
        id: "u-gallery".into(), aio: "display-collection".into(), class: "tui".into(),
        rationale: Some("no faithful character-grid form".into()),
    });
    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    assert_eq!(parsed.unreifiable_rules.len(), 1);
    let u = &parsed.unreifiable_rules[0];
    assert_eq!(u.aio, "display-collection");
    assert_eq!(u.class, "tui");
    assert_eq!(u.rationale.as_deref(), Some("no faithful character-grid form"));
}
