//! Tests for the `product_dep_*` MCP handlers.

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

fn registry_with_dep() -> (tempfile::TempDir, ToolRegistry) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("product.toml"), "name = \"test\"\n").expect("cfg");
    for d in ["docs/features", "docs/adrs", "docs/tests", "docs/dependencies"] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }
    let dep = "---\nid: DEP-001\ntitle: oxigraph\ntype: library\nstatus: active\nfeatures:\n- FT-001\n---\n\nThe RDF store.\n";
    fs::write(root.join("docs/dependencies/DEP-001-oxigraph.md"), dep).expect("dep");
    let reg = ToolRegistry::new(root.to_path_buf(), false);
    (dir, reg)
}

#[test]
fn dep_list_show_features_via_call_tool() {
    let (_d, reg) = registry_with_dep();
    let listed = reg.call_tool("product_dep_list", &json!({})).expect("list");
    assert!(listed["dependencies"].as_array().unwrap().iter().any(|d| d["id"] == "DEP-001"));
    let shown = reg.call_tool("product_dep_show", &json!({"id": "DEP-001"})).expect("show");
    assert_eq!(shown["title"], json!("oxigraph"));
    let feats = reg.call_tool("product_dep_features", &json!({"id": "DEP-001"})).expect("features");
    assert!(feats["features"].as_array().unwrap().iter().any(|f| f == "FT-001"));
}
