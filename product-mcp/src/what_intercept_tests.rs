//! The What interceptor loop across every interception mode.

use std::path::Path;

use serde_json::{json, Value};

use crate::registry::ToolRegistry;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

/// A repo with a `.ddd/` store in `mode` and an empty What graph for `acme`.
fn repo(mode: &str) -> (tempfile::TempDir, ToolRegistry) {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "product.toml", "name = \"acme\"\n");
    write(tmp.path(), ".ddd/config.yaml", &format!("format: 3\nintercept: {mode}\nignore: []\n"));
    std::fs::create_dir_all(tmp.path().join(".ddd/seams")).expect("mkdir");
    let registry = ToolRegistry::new(tmp.path().to_path_buf(), true);
    (tmp, registry)
}

/// A repo with no governance store at all.
fn ungoverned() -> (tempfile::TempDir, ToolRegistry) {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "product.toml", "name = \"acme\"\n");
    let registry = ToolRegistry::new(tmp.path().to_path_buf(), true);
    (tmp, registry)
}

fn add_system(registry: &ToolRegistry) -> Result<Value, String> {
    registry.call_tool("product_domain_new", &json!({
        "kind": "system", "id": "payments-api", "label": "Payments",
        "system_kind": "service", "purpose": "take payments for the shop",
    }))
}

fn graph_text(root: &Path) -> String {
    std::fs::read_to_string(root.join(".product/products/acme/acme.ttl")).unwrap_or_default()
}

#[test]
fn an_undeclared_boundary_is_rejected_and_leaves_no_trace() {
    let (tmp, registry) = repo("enforce");
    let out = add_system(&registry).expect("call");
    assert_eq!(out["status"], "rejected", "{out}");
    assert_eq!(out["ok"], false);
    let demand = &out["demands"][0];
    assert_eq!(demand["surface"]["kind"], "system");
    assert_eq!(demand["surface"]["element"], "payments-api");
    assert_eq!(demand["surface"]["change"], "added");
    assert_eq!(demand["surface"]["rule"], "pf-add-system");
    // The judgment is never pre-filled.
    assert_eq!(demand["template"]["verdict_knowledge"], "");
    // The rollback is total: the graph never records the system.
    assert!(!graph_text(tmp.path()).contains("payments-api"), "{}", graph_text(tmp.path()));
}

#[test]
fn declaring_the_seam_lets_the_same_edit_through() {
    let (tmp, registry) = repo("enforce");
    assert_eq!(add_system(&registry).expect("call")["status"], "rejected");

    let filed = registry
        .call_tool("product_what_declare", &json!({
            "id": "seam/what/payments-api",
            "element": "payments-api",
            "boundary": "the payments service boundary",
            "verdict_knowledge": "callers learn whether a payment is authorized",
        }))
        .expect("declare");
    assert_eq!(filed["status"], "filed", "{filed}");
    assert!(tmp.path().join(".ddd/seams/seam-what-payments-api.yaml").is_file());

    let applied = add_system(&registry).expect("call");
    assert_eq!(applied["ok"], true, "{applied}");
    assert!(applied.get("status").is_none(), "{applied}");
    assert!(graph_text(tmp.path()).contains("payments-api"));
}

#[test]
fn internal_elaboration_is_never_intercepted() {
    let (tmp, registry) = repo("enforce");
    let out = registry
        .call_tool("product_domain_new", &json!({
            "kind": "context", "id": "billing", "label": "Billing",
            "purpose": "money movement",
        }))
        .expect("call");
    assert_ne!(out["status"], "rejected", "{out}");
    assert!(graph_text(tmp.path()).contains("billing"));
}

#[test]
fn warn_mode_keeps_the_write_and_says_what_was_formed() {
    let (tmp, registry) = repo("warn");
    let out = add_system(&registry).expect("call");
    assert_ne!(out["status"], "rejected", "{out}");
    assert_eq!(out["mode"], "warn");
    assert!(out["warning"].as_str().expect("warning").contains("product_what_declare"), "{out}");
    assert_eq!(out["surface"][0]["surface"]["element"], "payments-api");
    assert!(graph_text(tmp.path()).contains("payments-api"));
}

#[test]
fn off_mode_and_an_ungoverned_repo_both_pass_straight_through() {
    let (tmp, registry) = repo("off");
    assert_ne!(add_system(&registry).expect("call")["status"], "rejected");
    assert!(graph_text(tmp.path()).contains("payments-api"));

    let (tmp2, registry2) = ungoverned();
    let out = add_system(&registry2).expect("call");
    assert_ne!(out["status"], "rejected", "{out}");
    assert!(graph_text(tmp2.path()).contains("payments-api"));
}

#[test]
fn declare_refuses_a_silent_overwrite_but_amends_on_request() {
    let (_tmp, registry) = repo("enforce");
    let args = json!({
        "id": "seam/what/payments-api", "element": "payments-api",
        "boundary": "b", "verdict_knowledge": "",
    });
    let filed = registry.call_tool("product_what_declare", &args).expect("declare");
    assert_eq!(filed["status"], "filed");
    assert!(filed["warning"].is_string(), "empty verdict_knowledge must warn: {filed}");

    let clash = registry.call_tool("product_what_declare", &args);
    assert!(clash.expect_err("must refuse").contains("amend: true"));

    let amended = registry
        .call_tool("product_what_declare", &json!({
            "id": "seam/what/payments-api", "amend": true,
            "verdict_knowledge": "callers learn whether a payment is authorized",
        }))
        .expect("amend");
    assert_eq!(amended["status"], "amended", "{amended}");
    assert!(amended["warning"].is_null());
    // The element it is about is identity, preserved without restating it.
    assert_eq!(amended["element"], "what:payments-api");
}

#[test]
fn the_declare_tool_is_advertised_on_the_registry() {
    let (_tmp, registry) = repo("enforce");
    assert!(registry.tool_list().iter().any(|t| t.name == "product_what_declare"));
}

/// The qualifier case: adding a §3.2.0 Translation publishes elements that
/// were already in the graph, so the demand names *them*, not the trigger.
#[test]
fn wiring_a_translation_demands_declarations_for_what_it_publishes() {
    let (tmp, registry) = repo("enforce");
    write(
        tmp.path(),
        ".product/products/acme/session.json",
        r#"{"product":"acme","graph":{
             "contexts":[{"id":"ordering","label":"Ordering","purpose":"orders"}],
             "systems":[{"id":"acme-shop","label":"Shop","kind":"website","purpose":"sell things"}],
             "events":[{"id":"OrderPlaced","label":"Placed","context":"ordering","changes":"Order"}],
             "commands":[{"id":"IssueRefund","label":"Refund","context":"ordering","targets":"Order","emits":[]}],
             "read_models":[{"id":"OrderConfirmation","label":"Confirmation","projects":["OrderPlaced"]}]},
           "started_at":"2026-08-07T00:00:00Z"}"#,
    );
    // Nothing is surface yet: no crossing carries any of it.
    let out = registry
        .call_tool("product_domain_new", &json!({
            "kind": "trigger", "id": "trg-fulfil", "label": "Translation in",
            "source": "automated", "issues": "IssueRefund",
            "watches": "OrderConfirmation", "translates_from": "acme-shop",
        }))
        .expect("call");
    assert_eq!(out["status"], "rejected", "{out}");
    let named: Vec<&str> = out["demands"]
        .as_array()
        .expect("demands")
        .iter()
        .filter_map(|d| d["surface"]["element"].as_str())
        .collect();
    for expect in ["OrderConfirmation", "OrderPlaced", "IssueRefund"] {
        assert!(named.contains(&expect), "{expect} missing from {named:?}");
    }
    // They were already in the graph, so the change is publication.
    let d = out["demands"][0].clone();
    assert_eq!(d["surface"]["change"], "published", "{d}");
    assert_eq!(d["surface"]["visibility"], "published");
    // And the trigger itself never landed.
    assert!(!graph_text(tmp.path()).contains("trg-fulfil"));
}
