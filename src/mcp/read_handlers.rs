//! MCP read tool handlers — query-only tool implementations.

use crate::gap;
use crate::graph::KnowledgeGraph;
use serde_json::Value;
use std::path::Path;

pub(crate) fn handle_context(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let bundle = if graph.features.contains_key(id) {
        crate::context::bundle_feature(graph, id, depth, true)
    } else {
        crate::context::bundle_adr(graph, id, depth)
    };
    Ok(serde_json::json!({
        "content": bundle.unwrap_or_default(),
        "type": "text"
    }))
}

pub(crate) fn handle_feature_list(graph: &KnowledgeGraph) -> Result<Value, String> {
    let mut items: Vec<Value> = graph.features.values()
        .map(|f| serde_json::json!({
            "id": f.front.id,
            "title": f.front.title,
            "phase": f.front.phase,
            "status": format!("{}", f.front.status),
        }))
        .collect();
    items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    Ok(serde_json::json!(items))
}

pub(crate) fn handle_feature_show(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    match graph.features.get(id) {
        Some(f) => Ok(serde_json::json!({
            "id": f.front.id,
            "title": f.front.title,
            "phase": f.front.phase,
            "status": format!("{}", f.front.status),
            "depends_on": f.front.depends_on,
            "adrs": f.front.adrs,
            "tests": f.front.tests,
            "body": f.body,
        })),
        None => Err(format!("Feature {} not found", id)),
    }
}

pub(crate) fn handle_feature_deps(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let feat = graph.features.get(id)
        .ok_or_else(|| format!("Feature {} not found", id))?;
    let depends_on: Vec<Value> = feat.front.depends_on.iter()
        .filter_map(|dep_id| graph.features.get(dep_id.as_str()).map(|df| {
            serde_json::json!({"id": dep_id, "title": df.front.title, "status": format!("{}", df.front.status)})
        }))
        .collect();
    let depended_by: Vec<Value> = graph.features.values()
        .filter(|f| f.front.depends_on.iter().any(|d| d == id))
        .map(|f| serde_json::json!({"id": f.front.id, "title": f.front.title, "status": format!("{}", f.front.status)}))
        .collect();
    Ok(serde_json::json!({"id": id, "depends_on": depends_on, "depended_by": depended_by}))
}

pub(crate) fn handle_adr_list(graph: &KnowledgeGraph) -> Result<Value, String> {
    let mut items: Vec<Value> = graph.adrs.values()
        .map(|a| serde_json::json!({
            "id": a.front.id,
            "title": a.front.title,
            "status": format!("{}", a.front.status),
        }))
        .collect();
    items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    Ok(serde_json::json!(items))
}

pub(crate) fn handle_adr_show(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    match graph.adrs.get(id) {
        Some(a) => Ok(serde_json::json!({
            "id": a.front.id,
            "title": a.front.title,
            "status": format!("{}", a.front.status),
            "body": a.body,
        })),
        None => Err(format!("ADR {} not found", id)),
    }
}

pub(crate) fn handle_test_show(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    match graph.tests.get(id) {
        Some(t) => Ok(serde_json::json!({
            "id": t.front.id,
            "title": t.front.title,
            "type": format!("{}", t.front.test_type),
            "status": format!("{}", t.front.status),
            "validates": {
                "features": t.front.validates.features,
                "adrs": t.front.validates.adrs,
            },
            "phase": t.front.phase,
            "body": t.body,
        })),
        None => Err(format!("Test criterion {} not found", id)),
    }
}

pub(crate) fn handle_graph_central(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let top = args.get("top").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let centrality = graph.betweenness_centrality();
    let mut ranked: Vec<_> = graph.adrs.keys()
        .map(|id| {
            let c = centrality.get(id).copied().unwrap_or(0.0);
            let title = graph.adrs.get(id).map(|a| a.front.title.as_str()).unwrap_or("");
            serde_json::json!({"id": id, "centrality": c, "title": title})
        })
        .collect();
    ranked.sort_by(|a, b| {
        b["centrality"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["centrality"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked.truncate(top);
    Ok(serde_json::json!(ranked))
}

pub(crate) fn handle_impact(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let impact = graph.impact(id);
    Ok(serde_json::json!({
        "seed": impact.seed,
        "direct_features": impact.direct_features,
        "direct_tests": impact.direct_tests,
        "transitive_features": impact.transitive_features,
        "transitive_tests": impact.transitive_tests,
    }))
}

pub(crate) fn handle_gap_check(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let baseline = gap::GapBaseline::load(&repo_root.join("gaps.json"));
    let adr_id = args.get("adr_id").and_then(|v| v.as_str());
    let findings = if let Some(id) = adr_id {
        gap::check_adr(graph, id, &baseline)
    } else {
        let reports = gap::check_all(graph, &baseline);
        reports.into_iter().flat_map(|r| r.findings).collect()
    };
    Ok(serde_json::json!(findings))
}
