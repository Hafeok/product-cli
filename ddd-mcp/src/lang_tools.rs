//! M3 language-intelligence handlers — normalized LSP pass-through.
//!
//! Every handler routes by adapter, surfaces host loading as an explicit
//! `{status: "loading"}` result (never a hang, never a fake answer —
//! PRD §11), and returns plain JSON the agent can consume. Diagnostics
//! join to the `.ddd/` manifests on the same `(namespace, rule_id)` key
//! the SARIF path uses (PRD §7).

use std::path::Path;

use ddd_lsp::host::{Host, Readiness};
use ddd_lsp::protocol::{
    flatten_symbols, kind_name, normalize_locations, position_params, to_uri,
};
use serde_json::{json, Value};

use crate::state::{opt_str, opt_u64, req_str, ServeState};

type ToolResult = Result<Value, String>;

/// Run `f` on the ready host for the request's language/file; a loading
/// host short-circuits into the explicit loading result.
pub fn with_ready_host(
    state: &ServeState,
    args: &Value,
    file: Option<&Path>,
    f: impl FnOnce(&mut Host) -> ToolResult,
) -> ToolResult {
    let language = opt_str(args, "language");
    let mut manager = state.manager.lock().map_err(|_| "host manager poisoned".to_string())?;
    let adapter =
        manager.resolve_adapter(language.as_deref(), file).map_err(|e| e.to_string())?;
    if !adapter.hosted {
        return Err(format!(
            "`{}` has no language server — the pair adapter classifies edits from source text; use ddd_apply_edit",
            adapter.language
        ));
    }
    let host = manager.host(adapter.language).map_err(|e| e.to_string())?;
    match host.readiness().map_err(|e| e.to_string())? {
        Readiness::Ready { .. } => f(host),
        loading => Ok(json!({
            "status": "loading",
            "language": adapter.language,
            "readiness": serde_json::to_value(&loading).unwrap_or(Value::Null),
            "notes": host.notes(),
        })),
    }
}

fn position_of(args: &Value) -> Result<(u64, u64), String> {
    let line = opt_u64(args, "line").ok_or("missing required argument `line`")?;
    let character = opt_u64(args, "character").ok_or("missing required argument `character`")?;
    Ok((line, character))
}

/// `ddd_find_symbol`: documentSymbol for a file, workspace/symbol for a query.
pub fn find_symbol(state: &ServeState, args: &Value) -> ToolResult {
    let file = opt_str(args, "file").map(|f| state.resolve_file(&f));
    let query = opt_str(args, "query");
    if file.is_none() && query.is_none() {
        return Err("give `file` (document symbols) or `query` (workspace search)".to_string());
    }
    with_ready_host(state, args, file.as_deref(), |host| {
        let result = match &file {
            Some(path) => {
                host.ensure_open(path).map_err(|e| e.to_string())?;
                host.request(
                    "textDocument/documentSymbol",
                    json!({"textDocument": {"uri": to_uri(path)}}),
                )
                .map_err(|e| e.to_string())?
            }
            None => host
                .request("workspace/symbol", json!({"query": query}))
                .map_err(|e| e.to_string())?,
        };
        let symbols: Vec<Value> = flatten_symbols(&result)
            .into_iter()
            .map(|s| {
                json!({
                    "name": s.name, "kind": kind_name(s.kind), "container": s.container,
                    "line": s.sel_line, "character": s.sel_char,
                })
            })
            .collect();
        Ok(json!({"status": "ok", "symbols": symbols}))
    })
}

/// `ddd_references`: locations plus the count the seam metadata wants.
pub fn references(state: &ServeState, args: &Value) -> ToolResult {
    let file = state.resolve_file(&req_str(args, "file")?);
    let (line, character) = position_of(args)?;
    with_ready_host(state, args, Some(&file), |host| {
        host.ensure_open(&file).map_err(|e| e.to_string())?;
        let mut params = position_params(&file, line, character);
        params["context"] = json!({"includeDeclaration": true});
        let result = host.request("textDocument/references", params).map_err(|e| e.to_string())?;
        let locations: Vec<Value> = normalize_locations(&result)
            .into_iter()
            .map(|l| json!({"file": l.file, "line": l.line, "character": l.character}))
            .collect();
        Ok(json!({"status": "ok", "count": locations.len(), "references": locations}))
    })
}

/// `ddd_hover`: the hover contents as one markdown string.
pub fn hover(state: &ServeState, args: &Value) -> ToolResult {
    let file = state.resolve_file(&req_str(args, "file")?);
    let (line, character) = position_of(args)?;
    with_ready_host(state, args, Some(&file), |host| {
        host.ensure_open(&file).map_err(|e| e.to_string())?;
        let result = host
            .request("textDocument/hover", position_params(&file, line, character))
            .map_err(|e| e.to_string())?;
        Ok(json!({"status": "ok", "contents": hover_text(&result)}))
    })
}

fn hover_text(result: &Value) -> String {
    let contents = result.get("contents").unwrap_or(&Value::Null);
    match contents {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("value").and_then(Value::as_str).unwrap_or("").to_string(),
        Value::Array(items) => items
            .iter()
            .map(|i| match i {
                Value::String(s) => s.clone(),
                other => other.get("value").and_then(Value::as_str).unwrap_or("").to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// `ddd_signature`: normalized signature help.
pub fn signature(state: &ServeState, args: &Value) -> ToolResult {
    let file = state.resolve_file(&req_str(args, "file")?);
    let (line, character) = position_of(args)?;
    with_ready_host(state, args, Some(&file), |host| {
        host.ensure_open(&file).map_err(|e| e.to_string())?;
        let result = host
            .request("textDocument/signatureHelp", position_params(&file, line, character))
            .map_err(|e| e.to_string())?;
        let signatures: Vec<Value> = result
            .get("signatures")
            .and_then(Value::as_array)
            .map(|sigs| {
                sigs.iter()
                    .map(|s| json!({"label": s.get("label").and_then(Value::as_str).unwrap_or("")}))
                    .collect()
            })
            .unwrap_or_default();
        Ok(json!({"status": "ok", "signatures": signatures}))
    })
}

/// `ddd_rename`: compute the workspace edit; application goes through
/// `ddd_apply_edit` so the interceptor sees every mutation.
pub fn rename(state: &ServeState, args: &Value) -> ToolResult {
    let file = state.resolve_file(&req_str(args, "file")?);
    let (line, character) = position_of(args)?;
    let new_name = req_str(args, "new_name")?;
    with_ready_host(state, args, Some(&file), |host| {
        host.ensure_open(&file).map_err(|e| e.to_string())?;
        let mut params = position_params(&file, line, character);
        params["newName"] = json!(new_name);
        let result = host.request("textDocument/rename", params).map_err(|e| e.to_string())?;
        Ok(json!({
            "status": "computed",
            "applied": false,
            "note": "apply each file's edits through ddd_apply_edit — mutations only land through the interceptor",
            "workspace_edit": workspace_edit_files(&result),
        }))
    })
}

/// Normalize a WorkspaceEdit's `changes` map into per-file edit lists.
fn workspace_edit_files(result: &Value) -> Vec<Value> {
    let mut files = Vec::new();
    if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            files.push(json!({
                "file": ddd_lsp::protocol::from_uri(uri),
                "edits": edits,
            }));
        }
    }
    files
}

/// `ddd_warmup`: start hosts, optionally block for readiness.
pub fn warmup(state: &ServeState, args: &Value) -> ToolResult {
    let languages: Option<Vec<String>> = args
        .get("languages")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect());
    let wait_ms = opt_u64(args, "wait_ms").unwrap_or(0);
    let mut manager = state.manager.lock().map_err(|_| "host manager poisoned".to_string())?;
    let mut rows = Vec::new();
    for (language, first) in manager.warm_all(languages.as_deref()) {
        let readiness = match first {
            Ok(r) if wait_ms > 0 => manager
                .host(&language)
                .and_then(|h| h.wait_ready(std::time::Duration::from_millis(wait_ms)))
                .unwrap_or(r),
            Ok(r) => r,
            Err(e) => {
                rows.push(json!({"language": language, "error": e.to_string()}));
                continue;
            }
        };
        let notes = manager.host(&language).map(|h| h.notes()).unwrap_or_default();
        rows.push(json!({
            "language": language,
            "readiness": serde_json::to_value(&readiness).unwrap_or(Value::Null),
            "notes": notes,
        }));
    }
    Ok(json!({"status": "ok", "hosts": rows}))
}

/// `ddd_diagnostics`: pull (or fall back to published) diagnostics, each
/// joined to its manifest namespace by rule id — the SARIF join.
pub fn diagnostics(state: &ServeState, args: &Value) -> ToolResult {
    let file = state.resolve_file(&req_str(args, "file")?);
    let store = ddd_core::store::load(&state.root.join(ddd_core::store::STORE_DIR));
    with_ready_host(state, args, Some(&file), |host| {
        host.ensure_open(&file).map_err(|e| e.to_string())?;
        let items = pull_or_published(host, &file)?;
        let joined: Vec<Value> = items.iter().map(|d| join_governance(&store, d)).collect();
        Ok(json!({"status": "ok", "diagnostics": joined}))
    })
}

/// Pull diagnostics; hosts that only push get a short published-state poll.
fn pull_or_published(host: &mut Host, file: &Path) -> Result<Vec<Value>, String> {
    let pulled = host
        .request("textDocument/diagnostic", json!({"textDocument": {"uri": to_uri(file)}}))
        .ok()
        .and_then(|r| r.get("items").and_then(Value::as_array).cloned());
    if let Some(items) = pulled {
        if !items.is_empty() {
            return Ok(items);
        }
    }
    for _ in 0..20 {
        if let Some(params) = host.published_diagnostics(file).map_err(|e| e.to_string())? {
            return Ok(params
                .get("diagnostics")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(Vec::new())
}

/// The M2 join: `(namespace_for(source, rule_id), rule_id)` → manifest
/// entry → decision | UNGOVERNED | no entry.
fn join_governance(store: &ddd_core::DddStore, d: &Value) -> Value {
    let rule_id = d
        .get("code")
        .map(|c| match c {
            Value::String(s) => s.clone(),
            Value::Object(o) => {
                o.get("value").and_then(Value::as_str).unwrap_or_default().to_string()
            }
            other => other.to_string(),
        })
        .unwrap_or_default();
    let source = d.get("source").and_then(Value::as_str).unwrap_or_default();
    let namespace = ddd_core::detect::namespace_for(source, &rule_id);
    let entry = store
        .manifests
        .iter()
        .find(|s| s.name == namespace)
        .and_then(|s| s.file.rules.iter().find(|r| r.rule_id == rule_id));
    let governance = match entry {
        Some(rule) => match (&rule.decision, &rule.governance) {
            (Some(d), _) => json!({"decision": d, "resolve": format!("ddd why {rule_id}")}),
            (None, Some(g)) => json!({"marker": g}),
            (None, None) => json!({"marker": "no decision (ontology rule 4)"}),
        },
        None => json!({"marker": "no manifest entry — UNGOVERNED if it fires (run ddd diff)"}),
    };
    json!({
        "rule_id": rule_id,
        "namespace": namespace,
        "message": d.get("message").cloned().unwrap_or(Value::Null),
        "severity": d.get("severity").cloned().unwrap_or(Value::Null),
        "range": d.get("range").cloned().unwrap_or(Value::Null),
        "source": source,
        "governance": governance,
    })
}
