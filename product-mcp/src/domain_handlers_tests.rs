//! Tests for the domain (What) graph MCP handlers.

use super::*;
use serde_json::json;

const CONFIG: &str = r#"name = "demo"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
"#;

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), CONFIG).expect("write config");
    dir
}

#[test]
fn full_crud_through_the_handlers() {
    let r = repo();
    let root = r.path();
    // create context + entity (product defaults to config name "demo")
    assert_eq!(handle_domain_new(&json!({"kind":"context","id":"Sales","label":"Sales"}), root).unwrap()["ok"], json!(true));
    let e = handle_domain_new(&json!({"kind":"entity","id":"Order","label":"Order","context":"Sales","definition":"an order","is_aggregate_root":true}), root).unwrap();
    assert_eq!(e["ok"], json!(true));

    // list shows them
    let list = handle_domain_list(&json!({}), root).unwrap();
    assert_eq!(list["count"], json!(2));
    let entities = handle_domain_list(&json!({"kind":"entity"}), root).unwrap();
    assert_eq!(entities["nodes"][0]["id"], json!("Order"));

    // show + validate
    let show = handle_domain_show(&json!({"id":"Order"}), root).unwrap();
    assert_eq!(show["node"]["definition"], json!("an order"));
    assert_eq!(handle_domain_validate(&json!({}), root).unwrap()["conformant"], json!(true));

    // edit
    let ed = handle_domain_edit(&json!({"id":"Order","definition":"a confirmed order"}), root).unwrap();
    assert_eq!(ed["ok"], json!(true));
    assert_eq!(handle_domain_show(&json!({"id":"Order"}), root).unwrap()["node"]["definition"], json!("a confirmed order"));

    // export + context
    assert!(handle_domain_export(&json!({}), root).unwrap()["turtle"].as_str().unwrap().contains("d:Order a pf:Entity"));
    assert!(handle_domain_context(&json!({"id":"Order","depth":1}), root).unwrap()["bundle"].as_str().unwrap().contains("Domain Context Bundle: Order"));

    // rm
    assert_eq!(handle_domain_rm(&json!({"id":"Order"}), root).unwrap()["ok"], json!(true));
}

#[test]
fn new_rejects_non_conformant_fragment() {
    let r = repo();
    handle_domain_new(&json!({"kind":"context","id":"Sales","label":"Sales"}), r.path()).unwrap();
    let bad = handle_domain_new(&json!({"kind":"event","id":"Ghost","label":"Ghost","context":"Sales","changes":"Nope"}), r.path()).unwrap();
    assert_eq!(bad["ok"], json!(false));
    assert!(bad["violations"][0]["message"].as_str().unwrap().contains("§3.2"));
}

#[test]
fn read_without_a_graph_is_a_clear_error() {
    let r = repo();
    let err = handle_domain_list(&json!({}), r.path()).unwrap_err();
    assert!(err.contains("no domain graph"));
}

#[test]
fn parity_with_cli_via_call_tool() {
    // the tools are reachable through the registry by their product_domain_* names
    let r = repo();
    let reg = crate::ToolRegistry::new(r.path().to_path_buf(), true);
    let names: Vec<String> = reg.tool_list().iter().map(|t| t.name.clone()).collect();
    for t in ["product_domain_list","product_domain_show","product_domain_validate","product_domain_export","product_domain_context","product_domain_new","product_domain_edit","product_domain_rm"] {
        assert!(names.contains(&t.to_string()), "missing tool {t}");
    }
    reg.call_tool("product_domain_new", &json!({"kind":"context","id":"C","label":"C"})).expect("create via call_tool");
    let listed = reg.call_tool("product_domain_list", &json!({})).expect("list via call_tool");
    assert_eq!(listed["count"], json!(1));
}
