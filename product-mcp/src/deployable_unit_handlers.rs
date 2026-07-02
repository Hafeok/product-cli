//! MCP handlers for `product_deployable_unit_*` — parity with the §4/§4.2
//! `product deployable-unit` commands (the concrete artifact a blueprint
//! produces for a system, carrying its deployment identity).

use std::path::{Path, PathBuf};

use product_core::pf::deployable_unit::{
    validate_deployable_unit, DeployableUnit, DeploymentIdentity,
};
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pdir, req_str};

fn units_dir(r: &Path) -> PathBuf {
    pdir(r).join("deployable-units")
}

/// Prefer `.product/blueprints/`, fall back to the legacy `.product/archetypes/`.
fn blueprints_dir(r: &Path) -> PathBuf {
    let blueprints = pdir(r).join("blueprints");
    if blueprints.is_dir() {
        return blueprints;
    }
    let legacy = pdir(r).join("archetypes");
    if legacy.is_dir() {
        return legacy;
    }
    blueprints
}

/// Blueprint names available on disk (directory names under blueprints_dir).
fn known_blueprints(r: &Path) -> Vec<String> {
    product_core::pf::deployable_unit::blueprint_names(&blueprints_dir(r))
}

fn str_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(String::from).filter(|s| !s.trim().is_empty())
}

pub fn handle_deployable_unit_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "deployable_units": ids_in(&units_dir(repo_root)) }))
}

pub fn handle_deployable_unit_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let du = load_yaml(&units_dir(repo_root), &req_str(args, "name")?, DeployableUnit::from_yaml)?;
    serde_json::to_value(&du).map_err(|e| format!("{e}"))
}

pub fn handle_deployable_unit_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let du = load_yaml(&units_dir(repo_root), &req_str(args, "name")?, DeployableUnit::from_yaml)?;
    let graph = graph_of(args, repo_root).ok();
    let problems = validate_deployable_unit(&du, graph.as_ref(), &known_blueprints(repo_root));
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    Ok(json!({ "ok": true, "id": du.id }))
}

pub fn handle_deployable_unit_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let du = DeployableUnit {
        id: id.clone(),
        built_from: req_str(args, "built_from")?,
        deploys_system: str_array(args, "deploys_system"),
        environment: opt_str(args, "environment"),
        identity: DeploymentIdentity {
            domain_name: opt_str(args, "domain_name"),
            bundle_id: opt_str(args, "bundle_id"),
            runtime: opt_str(args, "runtime"),
        },
    };
    let graph = graph_of(args, repo_root).ok();
    let problems = validate_deployable_unit(&du, graph.as_ref(), &known_blueprints(repo_root));
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let dir = units_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
    let path = dir.join(format!("{id}.yaml"));
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    if path.exists() && !force {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    std::fs::write(&path, du.to_yaml().map_err(|e| format!("{e}"))?).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": id }))
}

#[cfg(test)]
#[path = "deployable_unit_handlers_tests.rs"]
mod tests;
