//! MCP read handlers for `product_dep_*` — parity with `product dep` (ADR-030).
//!
//! Dependencies live in the legacy FT/ADR/TC knowledge graph, so these take the
//! loaded `graph` (not the `.product/` framework artifacts).

use product_core::graph::KnowledgeGraph;
use serde_json::{json, Value};

use crate::pf_mcp::req_str;

pub fn handle_dep_list(_args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let mut deps: Vec<&product_core::types::Dependency> = graph.dependencies.values().collect();
    deps.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    let items: Vec<Value> = deps
        .iter()
        .map(|d| json!({ "id": d.front.id, "title": d.front.title, "status": d.front.status }))
        .collect();
    Ok(json!({ "dependencies": items }))
}

pub fn handle_dep_show(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let dep = graph.dependencies.get(&id).ok_or_else(|| format!("no dependency '{id}'"))?;
    serde_json::to_value(&dep.front).map_err(|e| format!("{e}"))
}

pub fn handle_dep_features(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let dep = graph.dependencies.get(&id).ok_or_else(|| format!("no dependency '{id}'"))?;
    Ok(json!({ "id": id, "features": dep.front.features }))
}

#[cfg(test)]
#[path = "dep_handlers_tests.rs"]
mod tests;
