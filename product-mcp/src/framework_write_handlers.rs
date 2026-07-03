//! MCP write handlers for the How layer — author the Why cascade plus the
//! application/infrastructure contracts (CLI↔MCP parity for `product how`).
//!
//! Each mutating tool loads `.product/how-contract.yaml`, builds a typed element
//! from the call arguments, applies it via `how_edit`, re-validates the whole
//! contract in-loop, writes atomically, and returns `{ ok, id, element,
//! violations }` (structured-mcp-ops — never raw text).

use std::path::{Path, PathBuf};

use product_core::pf::how::{
    ApplicationContract, ContractStatement, HowContract, InfrastructureContract, InterfaceContract,
    Pattern, Principle, Resource, TopDecision,
};
use product_core::pf::how_edit as edit;
use product_core::pf::how_validate::validate_how;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use crate::pf_mcp::{load_yaml, pbase, req_str};

fn how_path(base: &Path) -> PathBuf {
    base.join("how-contract.yaml")
}

fn load_how(base: &Path) -> Result<HowContract, String> {
    load_yaml(base, "how-contract", HowContract::from_yaml)
}

fn save_how(base: &Path, c: &HowContract) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| format!("{e}"))?;
    let text = c.to_yaml().map_err(|e| format!("{e}"))?;
    product_core::fileops::write_file_atomic(&how_path(base), &text).map_err(|e| format!("{e}"))
}

/// Deserialize a typed element straight from the call arguments. Selector keys
/// like `element`/`target` are simply ignored (the structs accept unknown keys).
fn from_args<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("invalid fields: {e}"))
}

fn to_val<T: Serialize>(x: &T) -> Result<Value, String> {
    serde_json::to_value(x).map_err(|e| format!("{e}"))
}

/// Persist the mutated contract, re-validate it, and shape the response.
fn finish(base: &Path, c: &HowContract, id: &str, element: Value) -> Result<Value, String> {
    save_how(base, c)?;
    let violations = validate_how(c);
    let ok = !violations.iter().any(|v| v.severity == "violation");
    Ok(json!({ "ok": ok, "id": id, "element": element, "violations": violations }))
}

/// Scaffold a fresh How contract keyed to a blueprint (or the `product` arg).
pub fn handle_how_init(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let blueprint = req_str(args, "blueprint").or_else(|_| req_str(args, "product"))?;
    let base = pbase(args, repo_root);
    let path = how_path(&base);
    if path.exists() {
        return Err(format!(
            "how-contract already exists at {} — edit it with product_how_add / product_how_set",
            path.display()
        ));
    }
    let c = HowContract::scaffold(&blueprint);
    save_how(&base, &c)?;
    Ok(json!({ "ok": true, "created": path.display().to_string(), "blueprint": blueprint }))
}

/// Add a Why-cascade element or a contract part to the How contract.
pub fn handle_how_add(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let element = req_str(args, "element")?;
    let id = req_str(args, "id")?;
    let base = pbase(args, repo_root);
    let mut c = load_how(&base)?;
    let added = match element.as_str() {
        "decision" => {
            let d: TopDecision = from_args(args)?;
            edit::add_decision(&mut c, d.clone()).map_err(|e| format!("{e}"))?;
            to_val(&d)?
        }
        "principle" => {
            let p: Principle = from_args(args)?;
            edit::add_principle(&mut c, p.clone()).map_err(|e| format!("{e}"))?;
            to_val(&p)?
        }
        "pattern" => {
            let p: Pattern = from_args(args)?;
            edit::add_pattern(&mut c, p.clone()).map_err(|e| format!("{e}"))?;
            to_val(&p)?
        }
        "interface" => {
            let i: InterfaceContract = from_args(args)?;
            edit::add_interface(&mut c, i.clone()).map_err(|e| format!("{e}"))?;
            to_val(&i)?
        }
        "app-statement" => {
            let s: ContractStatement = from_args(args)?;
            edit::add_app_statement(&mut c, s.clone()).map_err(|e| format!("{e}"))?;
            to_val(&s)?
        }
        "resource" => {
            let r: Resource = from_args(args)?;
            edit::add_resource(&mut c, r.clone()).map_err(|e| format!("{e}"))?;
            to_val(&r)?
        }
        other => {
            return Err(format!(
                "unknown element '{other}' — one of decision | principle | pattern | interface | app-statement | resource"
            ))
        }
    };
    finish(&base, &c, &id, added)
}

/// Set a singleton contract (the application or infrastructure contract).
pub fn handle_how_set(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let target = req_str(args, "target")?;
    let id = req_str(args, "id")?;
    let base = pbase(args, repo_root);
    let mut c = load_how(&base)?;
    let set = match target.as_str() {
        "app-contract" => {
            let a: ApplicationContract = from_args(args)?;
            edit::set_app_contract(&mut c, a.clone());
            to_val(&a)?
        }
        "infra-contract" => {
            let i: InfrastructureContract = from_args(args)?;
            edit::set_infra_contract(&mut c, i.clone());
            to_val(&i)?
        }
        // §7.3 — the How's own version and the What-version it realises. `id`
        // carries the version string (mirrors the CLI's `--id`).
        "version" => {
            c.version = Some(id.clone());
            json!({ "version": id })
        }
        "realises-version" => {
            c.realises_version = Some(id.clone());
            json!({ "realisesVersion": id })
        }
        other => {
            return Err(format!(
                "unknown target '{other}' — one of app-contract | infra-contract | version | realises-version"
            ))
        }
    };
    finish(&base, &c, &id, set)
}

/// Remove a Why-cascade element or contract part by id.
pub fn handle_how_rm(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let base = pbase(args, repo_root);
    let mut c = load_how(&base)?;
    let removed = edit::remove(&mut c, &id).map_err(|e| format!("{e}"))?;
    save_how(&base, &c)?;
    let violations = validate_how(&c);
    let ok = !violations.iter().any(|v| v.severity == "violation");
    Ok(json!({ "ok": ok, "id": id, "removed": removed, "violations": violations }))
}

/// Overlay the provided fields onto an existing element, then re-type it. Keeps
/// any field the caller did not mention (a patch, like product_domain_edit).
fn patch<T: Serialize + DeserializeOwned>(current: &T, args: &Value) -> Result<T, String> {
    let mut base = to_val(current)?;
    if let (Value::Object(b), Value::Object(incoming)) = (&mut base, args) {
        for (k, v) in incoming {
            if k == "element" || k == "target" {
                continue;
            }
            b.insert(k.clone(), v.clone());
        }
    }
    serde_json::from_value(base).map_err(|e| format!("invalid fields: {e}"))
}

/// Patch an existing Why-cascade element by id (decision | principle | pattern |
/// interface), keeping unmentioned fields.
pub fn handle_how_edit(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let element = req_str(args, "element")?;
    let id = req_str(args, "id")?;
    let base = pbase(args, repo_root);
    let mut c = load_how(&base)?;
    let edited = match element.as_str() {
        "decision" => {
            let cur = c.top_decisions.iter().find(|x| x.id == id).ok_or_else(|| miss(&id))?;
            let next: TopDecision = patch(cur, args)?;
            edit::replace_decision(&mut c, next.clone()).map_err(|e| format!("{e}"))?;
            to_val(&next)?
        }
        "principle" => {
            let cur = c.principles.iter().find(|x| x.id == id).ok_or_else(|| miss(&id))?;
            let next: Principle = patch(cur, args)?;
            edit::replace_principle(&mut c, next.clone()).map_err(|e| format!("{e}"))?;
            to_val(&next)?
        }
        "pattern" => {
            let cur = c.patterns.iter().find(|x| x.id == id).ok_or_else(|| miss(&id))?;
            let next: Pattern = patch(cur, args)?;
            edit::replace_pattern(&mut c, next.clone()).map_err(|e| format!("{e}"))?;
            to_val(&next)?
        }
        "interface" => {
            let cur = c.interface_contracts.iter().find(|x| x.id == id).ok_or_else(|| miss(&id))?;
            let next: InterfaceContract = patch(cur, args)?;
            edit::replace_interface(&mut c, next.clone()).map_err(|e| format!("{e}"))?;
            to_val(&next)?
        }
        other => {
            return Err(format!(
                "unknown element '{other}' — product_how_edit handles decision | principle | pattern | interface (set contracts with product_how_set)"
            ))
        }
    };
    finish(&base, &c, &id, edited)
}

fn miss(id: &str) -> String {
    format!("no How element with id {id:?} to edit")
}

#[cfg(test)]
#[path = "framework_write_handlers_tests.rs"]
mod tests;
