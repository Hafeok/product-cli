//! Tests for the delivery MCP handlers (feature / deliverable / release).

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

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
fn delivery_chain_and_done_via_call_tool() {
    let (_dir, reg) = registry();

    // feature → context assembles the subgraph
    let s = reg.call_tool("product_feature_new", &json!({"id": "order-feature", "anchors": ["Order"]})).expect("feature new");
    assert_eq!(s["ok"], json!(true));
    let ctx = reg.call_tool("product_feature_context", &json!({"name": "order-feature"})).expect("context");
    assert!(ctx["bundle"].as_str().expect("bundle").contains("PlaceOrder"));

    // deliverable → pending acceptance is not done
    reg.call_tool("product_deliverable_new", &json!({"id": "place-order", "feature": "order-feature", "acceptance": ["a1:an order can be placed"]})).expect("deliv new");
    let nd = reg.call_tool("product_deliverable_done", &json!({"name": "place-order"})).expect("done");
    assert_eq!(nd["done"], json!(false));
    // record acceptance → done
    reg.call_tool("product_deliverable_accept", &json!({"id": "place-order", "criterion": "a1", "status": "passing"})).expect("accept");
    let d = reg.call_tool("product_deliverable_done", &json!({"name": "place-order"})).expect("done2");
    assert_eq!(d["done"], json!(true));

    // release → done requires members done + closed cut
    reg.call_tool("product_release_new", &json!({"id": "R1", "features": ["place-order"]})).expect("release new");
    let rd = reg.call_tool("product_release_done", &json!({"name": "R1"})).expect("release done");
    assert_eq!(rd["done"], json!(true));
    assert_eq!(rd["closed"], json!(true));
}

#[test]
fn feature_new_rejects_a_dangling_anchor() {
    let (_dir, reg) = registry();
    let s = reg.call_tool("product_feature_new", &json!({"id": "bad", "anchors": ["Ghost"]})).expect("feature new");
    assert_eq!(s["ok"], json!(false));
}

#[test]
fn write_tools_are_listed_and_gated() {
    let (_dir, _reg) = registry();
    // every new tool name resolves in the registry's tool list
    let names = ToolRegistry::new(_dir.path().to_path_buf(), false);
    for t in ["product_feature_new", "product_deliverable_new", "product_release_new", "product_decider_derive"] {
        assert!(names.tool_list().iter().any(|d| d.name == t), "missing {t}");
    }
    // a write tool is refused when write is disabled
    let err = names.call_tool("product_feature_new", &json!({"id": "x", "anchors": ["Order"]}));
    assert!(err.is_err());
}
