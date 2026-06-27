//! MCP write handlers that scaffold the delivery architecture — archetypes,
//! cells, work units (CLI↔MCP parity for `product archetype/cell/work-unit init`
//! plus `product cell dispatch`).
//!
//! Each scaffold reuses the same core constructors the CLI uses
//! (`Archetype::scaffold`, `TaskType::scaffold`, `WorkUnit::scaffold`,
//! `dispatch`), so the two surfaces lay down identical artifacts.

use std::path::{Path, PathBuf};

use product_core::pf::archetype::Archetype;
use product_core::pf::cell::TaskType;
use product_core::pf::dispatch::dispatch;
use product_core::pf::work_unit::WorkUnit;
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, load_yaml, pdir, req_str};

fn write_yaml(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{e}"))?;
    }
    product_core::fileops::write_file_atomic(path, text).map_err(|e| format!("{e}"))
}

fn force(args: &Value) -> bool {
    args.get("force").and_then(|v| v.as_bool()).unwrap_or(false)
}

fn opt(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(str::to_string).filter(|s| !s.trim().is_empty())
}

/// Resolve a `file` arg (absolute, or relative to the repo) else a default name
/// under `.product/`.
fn file_arg(args: &Value, repo_root: &Path, default: &str) -> PathBuf {
    match opt(args, "file") {
        Some(f) => {
            let p = PathBuf::from(&f);
            if p.is_absolute() { p } else { repo_root.join(p) }
        }
        None => pdir(repo_root).join(default),
    }
}

/// Scaffold a starter work unit at .product/work-unit.yaml (or `file`).
pub fn handle_work_unit_init(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let path = file_arg(args, repo_root, "work-unit.yaml");
    if path.exists() && !force(args) {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    let text = WorkUnit::scaffold(&id).to_yaml().map_err(|e| format!("{e}"))?;
    write_yaml(&path, &text)?;
    Ok(json!({ "ok": true, "id": id, "written": path.display().to_string() }))
}

/// Scaffold a starter task-type (cell) at .product/cell.yaml (or `file`).
pub fn handle_cell_init(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let archetype = opt(args, "archetype").unwrap_or_else(|| "archetype".to_string());
    let path = file_arg(args, repo_root, "cell.yaml");
    if path.exists() && !force(args) {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    let text = TaskType::scaffold(&id, &archetype).to_yaml().map_err(|e| format!("{e}"))?;
    write_yaml(&path, &text)?;
    Ok(json!({ "ok": true, "id": id, "archetype": archetype, "written": path.display().to_string() }))
}

/// Dispatch the cell at .product/cell.yaml into concrete §5 work units, bound to
/// the captured What graph, written under .product/work-units/.
pub fn handle_cell_dispatch(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let task = load_yaml(&pdir(repo_root), "cell", TaskType::from_yaml)?;
    let bindings = parse_binds(args)?;
    let domain = graph_of(args, repo_root).ok();
    let result = dispatch(&task, &bindings, domain.as_ref());
    if result.violations.iter().any(|v| v.severity == "violation") {
        return Ok(json!({ "ok": false, "violations": result.violations }));
    }
    let dir = pdir(repo_root).join("work-units");
    let mut written = Vec::new();
    for wu in &result.work_units {
        let path = dir.join(format!("{}.yaml", wu.id));
        let text = wu.to_yaml().map_err(|e| format!("{e}"))?;
        write_yaml(&path, &text)?;
        written.push(path.display().to_string());
    }
    let ids: Vec<String> = result.work_units.iter().map(|w| w.id.clone()).collect();
    Ok(json!({ "ok": true, "workUnits": ids, "written": written, "violations": result.violations }))
}

/// Scaffold a new archetype directory under .product/archetypes/<name>/.
pub fn handle_archetype_init(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let name = req_str(args, "name")?;
    let dir = pdir(repo_root).join("archetypes").join(&name);
    if dir.exists() && !force(args) {
        return Err(format!(
            "archetype '{name}' already exists at {} — pass force=true to overwrite",
            dir.display()
        ));
    }
    let written = Archetype::scaffold(&dir, &name).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "name": name, "written": written }))
}

/// Bindings for dispatch — an object `{slot: value}` or an array of "slot=value".
fn parse_binds(args: &Value) -> Result<Vec<(String, String)>, String> {
    match args.get("binds") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Object(m)) => Ok(m
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| {
                let s = x.as_str().ok_or_else(|| "binds entries must be strings 'slot=value'".to_string())?;
                let (k, val) = s.split_once('=').ok_or_else(|| format!("bind expects slot=value, got {s:?}"))?;
                Ok((k.trim().to_string(), val.trim().to_string()))
            })
            .collect(),
        Some(_) => Err("binds must be an object {slot: value} or an array of 'slot=value'".to_string()),
    }
}

#[cfg(test)]
#[path = "framework_scaffold_handlers_tests.rs"]
mod tests;
