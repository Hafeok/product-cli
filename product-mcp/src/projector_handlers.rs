//! MCP handlers for `product_projector_*` — parity with `product projector` (§3.4).

use std::path::Path;

use product_core::pf::projector::{derive_projector, validate_projector, Projector};
use product_core::pf::projector_sim::simulate;
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pdir, product_of, req_str};

fn projectors_dir(repo_root: &Path) -> std::path::PathBuf {
    pdir(repo_root).join("projectors")
}

fn load(repo_root: &Path, name: &str) -> Result<Projector, String> {
    load_yaml(&projectors_dir(repo_root), name, Projector::from_yaml)
}

pub fn handle_projector_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "projectors": ids_in(&projectors_dir(repo_root)) }))
}

pub fn handle_projector_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let p = load(repo_root, &req_str(args, "name")?)?;
    serde_json::to_value(&p).map_err(|e| format!("{e}"))
}

pub fn handle_projector_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let p = load(repo_root, &req_str(args, "name")?)?;
    let graph = graph_of(args, repo_root)?;
    let violations = validate_projector(&p, &graph);
    Ok(json!({ "ok": !violations.iter().any(|v| v.severity == "violation"), "violations": violations }))
}

pub fn handle_projector_simulate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let p = load(repo_root, &req_str(args, "name")?)?;
    let findings = simulate(&p);
    Ok(json!({ "sound_and_complete": findings.is_empty(), "findings": findings }))
}

pub fn handle_projector_derive(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let read_model = req_str(args, "read_model")?;
    let graph = graph_of(args, repo_root)?;
    let projector = derive_projector(&graph, &read_model).map_err(|e| format!("{e}"))?;
    let dir = projectors_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
    let path = dir.join(format!("{}.yaml", projector.id));
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    if path.exists() && !force {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    let yaml = projector.to_yaml().map_err(|e| format!("{e}"))?;
    std::fs::write(&path, yaml).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": projector.id, "product": product_of(args, repo_root)? }))
}

#[cfg(test)]
#[path = "projector_handlers_tests.rs"]
mod tests;
