//! MCP handlers for the product homes.
//!
//! Mirrors the CLI's `product product {list,show,new}`: every product lives at
//! `.product/products/<id>/` (its What graph beside its How/Delivery
//! artifacts); legacy `author-domain` graphs are still listed and readable.

use std::path::Path;

use product_core::author::domain::session_dir;
use product_core::guide::FrameworkState;
use product_core::pf::paths::{list_products, product_home};
use product_core::pf::session::DomainSession;
use serde_json::{json, Value};

/// `product_product_list` — every product, with its home and node count.
pub fn handle_product_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    let products: Vec<Value> = list_products(repo_root)
        .into_iter()
        .map(|name| summary(&name, repo_root))
        .collect();
    Ok(json!({ "products": products }))
}

/// `product_product_show` — one product's home and framework state.
pub fn handle_product_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_id(args)?;
    if !list_products(repo_root).iter().any(|n| n == &id) {
        return Err(format!("no product '{id}' — list them with product_product_list"));
    }
    let s = FrameworkState::probe(repo_root, &id);
    let mut out = summary(&id, repo_root);
    if let Some(o) = out.as_object_mut() {
        o.insert("violations".into(), json!(s.violations));
        o.insert("hasHow".into(), json!(s.has_how));
        o.insert("deciders".into(), json!(s.deciders));
        o.insert("projectors".into(), json!(s.projectors));
        o.insert("features".into(), json!(s.features));
        o.insert("deliverables".into(), json!(s.deliverables));
        o.insert("releases".into(), json!(s.releases));
    }
    Ok(out)
}

/// `product_product_new` — create the home with an empty What graph.
pub fn handle_product_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_id(args)?;
    product_core::pf::ids::validate_id(&id).map_err(|e| format!("{e}"))?;
    if DomainSession::load(&session_dir(repo_root, &id)).is_ok() {
        return Err(format!("product '{id}' already exists — inspect it with product_product_show"));
    }
    let title = args.get("title").and_then(|v| v.as_str()).map(str::to_string);
    let session = DomainSession::start(&id, title, vec![], None, chrono::Utc::now().to_rfc3339())
        .map_err(|e| format!("{e}"))?;
    let home = product_home(repo_root, &id);
    session.save(&home).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": id, "home": home.display().to_string() }))
}

/// One product's list row: name, home (repo-relative), node count, legacy flag.
fn summary(name: &str, repo_root: &Path) -> Value {
    let dir = session_dir(repo_root, name);
    let nodes = DomainSession::load(&dir).map(|s| s.graph.node_count()).unwrap_or(0);
    let home = dir.strip_prefix(repo_root).unwrap_or(&dir).display().to_string();
    json!({
        "id": name,
        "home": home,
        "nodes": nodes,
        "legacy": home.contains("author-domain"),
    })
}

fn req_id(args: &Value) -> Result<String, String> {
    args.get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "missing required argument: id".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn new_list_show_via_call_tool() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");
        let reg = crate::registry::ToolRegistry::new(root.to_path_buf(), true);

        let made = reg.call_tool("product_product_new", &json!({"id": "acme", "title": "Acme"})).expect("new");
        assert_eq!(made["ok"], json!(true));
        assert!(root.join(".product/products/acme/acme.ttl").exists(), "home carries the What spec");
        assert!(reg.call_tool("product_product_new", &json!({"id": "acme"})).is_err(), "duplicate refused");

        let listed = reg.call_tool("product_product_list", &json!({})).expect("list");
        let ids: Vec<_> = listed["products"].as_array().expect("arr").iter().map(|p| p["id"].clone()).collect();
        assert!(ids.contains(&json!("acme")) && ids.contains(&json!("demo")), "ids: {ids:?}");

        let shown = reg.call_tool("product_product_show", &json!({"id": "acme"})).expect("show");
        assert_eq!(shown["nodes"], json!(0));
        assert_eq!(shown["legacy"], json!(false));
        assert!(reg.call_tool("product_product_show", &json!({"id": "ghost"})).is_err());
    }
}
