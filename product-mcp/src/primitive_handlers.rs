//! MCP handlers for `product_primitive_*` — parity with `product primitive` (§3.5).
//!
//! A primitive is authored (not derived) and its only graph-free check is the
//! declaration validation; oracle `check` spawns a runner and stays CLI-only.

use std::path::Path;

use product_core::pf::primitive::{validate_primitive, Primitive};
use serde_json::{json, Value};

use crate::pf_mcp::{ids_in, load_yaml, pdir, req_str};

fn primitives_dir(repo_root: &Path) -> std::path::PathBuf {
    pdir(repo_root).join("primitives")
}

fn load(repo_root: &Path, name: &str) -> Result<Primitive, String> {
    load_yaml(&primitives_dir(repo_root), name, Primitive::from_yaml)
}

pub fn handle_primitive_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "primitives": ids_in(&primitives_dir(repo_root)) }))
}

pub fn handle_primitive_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let p = load(repo_root, &req_str(args, "name")?)?;
    serde_json::to_value(&p).map_err(|e| format!("{e}"))
}

pub fn handle_primitive_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let p = load(repo_root, &req_str(args, "name")?)?;
    let violations = validate_primitive(&p);
    Ok(json!({ "ok": !violations.iter().any(|v| v.severity == "violation"), "violations": violations }))
}

#[cfg(test)]
#[path = "primitive_handlers_tests.rs"]
mod tests;
