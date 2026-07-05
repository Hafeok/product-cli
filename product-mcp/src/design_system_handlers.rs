//! MCP handlers for `product_design_system_*` — parity with the §11
//! `product design-system` CLI family (vendor, validate, couple, bind).

use std::path::{Path, PathBuf};

use product_core::pf::ds_store;
use product_core::pf::how::DesignSystemBinding;
use product_core::pf::HowContract;
use serde_json::{json, Value};

use crate::pf_mcp::{graph_of, pbase, req_str};

pub fn handle_ds_list(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let base = pbase(args, repo_root);
    let bound = bound_id(&base);
    let systems: Vec<Value> = ds_store::list(&base)
        .into_iter()
        .map(|id| {
            let bound_here = Some(&id) == bound.as_ref();
            match ds_store::load(&base, &id) {
                Ok(s) => {
                    let ds = &s.manifest.design_system;
                    json!({ "id": id, "version": ds.version, "components": ds.components.len(),
                            "reification_rules": ds.reification.len(), "tokens": ds.tokens.len(), "bound": bound_here })
                }
                Err(_) => json!({ "id": id, "bound": bound_here }),
            }
        })
        .collect();
    Ok(json!({ "design_systems": systems }))
}

pub fn handle_ds_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let base = pbase(args, repo_root);
    let stored = resolve(args, &base)?;
    let ds = &stored.manifest.design_system;
    Ok(json!({
        "id": ds.id,
        "version": ds.version,
        "hash": format!("sha256:{}", stored.hash()),
        "wcag_target": ds.wcag_target,
        "targets": ds.targets,
        "themes": ds.themes,
        "components": ds.components.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        "reification_rules": ds.reification.len(),
        "tokens": ds.tokens.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
        "templates": ds.templates.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
    }))
}

pub fn handle_ds_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let base = pbase(args, repo_root);
    let stored = resolve(args, &base)?;
    let mut findings = product_core::pf::manifest::validate_ds(&stored.manifest);
    findings.extend(product_core::pf::manifest_bundle::validate_bundle(&stored.manifest, &stored.dir));
    Ok(json!({ "ok": findings.is_empty(), "id": stored.manifest.design_system.id, "findings": findings }))
}

pub fn handle_ds_couple(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let base = pbase(args, repo_root);
    let stored = resolve(args, &base)?;
    let graph = graph_of(args, repo_root)?;
    let findings = product_core::pf::manifest::couple_ds(&stored.manifest, &graph);
    Ok(json!({ "ok": findings.is_empty(), "id": stored.manifest.design_system.id, "findings": findings }))
}

pub fn handle_ds_add(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let rel = req_str(args, "manifest_path")?;
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return Err(format!("manifest_path '{rel}' must be relative and stay inside the repo"));
    }
    let base = pbase(args, repo_root);
    let stored = ds_store::save(&base, &repo_root.join(&rel)).map_err(|e| format!("{e}"))?;
    let ds = &stored.manifest.design_system;
    Ok(json!({
        "ok": true, "id": ds.id, "version": ds.version,
        "hash": format!("sha256:{}", stored.hash()),
        "components": ds.components.len(),
    }))
}

pub fn handle_ds_bind(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let base = pbase(args, repo_root);
    let stored = ds_store::load(&base, &id).map_err(|e| format!("{e}"))?;
    let path = how_path(&base);
    let mut c = HowContract::load_opt(&path)
        .map_err(|e| format!("{e}"))?
        .unwrap_or_else(|| HowContract { blueprint: "blueprint".to_string(), ..Default::default() });
    let ds = &stored.manifest.design_system;
    c.design_system = Some(DesignSystemBinding {
        id: ds.id.clone(),
        version: (!ds.version.is_empty()).then(|| ds.version.clone()),
    });
    let yaml = c.to_yaml().map_err(|e| format!("{e}"))?;
    product_core::fileops::write_file_atomic(&path, &yaml).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "bound": ds.id, "version": ds.version }))
}

fn how_path(base: &Path) -> PathBuf {
    base.join("how-contract.yaml")
}

fn bound_id(base: &Path) -> Option<String> {
    HowContract::load_opt(&how_path(base)).ok().flatten().and_then(|c| c.design_system).map(|b| b.id)
}

/// Resolve `id` (arg), else the How-bound design system.
fn resolve(args: &Value, base: &Path) -> Result<ds_store::StoredDs, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| bound_id(base))
        .ok_or("no design system named and none bound — pass `id` or bind one first")?;
    ds_store::load(base, &id).map_err(|e| format!("{e}"))
}
