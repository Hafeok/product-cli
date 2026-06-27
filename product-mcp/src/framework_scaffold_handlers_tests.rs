//! Tests for the delivery-scaffolding MCP write handlers.

use super::*;
use serde_json::json;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn work_unit_init_scaffolds_and_guards_overwrite() {
    let r = repo();
    let root = r.path();

    let out = handle_work_unit_init(&json!({"id": "complete-task-handler"}), root).expect("init");
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["id"], json!("complete-task-handler"));
    assert!(pdir(root).join("work-unit.yaml").exists());

    // A second init without force is refused; with force it overwrites.
    assert!(handle_work_unit_init(&json!({"id": "x"}), root).is_err());
    assert!(handle_work_unit_init(&json!({"id": "x", "force": true}), root).is_ok());
}

#[test]
fn cell_init_scaffolds_a_task_type() {
    let r = repo();
    let root = r.path();

    let out = handle_cell_init(&json!({"id": "add-crud-resource", "archetype": "rest-api"}), root).expect("init");
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["archetype"], json!("rest-api"));

    let show = crate::framework_read_handlers::handle_cell_show(&json!({}), root).expect("show");
    assert_eq!(show["name"], json!("add-crud-resource"));
}

#[test]
fn cell_dispatch_reports_unbound_required_slots() {
    let r = repo();
    let root = r.path();
    handle_cell_init(&json!({"id": "add-crud-resource", "archetype": "rest-api"}), root).expect("init");

    // The scaffolded cell has a required `entity` slot; dispatching with no
    // bindings must fail with violations rather than write work units.
    let out = handle_cell_dispatch(&json!({}), root).expect("dispatch");
    assert_eq!(out["ok"], json!(false));
    assert!(out["violations"].as_array().map(|a| !a.is_empty()).unwrap_or(false));

    // A malformed binds array is a hard error.
    assert!(handle_cell_dispatch(&json!({"binds": ["noequals"]}), root).is_err());
}

#[test]
fn archetype_init_lays_down_the_skeleton() {
    let r = repo();
    let root = r.path();

    let out = handle_archetype_init(&json!({"name": "rest-api"}), root).expect("init");
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["written"].as_array().map(|a| a.len()), Some(3));

    let dir = pdir(root).join("archetypes").join("rest-api");
    assert!(dir.join("how-contract.yaml").exists());
    assert!(dir.join("layout.yaml").exists());
    assert!(dir.join("cells").join("example-task.yaml").exists());

    // The read handler now sees the new archetype.
    let listed = crate::framework_read_handlers::handle_archetype_list(&json!({}), root).expect("list");
    assert!(listed["archetypes"].as_array().map(|a| a.iter().any(|v| v == "rest-api")).unwrap_or(false));

    // Re-init without force is refused.
    assert!(handle_archetype_init(&json!({"name": "rest-api"}), root).is_err());
}
