//! Shared helpers for the Product-Framework MCP handlers (delivery + decider).
//!
//! These mirror the CLI adapters' loading against `repo_root/.product/…` so the
//! framework families (`decider`, `feature`, `deliverable`, `release`) expose the
//! same functionality over MCP as on the CLI.

use std::path::{Path, PathBuf};

use product_core::author::domain::session_dir;
use product_core::config::ProductConfig;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use serde_json::Value;

/// The product to operate on: the `product` arg, else the repo's configured name.
pub(crate) fn product_of(args: &Value, repo_root: &Path) -> Result<String, String> {
    if let Some(p) = args.get("product").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
        return Ok(p.to_string());
    }
    let cfg = ProductConfig::load_from_root(repo_root).map_err(|e| format!("{e}"))?;
    let name = cfg.name.trim();
    if name.is_empty() {
        Err("no product — pass `product` or set `name` in product.toml".to_string())
    } else {
        Ok(name.to_string())
    }
}

/// Load the captured What graph for the resolved product.
pub(crate) fn graph_of(args: &Value, repo_root: &Path) -> Result<DomainGraph, String> {
    let p = product_of(args, repo_root)?;
    DomainSession::load(&session_dir(repo_root, &p))
        .map(|s| s.graph)
        .map_err(|_| format!("no captured What graph for '{p}' — author one with `product author domain`"))
}

/// The `.product` directory under the repo root.
pub(crate) fn pdir(repo_root: &Path) -> PathBuf {
    repo_root.join(".product")
}

/// Sorted artifact ids (filename stems) under a directory.
pub(crate) fn ids_in(dir: &Path) -> Vec<String> {
    let mut ids: Vec<String> = match std::fs::read_dir(dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect(),
        Err(_) => Vec::new(),
    };
    ids.sort();
    ids
}

/// Read a required string argument.
pub(crate) fn req_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing required argument '{key}'"))
}

/// Load a YAML artifact `<dir>/<name>.yaml`, parsed by `parse`.
pub(crate) fn load_yaml<T>(dir: &Path, name: &str, parse: impl Fn(&str) -> product_core::error::Result<T>) -> Result<T, String> {
    let path = dir.join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path).map_err(|_| format!("'{name}' not found at {}", path.display()))?;
    parse(&text).map_err(|e| format!("{e}"))
}
