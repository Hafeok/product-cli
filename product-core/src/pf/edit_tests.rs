//! Tests for CRUD operations (create/edit/remove via field maps).

use super::*;
use crate::pf::ids::NodeKind;
use crate::pf::session::DomainSession;
use serde_json::json;

fn session() -> DomainSession {
    DomainSession::start("demo", None, vec![], None, "t".into()).expect("start")
}

fn map(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

#[test]
fn create_then_edit_then_remove() {
    let mut s = session();
    assert!(create(&mut s, NodeKind::BoundedContext, "Tasks", &map(json!({ "label": "Tasks" }))).ok);
    let r = create(&mut s, NodeKind::Entity, "Task", &map(json!({ "label": "Task", "context": "Tasks", "definition": "a unit", "is_aggregate_root": true })));
    assert!(r.ok, "{:?}", r.violations);
    assert!(s.graph.entities[0].is_aggregate_root);

    // edit: change the definition + clear aggregate-root
    let e = edit(&mut s, "Task", &map(json!({ "definition": "a piece of work", "is_aggregate_root": false })));
    assert!(e.ok, "{:?}", e.violations);
    assert_eq!(s.graph.entities[0].definition, "a piece of work");
    assert!(!s.graph.entities[0].is_aggregate_root);

    // remove
    assert!(remove(&mut s, "Task").ok);
    assert!(!s.graph.contains("Task"));
}

#[test]
fn create_rejects_non_conformant_fragment() {
    let mut s = session();
    // entity with no context + no definition -> reverted, not committed
    let r = create(&mut s, NodeKind::Entity, "Bad", &map(json!({ "label": "Bad" })));
    assert!(!r.ok);
    assert!(!s.graph.contains("Bad"));
    assert!(r.violations.iter().any(|v| v.path == "definition" || v.path == "inContext"));
}

#[test]
fn create_rejects_duplicate_and_bad_id() {
    let mut s = session();
    create(&mut s, NodeKind::BoundedContext, "Tasks", &map(json!({ "label": "Tasks" })));
    assert!(!create(&mut s, NodeKind::BoundedContext, "Tasks", &map(json!({ "label": "x" }))).ok);
    assert!(!create(&mut s, NodeKind::BoundedContext, "1bad", &map(json!({ "label": "x" }))).ok);
}

#[test]
fn edit_rejecting_change_is_reverted() {
    let mut s = session();
    create(&mut s, NodeKind::BoundedContext, "Tasks", &map(json!({ "label": "Tasks" })));
    create(&mut s, NodeKind::Entity, "Task", &map(json!({ "label": "Task", "context": "Tasks", "definition": "d" })));
    // point the entity at a non-existent context -> rejected + reverted
    let e = edit(&mut s, "Task", &map(json!({ "context": "Ghost" })));
    assert!(!e.ok);
    assert_eq!(s.graph.entities[0].context, "Tasks", "edit must be reverted");
}

#[test]
fn edit_cannot_change_id() {
    let mut s = session();
    create(&mut s, NodeKind::BoundedContext, "Tasks", &map(json!({ "label": "Tasks" })));
    edit(&mut s, "Tasks", &map(json!({ "id": "Renamed", "label": "Renamed" })));
    assert!(s.graph.contains("Tasks"));
    assert!(!s.graph.contains("Renamed"));
    assert_eq!(s.graph.contexts[0].label, "Renamed");
}

#[test]
fn remove_missing_is_rejected() {
    let mut s = session();
    assert!(!remove(&mut s, "nope").ok);
}
