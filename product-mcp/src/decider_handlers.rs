//! MCP handlers for `product_decider_*` — parity with `product decider` (§3.3).

use std::path::Path;

use product_core::pf::decider::{derive_decider, validate_decider, Decider};
use product_core::pf::decider_sim::simulate;
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pdir, product_of, req_str};

fn deciders_dir(repo_root: &Path) -> std::path::PathBuf {
    pdir(repo_root).join("deciders")
}

fn load(repo_root: &Path, name: &str) -> Result<Decider, String> {
    load_yaml(&deciders_dir(repo_root), name, Decider::from_yaml)
}

pub fn handle_decider_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "deciders": ids_in(&deciders_dir(repo_root)) }))
}

pub fn handle_decider_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load(repo_root, &req_str(args, "name")?)?;
    serde_json::to_value(&d).map_err(|e| format!("{e}"))
}

pub fn handle_decider_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load(repo_root, &req_str(args, "name")?)?;
    let graph = graph_of(args, repo_root)?;
    let violations = validate_decider(&d, &graph);
    Ok(json!({ "ok": !violations.iter().any(|v| v.severity == "violation"), "violations": violations }))
}

pub fn handle_decider_simulate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load(repo_root, &req_str(args, "name")?)?;
    let findings = simulate(&d);
    Ok(json!({ "sound_and_complete": findings.is_empty(), "findings": findings }))
}

pub fn handle_decider_derive(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let aggregate = req_str(args, "aggregate")?;
    let graph = graph_of(args, repo_root)?;
    let decider = derive_decider(&graph, &aggregate).map_err(|e| format!("{e}"))?;
    let dir = deciders_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
    let path = dir.join(format!("{}.yaml", decider.id));
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    if path.exists() && !force {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    let yaml = decider.to_yaml().map_err(|e| format!("{e}"))?;
    std::fs::write(&path, yaml).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": decider.id, "product": product_of(args, repo_root)? }))
}

#[cfg(test)]
#[path = "decider_handlers_tests.rs"]
mod tests;
