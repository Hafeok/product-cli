//! Tests for the framework read handlers (blueprint / cell / how / work-unit).

use serde_json::json;
use std::fs;

use crate::registry::ToolRegistry;

const HOW: &str = include_str!("../../schema/examples/how-contract.example.yaml");
const LAYOUT: &str = include_str!("../../schema/examples/layout-model.example.yaml");
const CELL: &str = include_str!("../../schema/examples/task-type-definition.example.yaml");
const WORK_UNIT: &str = include_str!("../../schema/examples/work-unit.example.yaml");

fn registry() -> (tempfile::TempDir, ToolRegistry) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("product.toml"), "name = \"test\"\n").expect("cfg");
    for d in ["docs/features", "docs/adrs", "docs/tests"] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }
    let p = root.join(".product");
    fs::create_dir_all(&p).expect("mkdir .product");
    fs::write(p.join("how-contract.yaml"), HOW).expect("how");
    fs::write(p.join("cell.yaml"), CELL).expect("cell");
    fs::write(p.join("work-unit.yaml"), WORK_UNIT).expect("wu");
    let arch = p.join("blueprints/example-rest-api");
    fs::create_dir_all(arch.join("cells")).expect("mkdir arch");
    fs::write(arch.join("how-contract.yaml"), HOW).expect("a how");
    fs::write(arch.join("layout.yaml"), LAYOUT).expect("a layout");
    fs::write(arch.join("cells/add-crud-resource.yaml"), CELL).expect("a cell");
    let reg = ToolRegistry::new(root.to_path_buf(), false);
    (dir, reg)
}

#[test]
fn how_show_validate_export() {
    let (_d, reg) = registry();
    assert!(reg.call_tool("product_how_show", &json!({})).expect("show")["principles"].as_u64().unwrap() >= 1);
    assert_eq!(reg.call_tool("product_how_validate", &json!({})).expect("validate")["ok"], json!(true));
    assert!(reg.call_tool("product_how_export", &json!({})).expect("export")["turtle"].as_str().unwrap().contains("pf:"));
}

#[test]
fn blueprint_list_show_validate() {
    let (_d, reg) = registry();
    let listed = reg.call_tool("product_blueprint_list", &json!({})).expect("list");
    assert!(listed["blueprints"].as_array().unwrap().iter().any(|v| v == "example-rest-api"));
    let shown = reg.call_tool("product_blueprint_show", &json!({"name": "example-rest-api"})).expect("show");
    assert_eq!(shown["how"], json!(true));
    // validate returns a verdict (warnings allowed; no blocking)
    assert!(reg.call_tool("product_blueprint_validate", &json!({"name": "example-rest-api"})).expect("validate").get("ok").is_some());
}

#[test]
fn worker_list_and_resolve() {
    let (d, reg) = registry();
    let p = d.path().join(".product");
    fs::write(p.join("capabilities.yaml"), "capabilities:\n- id: claude-code\n  endpoint: claude\n  model_identifier: claude-opus-4-8\n  tier: 2\n- id: deep-reasoning\n  endpoint: litellm\n  model_identifier: anthropic/claude-opus\n  tier: 3\n").expect("caps");
    fs::write(p.join("role-bindings.yaml"), "role_bindings:\n- role_id: implementer\n  default_capability: claude-code\n  escalation_steps:\n  - capability: deep-reasoning\n    triggers:\n    - stakes_foundational\n  active: true\n").expect("bindings");
    assert!(reg.call_tool("product_worker_list", &json!({})).expect("list")["capabilities"].as_array().unwrap().len() >= 2);
    let def = reg.call_tool("product_worker_resolve", &json!({"role": "implementer"})).expect("resolve");
    assert_eq!(def["id"], json!("claude-code"));
    let esc = reg.call_tool("product_worker_resolve", &json!({"role": "implementer", "triggers": ["stakes_foundational"]})).expect("resolve2");
    assert_eq!(esc["id"], json!("deep-reasoning"));
}

#[test]
fn cell_and_work_unit_read() {
    let (_d, reg) = registry();
    assert!(reg.call_tool("product_cell_show", &json!({})).expect("cell show").get("name").is_some());
    assert!(reg.call_tool("product_cell_validate", &json!({})).expect("cell validate").get("ok").is_some());
    assert!(reg.call_tool("product_work_unit_show", &json!({})).expect("wu show").get("id").is_some());
    assert!(reg.call_tool("product_work_unit_validate", &json!({})).expect("wu validate").get("ok").is_some());
}
