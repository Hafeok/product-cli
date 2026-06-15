//! Full-session tests driving the domain registry via `call_tool`.

use super::*;
use serde_json::json;

fn reg() -> (tempfile::TempDir, DomainRegistry) {
    let dir = tempfile::tempdir().expect("tempdir");
    let reg = DomainRegistry::new(dir.path().to_path_buf());
    (dir, reg)
}

#[test]
fn full_session_reaches_a_conformant_finalize() {
    let (_dir, reg) = reg();
    reg.call_tool("session_start", &json!({ "product": "demo", "title": "Demo", "participants": ["PO", "UX"] })).expect("start");

    assert!(reg.call_tool("add_bounded_context", &json!({ "id": "Tasks", "label": "Tasks" })).expect("ctx")["ok"].as_bool().unwrap());
    assert!(reg.call_tool("add_entity", &json!({ "id": "Task", "label": "Task", "context": "Tasks", "definition": "a unit of work", "is_aggregate_root": true })).expect("ent")["ok"].as_bool().unwrap());
    assert!(reg.call_tool("add_event", &json!({ "id": "TaskDone", "label": "TaskDone", "context": "Tasks", "changes": "Task" })).expect("ev")["ok"].as_bool().unwrap());
    assert!(reg.call_tool("add_command", &json!({ "id": "Complete", "label": "Complete", "context": "Tasks", "targets": "Task", "emits": ["TaskDone"] })).expect("cmd")["ok"].as_bool().unwrap());
    assert!(reg.call_tool("add_read_model", &json!({ "id": "OpenTasks", "label": "OpenTasks", "projects": ["Task"] })).expect("rm")["ok"].as_bool().unwrap());

    let state = reg.call_tool("session_state", &json!({})).expect("state");
    assert_eq!(state["conformant"], json!(true));
    assert_eq!(state["counts"]["Entity"], json!(1));

    let fin = reg.call_tool("session_finalize", &json!({})).expect("finalize");
    assert_eq!(fin["ok"], json!(true));
    assert!(fin["turtle"].as_str().unwrap().contains("d:Task a pf:Entity"));
    assert_eq!(fin["provenance"]["participants"], json!(["PO", "UX"]));
    assert!(fin["provenance"]["content_hash"].as_str().unwrap().len() == 64);
    // The exported files exist on disk.
    let ttl_path = fin["turtlePath"].as_str().unwrap();
    assert!(std::path::Path::new(ttl_path).exists());
}

#[test]
fn in_loop_rejection_returns_framework_message() {
    let (_dir, reg) = reg();
    reg.call_tool("session_start", &json!({ "product": "demo" })).expect("start");
    reg.call_tool("add_bounded_context", &json!({ "id": "Tasks", "label": "Tasks" })).expect("ctx");
    // event changing a non-existent entity is rejected in-loop
    let r = reg.call_tool("add_event", &json!({ "id": "Ghost", "label": "Ghost", "context": "Tasks", "changes": "Nope" })).expect("ev");
    assert_eq!(r["ok"], json!(false));
    assert!(r["violations"][0]["message"].as_str().unwrap().contains("§3.2"));
}

#[test]
fn finalize_blocks_on_non_conformant_seed() {
    let (_dir, reg) = reg();
    // Seed a graph whose event changes a node that is not a real entity. The
    // in-loop tools can't produce this, but a prior session's Turtle might.
    let bad_seed = "\
@prefix pf: <https://productframework.org/ns#> .\n\
@prefix d: <https://productframework.org/product/demo#> .\n\
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
d:Tasks a pf:BoundedContext ; rdfs:label \"Tasks\" .\n\
d:Ghost a pf:Event ; rdfs:label \"Ghost\" ; pf:inContext d:Tasks ; pf:changes d:Nope .\n";
    reg.call_tool("session_start", &json!({ "product": "demo", "seed_graph": bad_seed })).expect("start");
    let fin = reg.call_tool("session_finalize", &json!({})).expect("finalize");
    assert_eq!(fin["ok"], json!(false), "non-conformant seed must block finalize: {fin}");
    assert!(fin["turtle"].is_null());
    assert!(fin["violations"][0]["message"].as_str().unwrap().contains("§3.2"));
}

#[test]
fn open_questions_surface_gaps() {
    let (_dir, reg) = reg();
    reg.call_tool("session_start", &json!({ "product": "demo" })).expect("start");
    reg.call_tool("add_bounded_context", &json!({ "id": "Billing", "label": "Billing" })).expect("ctx");
    let q = reg.call_tool("open_questions", &json!({ "focus": "structure" })).expect("oq");
    let arr = q["openQuestions"].as_array().expect("array");
    assert!(arr.iter().any(|q| q["question"].as_str().unwrap().contains("no entities")));
}

#[test]
fn query_what_happens_to_works_after_build() {
    let (_dir, reg) = reg();
    reg.call_tool("session_start", &json!({ "product": "demo" })).expect("start");
    reg.call_tool("add_bounded_context", &json!({ "id": "Tasks", "label": "Tasks" })).expect("ctx");
    reg.call_tool("add_entity", &json!({ "id": "Task", "label": "Task", "context": "Tasks", "definition": "d" })).expect("ent");
    reg.call_tool("add_event", &json!({ "id": "Done", "label": "Done", "context": "Tasks", "changes": "Task" })).expect("ev");
    let r = reg.call_tool("query", &json!({ "about": "Task", "question": "whatHappensTo" })).expect("query");
    assert_eq!(r["changedByEvents"][0], json!("Done"));
}

#[test]
fn calling_before_start_is_a_clear_error() {
    let (_dir, reg) = reg();
    let err = reg.call_tool("add_entity", &json!({ "id": "X", "label": "X", "context": "C", "definition": "d" })).unwrap_err();
    assert!(err.contains("session_start"));
}

#[test]
fn missing_required_arg_is_reported() {
    let (_dir, reg) = reg();
    reg.call_tool("session_start", &json!({ "product": "demo" })).expect("start");
    let err = reg.call_tool("add_entity", &json!({ "id": "X", "label": "X" })).unwrap_err();
    assert!(err.contains("context") || err.contains("definition"));
}
