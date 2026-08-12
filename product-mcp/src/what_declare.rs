//! `product_what_declare` — file the seam a What boundary demands.
//!
//! The interceptor rejects a boundary-forming edit until something in
//! `.ddd/seams/` names the element, so the declaration has to be fileable
//! without leaving the framework's own MCP surface. Entries land in the
//! same store `ddd why` reads, under `contract_location: what:<element>`.

use std::path::Path;

use ddd_core::seam::{self, Filed, SeamDeclaration};
use serde_json::{json, Value};

fn req_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing required argument '{key}'"))
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string).filter(|s| !s.trim().is_empty())
}

pub fn declare(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    if !id.starts_with("seam/") {
        return Err("seam ids are `seam/<area>/<name>`".to_string());
    }
    let seams_dir = repo_root.join(ddd_core::store::STORE_DIR).join("seams");
    let path = seam::path_for(&seams_dir, &id);
    let amend = args.get("amend").and_then(Value::as_bool).unwrap_or(false);
    let existing = seam::resolve_intent(&path, &id, amend)?;
    let entry = build(args, &id, existing)?;
    let yaml = serde_yaml::to_string(&entry).map_err(|e| e.to_string())?;
    product_core::fileops::write_file_atomic(&path, &yaml).map_err(|e| format!("{e}"))?;
    let outcome = if amend { Filed::Amended } else { Filed::Created };
    let warning = entry.verdict_knowledge.trim().is_empty().then_some(
        "verdict_knowledge is empty — this boundary declares seam cost with no demand absorbed; pass `amend: true` to fill it in",
    );
    Ok(json!({
        "ok": true,
        "status": outcome.as_str(),
        "id": id,
        "element": entry.contract_location,
        "warning": warning,
    }))
}

/// Judgement fields amend; the element the declaration is *about* is
/// identity and carries forward (`dec/ddd/amend-explicit-evidence-frozen`).
fn build(
    args: &Value,
    id: &str,
    existing: Option<SeamDeclaration>,
) -> Result<SeamDeclaration, String> {
    let Some(prev) = existing else {
        let element = req_str(args, "element")?;
        return Ok(SeamDeclaration {
            format: 1,
            id: id.to_string(),
            boundary: req_str(args, "boundary")?,
            verdict_knowledge: opt_str(args, "verdict_knowledge").unwrap_or_default(),
            contract_location: ddd_core::what::what_ref(&element),
            obligations: string_list(args, "obligations"),
            metadata: Default::default(),
            bindings: Vec::new(),
            notes: opt_str(args, "notes"),
        });
    };
    Ok(SeamDeclaration {
        format: prev.format,
        id: id.to_string(),
        boundary: opt_str(args, "boundary").unwrap_or(prev.boundary),
        verdict_knowledge: opt_str(args, "verdict_knowledge").unwrap_or(prev.verdict_knowledge),
        contract_location: prev.contract_location,
        obligations: match args.get("obligations") {
            Some(_) => string_list(args, "obligations"),
            None => prev.obligations,
        },
        metadata: prev.metadata,
        bindings: prev.bindings,
        notes: opt_str(args, "notes").or(prev.notes),
    })
}

fn string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}
