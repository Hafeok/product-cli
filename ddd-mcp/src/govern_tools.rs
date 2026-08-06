//! M4 governance handlers — why, graph_query, declare, accept_risk.
//!
//! All of these run over the `.ddd/` store (PRD §8): `why` mirrors the
//! CLI's resolution as structured content; `declare_seam` files the seam
//! (warning when no verdict knowledge is declared); `declare_pattern`
//! fetches the catalog predicate's obligation list — unanswered
//! obligations are rejected; `accept_risk` files the record a
//! suppression must cite (the M2 `UNCITED_SUPPRESSION` check).

use std::collections::BTreeMap;

use ddd_core::store::{load, STORE_DIR};
use serde_json::{json, Value};

use crate::intercept::slug;
use crate::state::{opt_str, req_str, ServeState};

type ToolResult = Result<Value, String>;

/// `ddd_why`: the governance chain behind an id.
pub fn why(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    let store = load(&state.root.join(STORE_DIR));
    match ddd_core::why::render_why(&store, &id) {
        Some(chain) => Ok(json!({"status": "ok", "id": id, "chain": chain})),
        None => Ok(json!({
            "status": "not-found", "id": id,
            "hint": "no decision, claim, manifest rule, pattern, or seam carries this id — an escape if it governs anything (ddd report escapes)",
        })),
    }
}

/// `ddd_graph_query`: read one entry family with simple filters.
pub fn graph_query(state: &ServeState, args: &Value) -> ToolResult {
    let select = req_str(args, "select")?;
    let id = opt_str(args, "id");
    let status = opt_str(args, "status");
    let predicate = opt_str(args, "predicate");
    let store = load(&state.root.join(STORE_DIR));
    let keep = |candidate: &str| id.as_deref().map(|w| w == candidate).unwrap_or(true);
    let entries: Vec<Value> = match select.as_str() {
        "predicates" => to_rows(store.predicates.iter().filter(|p| keep(&p.id))),
        "claims" => to_rows(store.claims.iter().filter(|c| {
            keep(&c.id)
                && status.as_deref().map(|s| c.status.as_str() == s).unwrap_or(true)
                && predicate
                    .as_deref()
                    .map(|p| c.predicate.as_deref() == Some(p))
                    .unwrap_or(true)
        })),
        "decisions" => to_rows(store.decisions.iter().filter(|d| keep(&d.id))),
        "patterns" => to_rows(store.patterns.iter().filter(|p| keep(&p.id))),
        "seams" => to_rows(store.seams.iter().filter(|s| keep(&s.id))),
        "seam-events" => to_rows(store.seam_events.iter().filter(|e| keep(&e.id))),
        "manifests" => store
            .manifests
            .iter()
            .map(|m| json!({"namespace": m.name, "rules": m.file.rules}))
            .collect(),
        other => {
            return Err(format!(
                "unknown selector `{other}` — one of predicates | claims | decisions | manifests | patterns | seams | seam-events"
            ))
        }
    };
    Ok(json!({"status": "ok", "select": select, "count": entries.len(), "entries": entries}))
}

fn to_rows<'a, T: serde::Serialize + 'a>(items: impl Iterator<Item = &'a T>) -> Vec<Value> {
    items.filter_map(|i| serde_json::to_value(i).ok()).collect()
}

/// `ddd_declare_seam`: file the declaration, record it for same-session
/// matching, warn when the boundary absorbs no demand.
pub fn declare_seam(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    if !id.starts_with("seam/") {
        return Err("seam ids are `seam/<area>/<name>`".to_string());
    }
    let boundary = req_str(args, "boundary")?;
    let contract_location = req_str(args, "contract_location")?;
    let verdict_knowledge = opt_str(args, "verdict_knowledge").unwrap_or_default();
    let obligations: Vec<String> = string_list(args, "obligations");
    let symbol = opt_str(args, "symbol");
    let path = state.root.join(STORE_DIR).join("seams").join(format!("{}.yaml", slug(&id)));
    if path.exists() {
        return Err(format!("{id} is already filed at {}", state.rel_display(&path)));
    }
    let mut metadata: BTreeMap<String, serde_yaml::Value> = BTreeMap::new();
    if let Some(s) = &symbol {
        metadata.insert("symbol".to_string(), s.clone().into());
    }
    let seam = ddd_core::seam::SeamDeclaration {
        format: 1,
        id: id.clone(),
        boundary,
        verdict_knowledge: verdict_knowledge.clone(),
        contract_location: contract_location.clone(),
        obligations,
        metadata,
        notes: opt_str(args, "notes"),
    };
    write_yaml(&path, &seam)?;
    state.record_declaration(&id, &contract_location, symbol);
    let warning = verdict_knowledge.trim().is_empty().then_some(
        "verdict_knowledge is empty — this boundary declares seam cost with no demand absorbed (PRD §8); state what the boundary encodes about the verdict",
    );
    Ok(json!({
        "status": "filed", "id": id, "path": state.rel_display(&path), "warning": warning,
    }))
}

/// `ddd_declare_pattern`: obligations come from the catalog predicate;
/// unanswered ones reject the declaration.
pub fn declare_pattern(state: &ServeState, args: &Value) -> ToolResult {
    let id = req_str(args, "id")?;
    if !id.starts_with("pat/") {
        return Err("pattern-instance ids are `pat/<pattern>/<instance>`".to_string());
    }
    let pattern = req_str(args, "pattern")?;
    let instance = req_str(args, "instance")?;
    let store = load(&state.root.join(STORE_DIR));
    let predicate = store
        .predicates
        .iter()
        .find(|p| p.id == pattern)
        .ok_or_else(|| format!("pattern predicate `{pattern}` is not in the catalog (ontology rule 5)"))?;
    let answers = checked_answers(args, &predicate.obligations, &pattern)?;
    let path = state.root.join(STORE_DIR).join("patterns").join(format!("{}.yaml", slug(&id)));
    if path.exists() {
        return Err(format!("{id} is already filed at {}", state.rel_display(&path)));
    }
    let entry = ddd_core::pattern::PatternInstance {
        format: 1,
        id: id.clone(),
        pattern_predicate: pattern,
        instance: instance.clone(),
        obligations: answers,
        decision: None,
        notes: opt_str(args, "notes"),
    };
    write_yaml(&path, &entry)?;
    state.record_declaration(&id, &instance, None);
    Ok(json!({"status": "filed", "id": id, "path": state.rel_display(&path)}))
}

/// The obligation answers, rejected unless every catalog obligation has
/// a non-blank answer (PRD §8).
fn checked_answers(
    args: &Value,
    obligations: &[String],
    pattern: &str,
) -> Result<BTreeMap<String, String>, String> {
    let answers: BTreeMap<String, String> = args
        .get("obligation_answers")
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();
    let unanswered: Vec<&str> = obligations
        .iter()
        .filter(|o| answers.get(*o).map(|a| a.trim().is_empty()).unwrap_or(true))
        .map(String::as_str)
        .collect();
    if unanswered.is_empty() {
        Ok(answers)
    } else {
        Err(format!(
            "unanswered obligations for {pattern}: {} — answer every catalog obligation to declare the instance",
            unanswered.join(", ")
        ))
    }
}

/// `ddd_accept_risk`: the record a suppression cites (ontology rule 4).
pub fn accept_risk(state: &ServeState, args: &Value) -> ToolResult {
    let diagnostic_id = req_str(args, "diagnostic_id")?;
    let rationale = req_str(args, "rationale")?;
    let principal = req_str(args, "principal")?;
    let id = format!("risk/rule/{}", slug(&diagnostic_id));
    let path = state.root.join(STORE_DIR).join("decisions").join(format!("{}.yaml", slug(&id)));
    if path.exists() {
        return Err(format!("{id} is already filed at {}", state.rel_display(&path)));
    }
    let record = ddd_core::decision::Decision {
        format: 1,
        id: id.clone(),
        kind: ddd_core::decision::DecisionKind::RiskAcceptance,
        title: format!("Risk accepted: {diagnostic_id} suppressed"),
        rationale,
        principal,
        based_on: Vec::new(),
        date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        notes: None,
    };
    write_yaml(&path, &record)?;
    Ok(json!({
        "status": "filed", "id": id, "path": state.rel_display(&path),
        "next": format!("cite it from the manifest entry: suppression.risk_acceptance: {id}"),
    }))
}

fn string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn write_yaml<T: serde::Serialize>(path: &std::path::Path, value: &T) -> Result<(), String> {
    let yaml = serde_yaml::to_string(value).map_err(|e| e.to_string())?;
    product_core::fileops::write_file_atomic(path, &yaml).map_err(|e| e.to_string())
}
