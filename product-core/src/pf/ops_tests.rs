//! Tests for the in-loop add_* operations.

use super::*;
use crate::pf::model::*;
use crate::pf::session::DomainSession;

fn session() -> DomainSession {
    DomainSession::start("demo", None, vec![], None, "t".into()).expect("start")
}

fn ctx(id: &str) -> BoundedContext {
    BoundedContext { id: id.into(), label: id.into(), ..Default::default() }
}

#[test]
fn happy_path_builds_a_conformant_fragment() {
    let mut s = session();
    assert!(add_bounded_context(&mut s, ctx("Tasks")).ok);
    let e = Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "a unit".into(), is_aggregate_root: true, ..Default::default() };
    assert!(add_entity(&mut s, e).ok);
    let ev = Event { fields: vec![], id: "Done".into(), label: "Done".into(), context: "Tasks".into(), changes: "Task".into() };
    assert!(add_event(&mut s, ev).ok);
    let cmd = Command { fields: vec![], id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Done".into()] };
    assert!(add_command(&mut s, cmd).ok);
    assert_eq!(s.tool_calls, 4);
}

#[test]
fn event_without_real_entity_is_rejected_and_reverted() {
    let mut s = session();
    add_bounded_context(&mut s, ctx("Tasks"));
    let ev = Event { fields: vec![], id: "Ghost".into(), label: "Ghost".into(), context: "Tasks".into(), changes: "Nope".into() };
    let r = add_event(&mut s, ev);
    assert!(!r.ok);
    assert_eq!(r.violations[0].path, "changes");
    // reverted: the bad event is not in the graph
    assert!(!s.graph.contains("Ghost"));
    // tool_calls only counted the context
    assert_eq!(s.tool_calls, 1);
}

#[test]
fn duplicate_id_is_rejected() {
    let mut s = session();
    assert!(add_bounded_context(&mut s, ctx("Tasks")).ok);
    let r = add_bounded_context(&mut s, ctx("Tasks"));
    assert!(!r.ok);
    assert_eq!(r.violations[0].path, "id");
}

#[test]
fn malformed_id_is_rejected() {
    let mut s = session();
    let r = add_bounded_context(&mut s, ctx("1bad"));
    assert!(!r.ok);
    assert_eq!(r.violations[0].path, "id");
}

#[test]
fn invalid_cardinality_is_rejected() {
    let mut s = session();
    let r = add_relation(&mut s, Relation { id: "rel".into(), label: None, from: "A".into(), to: "B".into(), cardinality: "lots".into(), rationale: "x".into() });
    assert!(!r.ok);
    assert_eq!(r.violations[0].path, "cardinality");
}

#[test]
fn command_before_its_event_is_rejected() {
    let mut s = session();
    add_bounded_context(&mut s, ctx("Tasks"));
    add_entity(&mut s, Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "d".into(), ..Default::default() });
    // event does not exist yet
    let r = add_command(&mut s, Command { fields: vec![], id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Done".into()] });
    assert!(!r.ok);
    assert_eq!(r.violations[0].path, "emits");
}
