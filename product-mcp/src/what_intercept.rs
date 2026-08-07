//! Write-time interception for What edits (`dec/ddd/what-policy-table`).
//!
//! Every `product_domain_*` write already runs under the repo lock, so the
//! interceptor applies the edit, classifies the before and after graphs,
//! and restores its snapshot when a boundary appears with nothing declared
//! for it. Nothing outside the lock observes the intermediate state, so a
//! rejection leaves no trace on disk.
//!
//! Interception is inert unless the repo carries a `.ddd/` store; the mode
//! comes from `intercept_by_class.specification`, falling back to the
//! global `intercept`.

use std::path::{Path, PathBuf};

use ddd_core::what::{self, WhatElement};
use serde_json::{json, Value};

/// The artifact class What edits are governed under.
pub const ARTIFACT_CLASS: &str = "specification";

/// The tools whose writes touch the What graph.
const INTERCEPTED: &[&str] = &["product_domain_new", "product_domain_edit", "product_domain_rm"];

/// State captured before the edit, so it can be judged and undone.
pub struct Guard {
    mode: String,
    snapshot: Vec<(PathBuf, Option<String>)>,
    before: Vec<WhatElement>,
    product: String,
}

/// One boundary the edit formed, with the row that decided it.
#[derive(Debug, Clone)]
struct Formed {
    element: WhatElement,
    change: &'static str,
}

/// Snapshot and classify, or `None` when interception does not apply.
pub fn before(name: &str, args: &Value, repo_root: &Path) -> Option<Guard> {
    if !INTERCEPTED.contains(&name) {
        return None;
    }
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let cfg = store.config.as_ref()?;
    let mode = cfg.intercept_for(ARTIFACT_CLASS).to_string();
    if mode == "off" {
        return None;
    }
    let product = product_of(args, repo_root)?;
    let dir = product_core::author::domain::session_dir(repo_root, &product);
    let before = graph_of(&dir).map(|g| what::classify(&g)).unwrap_or_default();
    Some(Guard { mode, snapshot: snapshot(&dir, &product), before, product })
}

/// Judge the applied edit; restore the snapshot when enforcing a demand.
pub fn after(
    guard: Guard,
    outcome: Result<Value, String>,
    repo_root: &Path,
) -> Result<Value, String> {
    let Ok(value) = outcome else { return outcome };
    let dir = product_core::author::domain::session_dir(repo_root, &guard.product);
    let after = graph_of(&dir).map(|g| what::classify(&g)).unwrap_or_default();
    let formed = formed_boundaries(&guard.before, &after);
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let undeclared: Vec<&Formed> = formed
        .iter()
        .filter(|f| governing(&store, &f.element.id).is_none())
        .collect();
    if undeclared.is_empty() {
        return Ok(value);
    }
    if guard.mode == "warn" {
        let mut out = value;
        annotate(&mut out, &undeclared);
        return Ok(out);
    }
    restore(&guard.snapshot);
    Ok(json!({
        "ok": false,
        "status": "rejected",
        "reason": "the edit forms a contract surface in the What with no declaration naming it",
        "mode": guard.mode,
        "demands": undeclared.iter().map(|f| demand(f)).collect::<Vec<_>>(),
        "next": "file product_what_declare for each surface named below, then retry this edit",
    }))
}

/// The surface a demand names, plus the declaration template to fill in.
fn demand(f: &Formed) -> Value {
    let el = &f.element;
    json!({
        "surface": {
            "kind": el.kind,
            "element": el.id,
            "container": el.container,
            "visibility": el.visibility,
            "signature": el.signature,
            "change": f.change,
            "rule": el.rule,
            "rule_claim": el.rule_claim,
        },
        "required": "a seam declaration (product_what_declare)",
        "template": {
            "id": format!("seam/what/{}", el.id),
            "element": el.id,
            "boundary": format!("{} {}", el.kind, el.id),
            // Never pre-filled: the judgment is authored
            // (`dec/ddd/rejection-facts-prefilled`).
            "verdict_knowledge": "",
        },
    })
}

fn annotate(value: &mut Value, undeclared: &[&Formed]) {
    let Value::Object(map) = value else { return };
    map.insert("mode".into(), json!("warn"));
    map.insert(
        "warning".into(),
        json!(format!(
            "{} contract surface(s) formed with no declaration — declare the seam with product_what_declare",
            undeclared.len()
        )),
    );
    map.insert("surface".into(), json!(undeclared.iter().map(|f| demand(f)).collect::<Vec<_>>()));
}

/// Boundaries the edit formed: newly added, newly published, or a payload
/// change on something already surface.
fn formed_boundaries(before: &[WhatElement], after: &[WhatElement]) -> Vec<Formed> {
    let mut out = Vec::new();
    for el in after.iter().filter(|e| e.surface) {
        let prior = before.iter().find(|b| b.kind == el.kind && b.id == el.id);
        let change = match prior {
            None => "added",
            // The qualifier case: a Translation started carrying it.
            Some(p) if !p.surface => "published",
            Some(p) if p.signature != el.signature => "signature-changed",
            Some(_) => continue,
        };
        out.push(Formed { element: el.clone(), change });
    }
    out
}

fn governing(store: &ddd_core::DddStore, element_id: &str) -> Option<String> {
    let want = what::what_ref(element_id);
    store.seams.iter().find(|s| s.contract_location == want).map(|s| s.id.clone())
}

/// The product the edit targets: the `product` arg, else the repo default.
fn product_of(args: &Value, repo_root: &Path) -> Option<String> {
    if let Some(p) = args.get("product").and_then(Value::as_str).filter(|s| !s.trim().is_empty()) {
        return Some(p.to_string());
    }
    let cfg = product_core::config::ProductConfig::load_from_root(repo_root).ok()?;
    Some(cfg.name.trim().to_string()).filter(|n| !n.is_empty())
}

fn graph_of(dir: &Path) -> Option<product_core::pf::model::DomainGraph> {
    product_core::pf::session::DomainSession::load(dir).ok().map(|s| s.graph)
}

/// The two files `DomainSession::save` writes; `None` means absent, which
/// restore honours by removing what the edit created.
fn snapshot(dir: &Path, product: &str) -> Vec<(PathBuf, Option<String>)> {
    [dir.join("session.json"), dir.join(format!("{product}.ttl"))]
        .into_iter()
        .map(|p| {
            let prior = std::fs::read_to_string(&p).ok();
            (p, prior)
        })
        .collect()
}

fn restore(snapshot: &[(PathBuf, Option<String>)]) {
    for (path, prior) in snapshot {
        match prior {
            Some(text) => {
                let _ = product_core::fileops::write_file_atomic(path, text);
            }
            None => {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
#[path = "what_intercept_tests.rs"]
mod tests;
