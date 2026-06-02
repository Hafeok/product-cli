//! MCP pattern tool handlers (FT-070, ADR-050).
//!
//! Each handler routes through the slice in `product_lib::pattern` so the
//! MCP surface and the CLI surface produce byte-identical files (ADR-020
//! parity invariant — see FT-066 lesson).

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::pattern;
use crate::types;
use serde_json::Value;
use std::path::Path;

pub(crate) fn handle_pattern_new(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let config = ProductConfig::load_from_root(repo_root).map_err(|e| format!("{}", e))?;
    let existing: Vec<String> = graph.patterns.keys().cloned().collect();
    let plan = pattern::plan_create(title, &existing, &config.prefixes.pattern, &config.patterns)
        .map_err(|e| format!("{}", e))?;
    let target_dir = config.resolve_path(repo_root, &config.paths.patterns);
    let path = pattern::apply_create(&plan, &target_dir).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({
        "id": plan.id,
        "path": path.display().to_string(),
    }))
}

pub(crate) fn handle_pattern_status(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let status_str = args
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let new_status: types::PatternStatus = status_str.parse().map_err(|e: String| e)?;
    let deprecated_by = args
        .get("deprecated_by")
        .and_then(|v| v.as_str());
    let plan = pattern::plan_status_change(graph, &graph.patterns, id, new_status, deprecated_by)
        .map_err(|e| format!("{}", e))?;
    pattern::apply_status_change(&plan).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({
        "id": plan.pattern_id,
        "status": plan.new_status.to_string(),
        "previous-status": plan.previous_status.to_string(),
        "deprecated-by": plan.deprecated_by,
    }))
}

pub(crate) fn handle_pattern_link(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let adr = args.get("adr").and_then(|v| v.as_str());
    let requires = args.get("requires").and_then(|v| v.as_str());
    let example = args.get("example").and_then(|v| v.as_str());
    let plan = pattern::plan_link(graph, &graph.patterns, id, adr, requires, example)
        .map_err(|e| format!("{}", e))?;
    pattern::apply_link(&plan).map_err(|e| format!("{}", e))?;
    let writes: Vec<Value> = plan
        .writes
        .iter()
        .map(|w| {
            serde_json::json!({
                "path": w.path.display().to_string(),
                "kind": w.kind.as_str(),
            })
        })
        .collect();
    let reciprocated: Vec<Value> = plan
        .reciprocated
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "field": r.field,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "id": plan.pattern_id,
        "writes": writes,
        "reciprocated": reciprocated,
    }))
}

pub(crate) fn handle_pattern_list(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let want: Option<types::PatternStatus> = match args.get("status").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => Some(s.parse().map_err(|e: String| e)?),
        _ => None,
    };
    let mut patterns: Vec<&types::Pattern> = graph.patterns.values().collect();
    if let Some(w) = want {
        patterns.retain(|p| p.front.status == w);
    }
    patterns.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    let arr: Vec<Value> = patterns
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.front.id,
                "status": p.front.status.to_string(),
                "title": p.front.title,
                "domains": p.front.domains,
            })
        })
        .collect();
    Ok(Value::Array(arr))
}

pub(crate) fn handle_pattern_show(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let p = graph
        .patterns
        .get(id)
        .ok_or_else(|| format!("pattern {} not found", id))?;
    Ok(serde_json::json!({
        "id": p.front.id,
        "title": p.front.title,
        "status": p.front.status.to_string(),
        "domains": p.front.domains,
        "adrs": p.front.adrs,
        "requires": p.front.requires,
        "examples": p.front.examples,
        "deprecated-by": p.front.deprecated_by,
        "body": p.body,
    }))
}
