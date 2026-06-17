//! MCP handlers for `product_slice_*`, `product_deliverable_*`, and
//! `product_release_*` — parity with the §7 delivery commands.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use product_core::pf::bundle::bundle_many;
use product_core::pf::deliverable::{validate_deliverable, AcceptanceCriterion, Deliverable};
use product_core::pf::done::{feature_done, release_done};
use product_core::pf::release::{validate_release, Release};
use product_core::pf::slice::{validate_slice, Slice};
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pdir, product_of, req_str};

fn slices_dir(r: &Path) -> PathBuf { pdir(r).join("slices") }
fn deliverables_dir(r: &Path) -> PathBuf { pdir(r).join("deliverables") }
fn releases_dir(r: &Path) -> PathBuf { pdir(r).join("releases") }
fn deciders_dir(r: &Path) -> PathBuf { pdir(r).join("deciders") }

fn id_set(dir: &Path) -> BTreeSet<String> {
    ids_in(dir).into_iter().collect()
}

fn load_deciders(repo_root: &Path) -> Vec<product_core::pf::decider::Decider> {
    ids_in(&deciders_dir(repo_root))
        .iter()
        .filter_map(|n| load_yaml(&deciders_dir(repo_root), n, product_core::pf::decider::Decider::from_yaml).ok())
        .collect()
}

fn str_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn write_new(dir: &Path, id: &str, yaml: String, force: bool) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{e}"))?;
    let path = dir.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass force=true to overwrite", path.display()));
    }
    std::fs::write(&path, yaml).map_err(|e| format!("{e}"))
}

// --- slice -----------------------------------------------------------------

pub fn handle_slice_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "slices": ids_in(&slices_dir(repo_root)) }))
}

pub fn handle_slice_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let s = load_yaml(&slices_dir(repo_root), &req_str(args, "name")?, Slice::from_yaml)?;
    serde_json::to_value(&s).map_err(|e| format!("{e}"))
}

pub fn handle_slice_context(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let s = load_yaml(&slices_dir(repo_root), &req_str(args, "name")?, Slice::from_yaml)?;
    let graph = graph_of(args, repo_root)?;
    let depth = args.get("depth").and_then(|v| v.as_u64()).map(|d| d as usize).unwrap_or_else(|| s.depth());
    let bundle = bundle_many(&graph, &s.anchors, depth, &product_of(args, repo_root)?)
        .ok_or_else(|| "slice resolves to no nodes in the What graph".to_string())?;
    Ok(json!({ "bundle": bundle }))
}

pub fn handle_slice_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let slice = Slice {
        id: id.clone(),
        anchors: str_array(args, "anchors"),
        depth: args.get("depth").and_then(|v| v.as_u64()).map(|d| d as usize),
    };
    let problems = validate_slice(&slice, &graph_of(args, repo_root)?);
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    write_new(&slices_dir(repo_root), &id, slice.to_yaml().map_err(|e| format!("{e}"))?, force)?;
    Ok(json!({ "ok": true, "id": id }))
}

// --- deliverable -----------------------------------------------------------

pub fn handle_deliverable_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "deliverables": ids_in(&deliverables_dir(repo_root)) }))
}

pub fn handle_deliverable_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load_yaml(&deliverables_dir(repo_root), &req_str(args, "name")?, Deliverable::from_yaml)?;
    serde_json::to_value(&d).map_err(|e| format!("{e}"))
}

pub fn handle_deliverable_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let acceptance = str_array(args, "acceptance")
        .into_iter()
        .map(|s| match s.split_once(':') {
            Some((i, st)) => AcceptanceCriterion { id: i.trim().into(), statement: st.trim().into(), status: "pending".into() },
            None => AcceptanceCriterion { id: s.trim().into(), statement: String::new(), status: "pending".into() },
        })
        .collect();
    let d = Deliverable { id: id.clone(), slice: req_str(args, "slice")?, acceptance };
    let problems = validate_deliverable(&d, &id_set(&slices_dir(repo_root)));
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    write_new(&deliverables_dir(repo_root), &id, d.to_yaml().map_err(|e| format!("{e}"))?, force)?;
    Ok(json!({ "ok": true, "id": id }))
}

pub fn handle_deliverable_accept(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let criterion = req_str(args, "criterion")?;
    let status = req_str(args, "status")?; // "passing" | "failing"
    let mut d = load_yaml(&deliverables_dir(repo_root), &id, Deliverable::from_yaml)?;
    let c = d.acceptance.iter_mut().find(|c| c.id == criterion)
        .ok_or_else(|| format!("no acceptance criterion '{criterion}' on '{id}'"))?;
    c.status = status.clone();
    std::fs::write(deliverables_dir(repo_root).join(format!("{id}.yaml")), d.to_yaml().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": id, "criterion": criterion, "status": status }))
}

pub fn handle_deliverable_done(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load_yaml(&deliverables_dir(repo_root), &req_str(args, "name")?, Deliverable::from_yaml)?;
    let slice = load_yaml(&slices_dir(repo_root), &d.slice, Slice::from_yaml)?;
    let fd = feature_done(&d, &slice, &graph_of(args, repo_root)?, &load_deciders(repo_root));
    Ok(json!({ "id": fd.id, "done": fd.done, "progress": fd.progress(),
        "checks": fd.checks.iter().map(|c| json!({"kind": c.kind, "subject": c.subject, "passing": c.passing, "detail": c.detail})).collect::<Vec<_>>() }))
}

// --- release ---------------------------------------------------------------

pub fn handle_release_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "releases": ids_in(&releases_dir(repo_root)) }))
}

pub fn handle_release_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let r = load_yaml(&releases_dir(repo_root), &req_str(args, "name")?, Release::from_yaml)?;
    serde_json::to_value(&r).map_err(|e| format!("{e}"))
}

pub fn handle_release_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let r = Release { id: id.clone(), features: str_array(args, "features") };
    let problems = validate_release(&r, &id_set(&deliverables_dir(repo_root)));
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    write_new(&releases_dir(repo_root), &id, r.to_yaml().map_err(|e| format!("{e}"))?, force)?;
    Ok(json!({ "ok": true, "id": id }))
}

pub fn handle_release_done(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let r = load_yaml(&releases_dir(repo_root), &req_str(args, "name")?, Release::from_yaml)?;
    let graph = graph_of(args, repo_root)?;
    let deciders = load_deciders(repo_root);
    let mut members = Vec::new();
    for f in &r.features {
        let d = load_yaml(&deliverables_dir(repo_root), f, Deliverable::from_yaml)?;
        let s = load_yaml(&slices_dir(repo_root), &d.slice, Slice::from_yaml)?;
        members.push((d, s));
    }
    let rd = release_done(&r.id, &members, &graph, &deciders);
    Ok(json!({ "id": rd.id, "done": rd.done, "closed": rd.closed(),
        "members": rd.members.iter().map(|m| json!({"id": m.id, "done": m.done})).collect::<Vec<_>>(),
        "open_edges": rd.open_edges.iter().map(|(n, d)| json!({"node": n, "depends_on_excluded": d})).collect::<Vec<_>>() }))
}

#[cfg(test)]
#[path = "delivery_handlers_tests.rs"]
mod tests;
