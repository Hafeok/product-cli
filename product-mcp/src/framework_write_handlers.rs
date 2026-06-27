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

use crate::pf_mcp::{load_yaml, pdir, req_str};

fn how_path(repo_root: &Path) -> PathBuf {
    pdir(repo_root).join("how-contract.yaml")
}

fn load_how(repo_root: &Path) -> Result<HowContract, String> {
    load_yaml(&pdir(repo_root), "how-contract", HowContract::from_yaml)
}

fn save_how(repo_root: &Path, c: &HowContract) -> Result<(), String> {
    std::fs::create_dir_all(pdir(repo_root)).map_err(|e| format!("{e}"))?;
    let text = c.to_yaml().map_err(|e| format!("{e}"))?;
    product_core::fileops::write_file_atomic(&how_path(repo_root), &text).map_err(|e| format!("{e}"))
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
fn finish(repo_root: &Path, c: &HowContract, id: &str, element: Value) -> Result<Value, String> {
    save_how(repo_root, c)?;
    let violations = validate_how(c);
    let ok = !violations.iter().any(|v| v.severity == "violation");
    Ok(json!({ "ok": ok, "id": id, "element": element, "violations": violations }))
}

/// Scaffold a fresh How contract keyed to an archetype (or the `product` arg).
pub fn handle_how_init(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let archetype = req_str(args, "archetype").or_else(|_| req_str(args, "product"))?;
    let path = how_path(repo_root);
    if path.exists() {
        return Err(format!(
            "how-contract already exists at {} — edit it with product_how_add / product_how_set",
            path.display()
        ));
    }
    let c = HowContract::scaffold(&archetype);
    save_how(repo_root, &c)?;
    Ok(json!({ "ok": true, "created": path.display().to_string(), "archetype": archetype }))
}

/// Add a Why-cascade element or a contract part to the How contract.
pub fn handle_how_add(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let element = req_str(args, "element")?;
    let id = req_str(args, "id")?;
    let mut c = load_how(repo_root)?;
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
    finish(repo_root, &c, &id, added)
}

/// Set a singleton contract (the application or infrastructure contract).
pub fn handle_how_set(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let target = req_str(args, "target")?;
    let id = req_str(args, "id")?;
    let mut c = load_how(repo_root)?;
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
        other => {
            return Err(format!(
                "unknown target '{other}' — one of app-contract | infra-contract"
            ))
        }
    };
    finish(repo_root, &c, &id, set)
}

#[cfg(test)]
#[path = "framework_write_handlers_tests.rs"]
mod tests;
