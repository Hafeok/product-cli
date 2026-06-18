//! Tests for the `product_decider_*` MCP handlers.

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

/// A repo with product.toml + the legacy dirs, plus a What graph (built via the
/// domain MCP tools) containing an aggregate `Order` with a command.
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
    reg.call_tool("product_domain_new", &json!({"kind": "command", "id": "PlaceOrder", "label": "PlaceOrder", "context": "Sales", "targets": "Order", "emits": ["OrderPlaced"]})).expect("cmd");
    (dir, reg)
}

#[test]
fn derive_then_validate_and_list_via_call_tool() {
    let (_dir, reg) = registry();
    let derived = reg.call_tool("product_decider_derive", &json!({"aggregate": "Order"})).expect("derive");
    assert_eq!(derived["ok"], json!(true));
    assert_eq!(derived["id"], json!("Order-decider"));

    let listed = reg.call_tool("product_decider_list", &json!({})).expect("list");
    assert!(listed["deciders"].as_array().expect("arr").iter().any(|v| v == "Order-decider"));

    let validated = reg.call_tool("product_decider_validate", &json!({"name": "Order-decider"})).expect("validate");
    assert_eq!(validated["ok"], json!(true));

    // a freshly derived decider has no logic yet → not sound+complete
    let sim = reg.call_tool("product_decider_simulate", &json!({"name": "Order-decider"})).expect("simulate");
    assert_eq!(sim["sound_and_complete"], json!(false));
}
