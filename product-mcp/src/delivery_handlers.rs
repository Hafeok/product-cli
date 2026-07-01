//! MCP handlers for `product_feature_*`, `product_deliverable_*`, and
//! `product_release_*` — parity with the §7 delivery commands.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use product_core::pf::bundle::bundle_many;
use product_core::pf::deliverable::{validate_deliverable, AcceptanceCriterion, Deliverable};
use product_core::pf::done::{feature_done, release_done};
use product_core::pf::release::{validate_release, Release};
use product_core::pf::feature::{validate_feature, Feature};
use product_core::pf::target::{direction, validate_target, Target};
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pdir, product_of, req_str};

fn features_dir(r: &Path) -> PathBuf { pdir(r).join("features") }
fn deliverables_dir(r: &Path) -> PathBuf { pdir(r).join("deliverables") }
fn releases_dir(r: &Path) -> PathBuf { pdir(r).join("releases") }
fn targets_dir(r: &Path) -> PathBuf { pdir(r).join("targets") }
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

fn projectors_dir(r: &Path) -> PathBuf {
    pdir(r).join("projectors")
}

fn load_projectors(repo_root: &Path) -> Vec<product_core::pf::projector::Projector> {
    ids_in(&projectors_dir(repo_root))
        .iter()
        .filter_map(|n| load_yaml(&projectors_dir(repo_root), n, product_core::pf::projector::Projector::from_yaml).ok())
        .collect()
}

/// Decider ids with a recorded passing conformance verdict (`<id>.conform.json`).
fn conformed_set(repo_root: &Path) -> BTreeSet<String> {
    let dir = deciders_dir(repo_root);
    let mut out = BTreeSet::new();
    let Ok(entries) = std::fs::read_dir(&dir) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { continue };
        if std::fs::read_to_string(&p).ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.get("conformant").and_then(|c| c.as_bool()))
            .unwrap_or(false)
        {
            out.insert(stem.trim_end_matches(".conform").to_string());
        }
    }
    out
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

// --- feature -----------------------------------------------------------------

pub fn handle_feature_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "features": ids_in(&features_dir(repo_root)) }))
}

pub fn handle_feature_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let s = load_yaml(&features_dir(repo_root), &req_str(args, "name")?, Feature::from_yaml)?;
    serde_json::to_value(&s).map_err(|e| format!("{e}"))
}

pub fn handle_feature_context(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let s = load_yaml(&features_dir(repo_root), &req_str(args, "name")?, Feature::from_yaml)?;
    let graph = graph_of(args, repo_root)?;
    let depth = args.get("depth").and_then(|v| v.as_u64()).map(|d| d as usize).unwrap_or_else(|| s.depth());
    let bundle = bundle_many(&graph, &s.anchors, depth, &product_of(args, repo_root)?)
        .ok_or_else(|| "feature resolves to no nodes in the What graph".to_string())?;
    Ok(json!({ "bundle": bundle }))
}

pub fn handle_feature_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let feature = Feature {
        id: id.clone(),
        anchors: str_array(args, "anchors"),
        depth: args.get("depth").and_then(|v| v.as_u64()).map(|d| d as usize),
    };
    let problems = validate_feature(&feature, &graph_of(args, repo_root)?);
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    write_new(&features_dir(repo_root), &id, feature.to_yaml().map_err(|e| format!("{e}"))?, force)?;
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
            Some((i, st)) => AcceptanceCriterion { id: i.trim().into(), statement: st.trim().into(), status: "pending".into(), runner: None, runner_args: None },
            None => AcceptanceCriterion { id: s.trim().into(), statement: String::new(), status: "pending".into(), runner: None, runner_args: None },
        })
        .collect();
    let feature = req_str(args, "feature").or_else(|_| req_str(args, "slice"))?;
    let d = Deliverable { id: id.clone(), feature, acceptance };
    let problems = validate_deliverable(&d, &id_set(&features_dir(repo_root)));
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

pub fn handle_deliverable_runner(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let criterion = req_str(args, "criterion")?;
    let runner = req_str(args, "runner")?; // "cargo-test" | "shell"
    if runner != "cargo-test" && runner != "shell" {
        return Err(format!("unknown runner '{runner}' — use cargo-test or shell"));
    }
    let runner_args = args.get("args").and_then(|v| v.as_str())
        .map(str::to_string).filter(|s| !s.trim().is_empty());
    let mut d = load_yaml(&deliverables_dir(repo_root), &id, Deliverable::from_yaml)?;
    let c = d.acceptance.iter_mut().find(|c| c.id == criterion)
        .ok_or_else(|| format!("no acceptance criterion '{criterion}' on '{id}'"))?;
    c.runner = Some(runner.clone());
    c.runner_args = runner_args.clone();
    std::fs::write(deliverables_dir(repo_root).join(format!("{id}.yaml")), d.to_yaml().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "id": id, "criterion": criterion, "runner": runner, "runner_args": runner_args }))
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    use product_core::pf::deliverable::AcceptanceCriterion;

    fn repo_with_deliverable() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let dd = deliverables_dir(dir.path());
        std::fs::create_dir_all(&dd).expect("mkdir");
        let d = Deliverable {
            id: "d1".into(),
            feature: "s1".into(),
            acceptance: vec![AcceptanceCriterion {
                id: "c1".into(),
                statement: "x holds".into(),
                status: "pending".into(),
                runner: None,
                runner_args: None,
            }],
        };
        std::fs::write(dd.join("d1.yaml"), d.to_yaml().expect("yaml")).expect("write");
        dir
    }

    #[test]
    fn runner_binds_persists_and_validates() {
        let r = repo_with_deliverable();
        let root = r.path();

        let out = handle_deliverable_runner(
            &json!({"id": "d1", "criterion": "c1", "runner": "cargo-test", "args": "tc_x"}),
            root,
        )
        .expect("bind");
        assert_eq!(out["runner"], json!("cargo-test"));
        assert_eq!(out["runner_args"], json!("tc_x"));

        let d = load_yaml(&deliverables_dir(root), "d1", Deliverable::from_yaml).expect("reload");
        assert_eq!(d.acceptance[0].runner.as_deref(), Some("cargo-test"));
        assert_eq!(d.acceptance[0].runner_args.as_deref(), Some("tc_x"));

        // Unknown runner and unknown criterion are both rejected.
        assert!(handle_deliverable_runner(&json!({"id": "d1", "criterion": "c1", "runner": "make"}), root).is_err());
        assert!(handle_deliverable_runner(&json!({"id": "d1", "criterion": "ghost", "runner": "shell", "args": "true"}), root).is_err());
    }
}

pub fn handle_deliverable_done(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let d = load_yaml(&deliverables_dir(repo_root), &req_str(args, "name")?, Deliverable::from_yaml)?;
    let feature = load_yaml(&features_dir(repo_root), &d.feature, Feature::from_yaml)?;
    let fd = feature_done(&d, &feature, &graph_of(args, repo_root)?, &load_deciders(repo_root), &conformed_set(repo_root), &load_projectors(repo_root));
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
        let s = load_yaml(&features_dir(repo_root), &d.feature, Feature::from_yaml)?;
        members.push((d, s));
    }
    let rd = release_done(&r.id, &members, &graph, &deciders, &conformed_set(repo_root), &load_projectors(repo_root));
    Ok(json!({ "id": rd.id, "done": rd.done, "closed": rd.closed(),
        "members": rd.members.iter().map(|m| json!({"id": m.id, "done": m.done})).collect::<Vec<_>>(),
        "open_edges": rd.open_edges.iter().map(|(n, d)| json!({"node": n, "depends_on_excluded": d})).collect::<Vec<_>>() }))
}

// --- §7.3 target versions + direction -------------------------------------

pub fn handle_target_list(_args: &Value, repo_root: &Path) -> Result<Value, String> {
    Ok(json!({ "targets": ids_in(&targets_dir(repo_root)) }))
}
pub fn handle_target_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let t = load_yaml(&targets_dir(repo_root), &req_str(args, "name")?, Target::from_yaml)?;
    serde_json::to_value(&t).map_err(|e| format!("{e}"))
}
pub fn handle_target_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let mut members = str_array(args, "features");
    if members.is_empty() { members = str_array(args, "slices"); } // back-compat alias
    let t = Target { id: id.clone(), version: args.get("version").and_then(|v| v.as_str()).map(String::from),
        in_target: members };
    let problems = validate_target(&t, &id_set(&deliverables_dir(repo_root)));
    if !problems.is_empty() {
        return Ok(json!({ "ok": false, "violations": problems }));
    }
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    write_new(&targets_dir(repo_root), &id, t.to_yaml().map_err(|e| format!("{e}"))?, force)?;
    Ok(json!({ "ok": true, "id": id }))
}
pub fn handle_target_direction(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let t = load_yaml(&targets_dir(repo_root), &req_str(args, "name")?, Target::from_yaml)?;
    let graph = graph_of(args, repo_root)?;
    let deciders = load_deciders(repo_root);
    let projectors = load_projectors(repo_root);
    let conformed = conformed_set(repo_root);
    let mut done = std::collections::BTreeMap::new();
    for m in &t.in_target {
        if let Ok(d) = load_yaml(&deliverables_dir(repo_root), m, Deliverable::from_yaml) {
            if let Ok(s) = load_yaml(&features_dir(repo_root), &d.feature, Feature::from_yaml) {
                done.insert(m.clone(), feature_done(&d, &s, &graph, &deciders, &conformed, &projectors).done);
            }
        }
    }
    let dir = direction(&t, &done);
    Ok(json!({ "id": t.id, "version": dir.version, "total": dir.total,
        "unrealised": dir.unrealised, "progress": dir.progress() }))
}

#[cfg(test)]
#[path = "delivery_handlers_tests.rs"]
mod tests;
