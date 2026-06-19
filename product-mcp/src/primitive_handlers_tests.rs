//! Tests for the `product_primitive_*` MCP handlers.

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

/// A repo with one authored named-algorithm primitive under .product/primitives/.
fn registry() -> (tempfile::TempDir, ToolRegistry) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("product.toml"), "name = \"test\"\n").expect("cfg");
    for d in ["docs/features", "docs/adrs", "docs/tests"] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }
    let pdir = root.join(".product").join("primitives");
    fs::create_dir_all(&pdir).expect("mkdir");
    fs::write(
        pdir.join("betweenness.yaml"),
        "id: betweenness\nreference: \"Brandes' betweenness centrality\"\ninput: a graph\noutput: a score per node\noracle:\n- input: path a-b-c\n  output: b=1.0\n",
    ).expect("prim");
    let reg = ToolRegistry::new(root.to_path_buf(), true);
    (dir, reg)
}

#[test]
fn list_show_validate_via_call_tool() {
    let (_dir, reg) = registry();
    let listed = reg.call_tool("product_primitive_list", &json!({})).expect("list");
    assert!(listed["primitives"].as_array().expect("arr").iter().any(|v| v == "betweenness"));

    let shown = reg.call_tool("product_primitive_show", &json!({"name": "betweenness"})).expect("show");
    assert_eq!(shown["reference"], json!("Brandes' betweenness centrality"));

    let validated = reg.call_tool("product_primitive_validate", &json!({"name": "betweenness"})).expect("validate");
    assert_eq!(validated["ok"], json!(true));
}
