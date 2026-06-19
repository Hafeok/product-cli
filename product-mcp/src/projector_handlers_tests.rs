//! Tests for the `product_projector_*` MCP handlers.

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

/// A repo with a What graph (built via the domain MCP tools): an entity `Order`,
/// an event that changes it, and a read model `rm-orders` projecting it.
fn registry() -> (tempfile::TempDir, ToolRegistry) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("product.toml"), "name = \"test\"\n").expect("cfg");
    for d in ["docs/features", "docs/adrs", "docs/tests"] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }
    let reg = ToolRegistry::new(root.to_path_buf(), true);
    reg.call_tool("product_domain_new", &json!({"kind": "context", "id": "Sales", "label": "Sales"})).expect("ctx");
    reg.call_tool("product_domain_new", &json!({"kind": "entity", "id": "Order", "label": "Order", "context": "Sales", "definition": "an order"})).expect("entity");
    reg.call_tool("product_domain_new", &json!({"kind": "event", "id": "OrderPlaced", "label": "OrderPlaced", "context": "Sales", "changes": "Order"})).expect("event");
    reg.call_tool("product_domain_new", &json!({"kind": "read-model", "id": "rm-orders", "label": "Orders", "projects": ["Order"]})).expect("rm");
    (dir, reg)
}

#[test]
fn derive_then_validate_and_list_via_call_tool() {
    let (_dir, reg) = registry();
    let derived = reg.call_tool("product_projector_derive", &json!({"read_model": "rm-orders"})).expect("derive");
    assert_eq!(derived["ok"], json!(true));
    assert_eq!(derived["id"], json!("rm-orders-projector"));

    let listed = reg.call_tool("product_projector_list", &json!({})).expect("list");
    assert!(listed["projectors"].as_array().expect("arr").iter().any(|v| v == "rm-orders-projector"));

    let validated = reg.call_tool("product_projector_validate", &json!({"name": "rm-orders-projector"})).expect("validate");
    assert_eq!(validated["ok"], json!(true));

    // a freshly derived projector has no logic/scenarios yet → not sound+complete
    let sim = reg.call_tool("product_projector_simulate", &json!({"name": "rm-orders-projector"})).expect("simulate");
    assert_eq!(sim["sound_and_complete"], json!(false));
}
