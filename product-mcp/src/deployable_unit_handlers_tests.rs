//! Handler tests for the `product_deployable_unit_*` MCP tool family.

use super::*;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn new_then_show_then_list_round_trips() {
    let r = repo();
    let root = r.path();
    let out = handle_deployable_unit_new(
        &json!({
            "id": "shop-ios",
            "built_from": "rn-hexagonal-app",
            "deploys_system": ["acme-shop"],
            "environment": "production",
            "bundle_id": "com.acme.shop"
        }),
        root,
    )
    .expect("new");
    assert_eq!(out["ok"], json!(true));

    let shown = handle_deployable_unit_show(&json!({"name": "shop-ios"}), root).expect("show");
    assert_eq!(shown["built_from"], json!("rn-hexagonal-app"));
    assert_eq!(shown["identity"]["bundle_id"], json!("com.acme.shop"));

    let listed = handle_deployable_unit_list(&json!({}), root).expect("list");
    assert_eq!(listed["deployable_units"], json!(["shop-ios"]));
}

#[test]
fn new_rejects_a_unit_without_deployment_identity() {
    let r = repo();
    let out = handle_deployable_unit_new(
        &json!({"id": "bare", "built_from": "bp", "deploys_system": ["sys"]}),
        r.path(),
    )
    .expect("new");
    assert_eq!(out["ok"], json!(false));
    let vs = out["violations"].as_array().expect("violations");
    assert!(vs.iter().any(|v| v["path"] == json!("identity")));
}

#[test]
fn validate_reports_conformance() {
    let r = repo();
    let root = r.path();
    handle_deployable_unit_new(
        &json!({"id": "u", "built_from": "bp", "deploys_system": ["sys"], "runtime": "iOS 17"}),
        root,
    )
    .expect("new");
    let out = handle_deployable_unit_validate(&json!({"name": "u"}), root).expect("validate");
    assert_eq!(out["ok"], json!(true));
}
