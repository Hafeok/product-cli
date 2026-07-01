//! MCP handlers for the domain (What) graph — parity with `product domain`.
//!
//! Each `product_domain_*` tool mirrors a `product domain` subcommand, calling
//! the same `product_core::pf` functions the CLI adapter uses, against the
//! persisted session under `.product/author-domain/<product>/`. Mutating tools
//! return the `{ ok, node, violations[] }` contract.

use std::path::{Path, PathBuf};

use product_core::author::domain::session_dir;
use product_core::config::ProductConfig;
use product_core::pf::ids::NodeKind;
use product_core::pf::session::DomainSession;
use product_core::pf::{bundle, edit, ops, query, turtle, validate};
use serde_json::{json, Map, Value};

/// The product whose What graph to operate on: the `product` arg, else the
/// repo's configured `name`.
fn product_of(args: &Value, repo_root: &Path) -> Result<String, String> {
    if let Some(p) = args.get("product").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
        return Ok(p.to_string());
    }
    let cfg = ProductConfig::load_from_root(repo_root).map_err(|e| format!("{}", e))?;
    let name = cfg.name.trim();
    if name.is_empty() {
        Err("no product — pass `product` or set `name` in product.toml".to_string())
    } else {
        Ok(name.to_string())
    }
}

fn dir_of(args: &Value, repo_root: &Path) -> Result<(String, PathBuf), String> {
    let p = product_of(args, repo_root)?;
    let dir = session_dir(repo_root, &p);
    Ok((p, dir))
}

/// Load the active session, or a clear error telling the caller to create one.
fn load(dir: &Path) -> Result<DomainSession, String> {
    DomainSession::load(dir).map_err(|_| {
        "no domain graph for this product yet — create one with `product_domain_new` or `product author domain`".to_string()
    })
}

fn req_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing required argument '{}'", key))
}

/// The argument object minus the routing keys, used as a node field map, with
/// the CLI-parity `kind` aliases normalized onto the struct field.
fn field_map(args: &Value, drop: &[&str]) -> Map<String, Value> {
    let mut m = args.as_object().cloned().unwrap_or_default();
    for k in drop {
        m.remove(*k);
    }
    normalize_kind_aliases(&mut m);
    m
}

/// Map the surface aliases `system_kind` / `mapping_kind` onto the struct field
/// `kind`, which the top-level `kind` node-type router shadows (a caller cannot
/// pass a raw `kind` for a System sub-kind — it is consumed as the router and
/// dropped). Mirrors product-cli `NodeFields::to_map` (`--system-kind` /
/// `--mapping-kind` both write the field-map key `kind`). Without this a system
/// cannot be created conformant via MCP: §3.2.5 requires the sub-kind, and it
/// has no other way in.
fn normalize_kind_aliases(m: &mut Map<String, Value>) {
    for alias in ["system_kind", "mapping_kind"] {
        if let Some(v) = m.remove(alias) {
            m.entry("kind".to_string()).or_insert(v);
        }
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn op_value(result: ops::OpResult) -> Result<Value, String> {
    serde_json::to_value(result).map_err(|e| format!("serialize result: {}", e))
}

// --- read tools ----------------------------------------------------------

pub fn handle_domain_list(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, dir) = dir_of(args, repo_root)?;
    let session = load(&dir)?;
    let filter = match args.get("kind").and_then(|v| v.as_str()) {
        Some(k) => Some(NodeKind::parse(k).map_err(|e| format!("{}", e))?),
        None => None,
    };
    let nodes: Vec<Value> = session.graph.ids().into_iter()
        .filter(|(_, k)| filter.is_none_or(|f| f == *k))
        .map(|(id, kind)| {
            let label = query::node_value(&session.graph, &id)
                .and_then(|v| v.get("label").and_then(|l| l.as_str()).map(str::to_string))
                .unwrap_or_default();
            json!({ "id": id, "kind": kind.cli_name(), "label": label })
        })
        .collect();
    Ok(json!({ "nodes": nodes, "count": nodes.len() }))
}

pub fn handle_domain_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, dir) = dir_of(args, repo_root)?;
    let session = load(&dir)?;
    let id = req_str(args, "id")?;
    let node = query::node_value(&session.graph, &id)
        .ok_or_else(|| format!("no node with id {:?} in the graph", id))?;
    let links = query::describe(&session.graph, &id).map_err(|e| format!("{}", e))?;
    Ok(json!({ "node": node, "links": links }))
}

pub fn handle_domain_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, dir) = dir_of(args, repo_root)?;
    Ok(load(&dir)?.validate_json())
}

pub fn handle_domain_export(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (product, dir) = dir_of(args, repo_root)?;
    let session = load(&dir)?;
    Ok(json!({ "turtle": turtle::to_turtle(&session.graph, &product) }))
}

pub fn handle_domain_context(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (product, dir) = dir_of(args, repo_root)?;
    let session = load(&dir)?;
    let id = req_str(args, "id")?;
    let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
    let bundle = bundle::bundle(&session.graph, &id, depth, &product)
        .ok_or_else(|| format!("no node with id {:?} in the graph", id))?;
    Ok(json!({ "bundle": bundle }))
}

// --- write tools ---------------------------------------------------------

pub fn handle_domain_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (product, dir) = dir_of(args, repo_root)?;
    let kind = NodeKind::parse(&req_str(args, "kind")?).map_err(|e| format!("{}", e))?;
    let id = req_str(args, "id")?;
    let mut session = DomainSession::load(&dir)
        .unwrap_or(DomainSession::start(&product, None, vec![], None, now()).map_err(|e| format!("{}", e))?);
    let result = edit::create(&mut session, kind, &id, &field_map(args, &["kind", "id", "product"]));
    session.save(&dir).map_err(|e| format!("{}", e))?;
    op_value(result)
}

pub fn handle_domain_edit(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, dir) = dir_of(args, repo_root)?;
    let mut session = load(&dir)?;
    let id = req_str(args, "id")?;
    let result = edit::edit(&mut session, &id, &field_map(args, &["id", "product"]));
    session.save(&dir).map_err(|e| format!("{}", e))?;
    op_value(result)
}

pub fn handle_domain_rm(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, dir) = dir_of(args, repo_root)?;
    let mut session = load(&dir)?;
    let id = req_str(args, "id")?;
    let result = edit::remove(&mut session, &id);
    session.save(&dir).map_err(|e| format!("{}", e))?;
    let dangling = validate::validate_graph(&session.graph);
    let mut out = op_value(result)?;
    if let Value::Object(ref mut map) = out {
        map.insert("dangling".to_string(), serde_json::to_value(&dangling).unwrap_or(Value::Null));
    }
    Ok(out)
}

#[cfg(test)]
#[path = "domain_handlers_tests.rs"]
mod tests;
