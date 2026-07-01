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
fn system_new_accepts_a_sub_kind_via_alias() {
    // BUG 1: the top-level `kind` routes the node type, so a System's §3.2.5
    // sub-kind must come through the `system_kind` alias (CLI `--system-kind`).
    let r = repo();
    let root = r.path();

    // Without a sub-kind the system is non-conformant and (Finding 3) rolls back.
    let bare = handle_domain_new(&json!({"kind":"system","id":"sys-x","label":"X","purpose":"does X"}), root).unwrap();
    assert_eq!(bare["ok"], json!(false));
    assert!(bare["violations"].as_array().unwrap().iter()
        .any(|v| v["message"].as_str().unwrap_or_default().contains("§3.2.5")));
    assert!(handle_domain_show(&json!({"id":"sys-x"}), root).is_err(), "rolled back, no node persisted");

    // With `system_kind` the create is conformant and the sub-kind lands on `kind`.
    let ok = handle_domain_new(
        &json!({"kind":"system","id":"sys-x","label":"X","purpose":"does X","system_kind":"service","references_domain":["Sales"]}),
        root,
    ).unwrap();
    assert_eq!(ok["ok"], json!(true));
    let show = handle_domain_show(&json!({"id":"sys-x"}), root).unwrap();
    assert_eq!(show["node"]["kind"], json!("service"));
    assert_eq!(show["node"]["references_domain"][0], json!("Sales"));

    // A sub-kind passed as the node type is still (correctly) an unknown kind.
    assert!(handle_domain_new(&json!({"kind":"service","id":"sys-y"}), root).is_err());
}

#[test]
fn product_owns_edges_persist_on_new_and_edit() {
    // Finding 2: relation fields persist; owns_domain/owns_system live on the
    // PRODUCT node (§3.0), not the system. Guards against the reported "ghost".
    let r = repo();
    let root = r.path();
    handle_domain_new(&json!({"kind":"context","id":"Sales","label":"Sales"}), root).unwrap();
    handle_domain_new(&json!({"kind":"context","id":"Ops","label":"Ops"}), root).unwrap();
    handle_domain_new(&json!({"kind":"system","id":"sys-x","label":"X","purpose":"does X","system_kind":"service"}), root).unwrap();

    // owns_* set at create resolve and persist.
    let p = handle_domain_new(
        &json!({"kind":"product","id":"prod","label":"Prod","purpose":"the product","owns_domain":["Sales"],"owns_system":["sys-x"]}),
        root,
    ).unwrap();
    assert_eq!(p["ok"], json!(true));
    let show = handle_domain_show(&json!({"id":"prod"}), root).unwrap();
    assert_eq!(show["node"]["owns_domain"], json!(["Sales"]));
    assert_eq!(show["node"]["owns_system"], json!(["sys-x"]));

    // …and an edit of the relation list persists too (not silently dropped).
    let ed = handle_domain_edit(&json!({"id":"prod","owns_domain":["Sales","Ops"]}), root).unwrap();
    assert_eq!(ed["ok"], json!(true));
    assert_eq!(handle_domain_show(&json!({"id":"prod"}), root).unwrap()["node"]["owns_domain"], json!(["Sales","Ops"]));
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
