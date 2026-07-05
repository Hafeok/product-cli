//! Handlers for the `product_reify_*` tools (Build phase).
//!
//! Thin adapters over the `product-core` reify slice: resolve the
//! product, What graph, and authored Deciders/Projectors via the shared
//! `pf_mcp` loaders, then delegate to the backend registry / manifest /
//! drift check. `emit` writes generated trees under the repo root (never
//! under `.product/` — reify reads the spec, it does not mutate it).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use product_core::pf::decider::Decider;
use product_core::pf::projector::Projector;
use product_core::pf::reify::{input_hash, recorded_hash, ReifyOptions, ReifyPlan};
use product_core::pf::reify_backend::{backend, backends};
use product_core::pf::reify_ident::pascal;

use crate::pf_mcp::{graph_of, ids_in, load_yaml, pbase, product_of, req_str};

pub fn handle_backends(_args: &Value, _repo_root: &Path) -> Result<Value, String> {
    let list: Vec<Value> = backends()
        .iter()
        .map(|b| {
            json!({
                "id": b.id(),
                "description": b.description(),
                "oracle_only_forced": b.oracle_only_forced(),
            })
        })
        .collect();
    Ok(json!({ "ok": true, "backends": list }))
}

pub fn handle_manifest(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (graph, deciders, projectors, opts) = inputs(args, repo_root)?;
    let m = product_core::pf::reify_manifest::manifest(&graph, &deciders, &projectors, &opts)
        .map_err(|e| format!("{e}"))?;
    let manifest = serde_json::to_value(&m).map_err(|e| format!("serialize manifest: {e}"))?;
    Ok(json!({ "ok": true, "manifest": manifest }))
}

pub fn handle_check(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let out = req_str(args, "out")?;
    let root = safe_join(repo_root, &out)?;
    let (graph, deciders, projectors, opts) = inputs(args, repo_root)?;
    let current = input_hash(&graph, &opts.product, &deciders, &projectors).map_err(|e| format!("{e}"))?;
    let prov = std::fs::read_to_string(root.join("provenance.g.json"))
        .map_err(|_| format!("no provenance.g.json under '{out}' — emit first"))?;
    let recorded = recorded_hash(&prov).map_err(|e| format!("{e}"))?;
    Ok(json!({
        "ok": true,
        "conformant": recorded == current,
        "current": format!("sha256:{current}"),
        "recorded": format!("sha256:{recorded}"),
        "out": out,
    }))
}

pub fn handle_emit(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let lang = req_str(args, "lang")?;
    let b = backend(&lang).map_err(|e| format!("{e}"))?;
    let (graph, deciders, projectors, mut opts) = inputs(args, repo_root)?;
    opts.oracle_only = b.oracle_only_forced()
        || args.get("oracle_only").and_then(|v| v.as_bool()).unwrap_or(false);
    let plan = b.plan(&graph, &deciders, &projectors, &opts).map_err(|e| format!("{e}"))?;
    let out = args
        .get("out")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("reified/{}/{}", opts.product, b.id()));
    let root = safe_join(repo_root, &out)?;
    let stale = remove_stale(&root, &plan);
    let (written, kept) = write_plan(&root, &plan).map_err(|e| format!("{e}"))?;
    Ok(json!({
        "ok": true,
        "lang": b.id(),
        "oracle_only": opts.oracle_only,
        "out": out,
        "written": written,
        "kept": kept,
        "stale_removed": stale,
        "graph_hash": format!("sha256:{}", plan.graph_hash),
        "aggregates": plan.aggregates,
    }))
}

/// Resolve product name, graph, artifacts, and options from the arguments.
fn inputs(args: &Value, repo_root: &Path) -> Result<(product_core::pf::model::DomainGraph, Vec<Decider>, Vec<Projector>, ReifyOptions), String> {
    let product = product_of(args, repo_root)?;
    let graph = graph_of(args, repo_root)?;
    let base = pbase(args, repo_root);
    let deciders: Vec<Decider> = ids_in(&base.join("deciders"))
        .iter()
        .filter_map(|n| load_yaml(&base.join("deciders"), n, Decider::from_yaml).ok())
        .collect();
    let projectors: Vec<Projector> = ids_in(&base.join("projectors"))
        .iter()
        .filter_map(|n| load_yaml(&base.join("projectors"), n, Projector::from_yaml).ok())
        .collect();
    let namespace = args
        .get("namespace")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| pascal(&product));
    let what_version = product_core::pf::HowContract::load_opt(&base.join("how-contract.yaml"))
        .ok()
        .flatten()
        .and_then(|c| c.realises_version)
        .unwrap_or_else(|| "unversioned".to_string());
    Ok((graph, deciders, projectors, ReifyOptions { product, namespace, what_version, oracle_only: false }))
}

/// Join a caller-supplied relative path under the repo root, rejecting
/// absolute paths and traversal.
fn safe_join(repo_root: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return Err(format!("'{rel}' must be a relative path inside the repository"));
    }
    Ok(repo_root.join(rel))
}

/// Mirror of the CLI's manifest-driven stale-file cleanup.
fn remove_stale(root: &Path, plan: &ReifyPlan) -> usize {
    let Ok(prev) = std::fs::read_to_string(root.join("provenance.g.json")) else { return 0 };
    let Ok(v) = serde_json::from_str::<Value>(&prev) else { return 0 };
    let current: std::collections::BTreeSet<&str> = plan.files.iter().map(|f| f.path.as_str()).collect();
    let mut removed = 0;
    for old in v.get("generated_files").and_then(|f| f.as_array()).into_iter().flatten() {
        let Some(path) = old.as_str() else { continue };
        if !current.contains(path) && std::fs::remove_file(root.join(path)).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Write the plan (scaffolds only when absent — MCP has no --force).
fn write_plan(root: &Path, plan: &ReifyPlan) -> std::io::Result<(usize, usize)> {
    let (mut written, mut kept) = (0usize, 0usize);
    for f in &plan.files {
        let path = root.join(&f.path);
        if !f.overwrite && path.exists() {
            kept += 1;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &f.content)?;
        written += 1;
    }
    Ok((written, kept))
}
