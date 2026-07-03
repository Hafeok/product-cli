//! Tests for the How-authoring MCP write handlers.

use super::*;
use crate::pf_mcp::pdir;
use serde_json::json;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn init_scaffolds_a_contract_and_refuses_to_clobber() {
    let r = repo();
    let root = r.path();

    let init = handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");
    assert_eq!(init["ok"], json!(true));
    assert_eq!(init["blueprint"], json!("demo-cli"));
    assert!(how_path(&pdir(root)).exists());

    // A second init must not overwrite an existing contract.
    assert!(handle_how_init(&json!({"blueprint": "demo-cli"}), root).is_err());
}

#[test]
fn add_builds_the_why_cascade_and_rejects_duplicates() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");

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
    handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");

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
fn set_carries_the_section_7_3_versions() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");

    // §7.3 — `id` carries the version string, mirroring the CLI's `--id`.
    let v = handle_how_set(&json!({"target": "version", "id": "1.4.0"}), root).expect("set version");
    assert_eq!(v["element"]["version"], json!("1.4.0"));
    let rv = handle_how_set(&json!({"target": "realises-version", "id": "3.2.0"}), root)
        .expect("set realises-version");
    assert_eq!(rv["element"]["realisesVersion"], json!("3.2.0"));

    // Both land on the persisted contract (CLI↔MCP parity for `product how set`).
    let c = load_how(&pdir(root)).expect("reload");
    assert_eq!(c.version.as_deref(), Some("1.4.0"));
    assert_eq!(c.realises_version.as_deref(), Some("3.2.0"));
}

#[test]
fn add_requires_an_existing_contract() {
    let r = repo();
    // No init → no contract to add to.
    assert!(handle_how_add(&json!({"element": "principle", "id": "p", "statement": "s"}), r.path()).is_err());
}

#[test]
fn edit_patches_fields_and_keeps_the_rest() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");
    handle_how_add(
        &json!({"element": "decision", "id": "d-lang", "decision": "Use Rust", "rationale": "safety"}),
        root,
    )
    .expect("add");

    // Patch only the rationale; the decision text must survive.
    let out = handle_how_edit(
        &json!({"element": "decision", "id": "d-lang", "rationale": "zero-cost + safety"}),
        root,
    )
    .expect("edit");
    assert_eq!(out["element"]["decision"], json!("Use Rust"));
    assert_eq!(out["element"]["rationale"], json!("zero-cost + safety"));

    // Editing a missing id is an error.
    assert!(handle_how_edit(&json!({"element": "decision", "id": "nope", "rationale": "x"}), root).is_err());
}

#[test]
fn rm_removes_by_id() {
    let r = repo();
    let root = r.path();
    handle_how_init(&json!({"blueprint": "demo-cli"}), root).expect("init");
    handle_how_add(&json!({"element": "pattern", "id": "p-slice", "shape": "pure slice"}), root).expect("add");

    let out = handle_how_rm(&json!({"id": "p-slice"}), root).expect("rm");
    assert_eq!(out["removed"], json!("pattern"));
    let show = crate::framework_read_handlers::handle_how_show(&json!({}), root).expect("show");
    assert_eq!(show["patterns"], json!(0));

    // Removing an unknown id is an error.
    assert!(handle_how_rm(&json!({"id": "ghost"}), root).is_err());
}
