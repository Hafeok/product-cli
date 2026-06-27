//! Tests for the How-authoring MCP write handlers.

use super::*;
use serde_json::json;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn init_scaffolds_a_contract_and_refuses_to_clobber() {
    let r = repo();
    let root = r.path();

    let init = handle_how_init(&json!({"archetype": "demo-cli"}), root).expect("init");
    assert_eq!(init["ok"], json!(true));
    assert_eq!(init["archetype"], json!("demo-cli"));
    assert!(how_path(root).exists());

    // A second init must not overwrite an existing contract.
    assert!(handle_how_init(&json!({"archetype": "demo-cli"}), root).is_err());
}

#[test]
fn add_builds_the_why_cascade_and_rejects_duplicates() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"archetype": "demo-cli"}), root).expect("init");

    let d = handle_how_add(
        &json!({"element": "decision", "id": "d-lang", "decision": "Use Rust", "rationale": "zero-cost + safety"}),
        root,
    )
    .expect("add decision");
    assert_eq!(d["id"], json!("d-lang"));
    assert_eq!(d["element"]["decision"], json!("Use Rust"));

    handle_how_add(
        &json!({"element": "principle", "id": "zero-unwrap", "statement": "no unwrap() in non-test code"}),
        root,
    )
    .expect("add principle");
    handle_how_add(
        &json!({"element": "pattern", "id": "slice-adapter", "shape": "pure slice + thin adapter"}),
        root,
    )
    .expect("add pattern");

    // Ids are unique across the whole Why cascade.
    assert!(handle_how_add(&json!({"element": "principle", "id": "d-lang", "statement": "x"}), root).is_err());
    // Unknown element kind is rejected.
    assert!(handle_how_add(&json!({"element": "nonsense", "id": "x"}), root).is_err());

    // The read handler now reflects the authored elements.
    let show = crate::framework_read_handlers::handle_how_show(&json!({}), root).expect("show");
    assert_eq!(show["decisions"], json!(1));
    assert_eq!(show["principles"], json!(1));
    assert_eq!(show["patterns"], json!(1));
}

#[test]
fn set_replaces_the_application_contract() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"archetype": "demo-cli"}), root).expect("init");

    let a = handle_how_set(
        &json!({"target": "app-contract", "id": "demo-app", "language": "Rust"}),
        root,
    )
    .expect("set app-contract");
    assert_eq!(a["element"]["language"], json!("Rust"));

    let show = crate::framework_read_handlers::handle_how_show(&json!({}), root).expect("show");
    assert_eq!(show["application_contract"], json!("demo-app"));

    // An unknown singleton target is rejected.
    assert!(handle_how_set(&json!({"target": "mystery", "id": "x"}), root).is_err());
}

#[test]
fn add_requires_an_existing_contract() {
    let r = repo();
    // No init → no contract to add to.
    assert!(handle_how_add(&json!({"element": "principle", "id": "p", "statement": "s"}), r.path()).is_err());
}
