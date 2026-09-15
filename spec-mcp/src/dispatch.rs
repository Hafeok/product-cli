//! Tool dispatch, with the withheld verbs refused by name.
//!
//! Belt and braces on purpose: the registry does not carry a withheld verb, so
//! it cannot be reached through `tools/list`, and the dispatcher refuses it
//! anyway. A boundary with one guard is a boundary that survives exactly one
//! careless edit.

use std::path::Path;

use chrono::Utc;
use serde_json::{json, Value};
use spec_core::{gate, map, metrics, store};

use crate::tools::WITHHELD;

/// Handle one `spec_*` call, or `None` when the name is not ours.
pub fn dispatch(name: &str, args: &Value, root: &Path) -> Option<Result<Value, String>> {
    if WITHHELD.contains(&name) {
        return Some(Err(format!(
            "`{name}` names a principal, and a model cannot be one. \
             Draft the record and hand it to a person to run at the CLI."
        )));
    }
    match name {
        "spec_candidates" => Some(candidates(args, root)),
        "spec_map" => Some(joined(root)),
        "spec_check" => Some(check(root)),
        "spec_records" => Some(records(args, root)),
        "spec_policy_show" => Some(policy_show(root)),
        "spec_implement" => Some(implement(args, root)),
        _ => None,
    }
}

fn candidates(args: &Value, root: &Path) -> Result<Value, String> {
    let spec = load(root)?;
    let unreviewed_only = args.get("unreviewed").and_then(Value::as_bool).unwrap_or(false);
    let Some(inventory) = &spec.inventory else {
        return Ok(json!({
            "candidates": [],
            "note": "no inventory — run `specflow import` first"
        }));
    };
    let listed: Vec<Value> = inventory
        .candidates
        .iter()
        .filter(|c| !unreviewed_only || spec.is_unreviewed(&c.id))
        .map(|c| json!({
            "candidate": c.id,
            "entry_point": c.entry_point,
            "observed": c.observed,
            "unfilled_slots": c.unfilled_slots,
            "unreviewed": spec.is_unreviewed(&c.id),
        }))
        .collect();
    Ok(json!({
        "candidates": listed,
        "note": "Ratifying one is `spec accept`, at the CLI, by a person. \
                 Draft the name and what it settles; do not claim to have filed them."
    }))
}

fn joined(root: &Path) -> Result<Value, String> {
    let spec = load(root)?;
    Ok(json!({"findings": map::join(&spec)}))
}

fn check(root: &Path) -> Result<Value, String> {
    let spec = load(root)?;
    let structural = gate::judge_store(&spec);
    Ok(json!({
        "structural": structural.iter().map(|f| json!({
            "class": f.class.to_string(),
            "subject": f.record,
            "message": f.message,
        })).collect::<Vec<_>>(),
        "policy": gate::run_policy(&spec),
        "metrics": metrics::compute(&spec),
    }))
}

fn records(args: &Value, root: &Path) -> Result<Value, String> {
    let spec = load(root)?;
    let open_only = args.get("open").and_then(Value::as_bool).unwrap_or(false);
    let listed: Vec<Value> = spec
        .records
        .iter()
        .filter(|r| !open_only || r.is_open())
        .map(|r| json!({
            "record": r.id,
            "slice": r.slice,
            "act_ref": r.act_ref,
            "open": r.is_open(),
        }))
        .collect();
    Ok(json!({"records": listed}))
}

fn policy_show(root: &Path) -> Result<Value, String> {
    let spec = load(root)?;
    match spec.policy_in_force() {
        Ok(None) => Ok(json!({
            "in_force": null,
            "note": "no policy filed — structural verdicts only"
        })),
        Ok(Some(policy)) => Ok(json!({"in_force": policy})),
        Err(tips) => Err(format!(
            "the policy chain forks at {} — only a recorded supersession settles it",
            tips.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

fn implement(args: &Value, root: &Path) -> Result<Value, String> {
    let slice = required(args, "slice")?;
    let act_ref = required(args, "act_ref")?;
    let by = args.get("by").and_then(Value::as_str).unwrap_or("agent@example.invalid");

    let opening = spec_core::record::Opening {
        id: ledger_core_mint(),
        act: "implement".to_string(),
        slice: slice.to_string(),
        act_ref: act_ref.to_string(),
        opened_at: Utc::now(),
        opened_by: by.parse().map_err(|e: String| e)?,
        base_revision: head_revision(root),
    };
    let record = store::open(root, opening).map_err(|e| format!("{e}"))?;
    Ok(json!({
        "record": record.id,
        "slice": record.slice,
        "act_ref": record.act_ref,
        "status": "pending-closure",
        "hand_off": format!(
            "spec close {} --principal <you@example.com> --nothing-arose",
            record.id
        ),
        "note": "The record is OPEN. Closing it names a principal and is not yours to do — \
                 give the hand_off command to a person."
    }))
}

fn required<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("`{key}` is required"))
}

fn load(root: &Path) -> Result<gate::SpecStore, String> {
    store::load_store(root).map_err(|e| format!("{e}"))
}

fn ledger_core_mint() -> String {
    ledger_core::mint::UlidMint::system().mint()
}

fn head_revision(root: &Path) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

#[path = "dispatch_tests.rs"]
#[cfg(test)]
mod tests;
