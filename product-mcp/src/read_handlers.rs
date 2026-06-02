//! MCP read tool handlers — query-only tool implementations.

use crate::gap;
use crate::graph::KnowledgeGraph;
use serde_json::Value;
use std::path::Path;

pub(crate) fn handle_responsibility(repo_root: &Path) -> Result<Value, String> {
    let config = crate::config::ProductConfig::load_from_root(repo_root)
        .map_err(|e| format!("{}", e))?;
    match config.responsibility() {
        Some(responsibility) => Ok(serde_json::json!({
            "name": config.product_name(),
            "responsibility": responsibility,
        })),
        None => Err("Product responsibility is not configured. Add a [product] section with a responsibility field to the product config file".to_string()),
    }
}

pub(crate) fn handle_context(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let explicit_target = args.get("target").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Feature path with target template (FT-063).
    //
    // Selection priority: explicit `target` → `[context].default-target`
    // → `human` (fallback). The fallback to `human` matches the CLI
    // selection rule and the FT-063 invariant that an omitted target
    // resolves to the `human` template. The synthetic `legacy` target
    // routes through the pre-FT-063 AISP-framed bundler.
    if graph.features.contains_key(id) {
        let config = crate::config::ProductConfig::load_from_root(repo_root)
            .map_err(|e| format!("{}", e))?;
        let target_name: String = explicit_target
            .or_else(|| config.context.default_target.clone())
            .unwrap_or_else(|| "human".to_string());
        if target_name == "legacy" {
            let pi = config.responsibility().map(|resp| crate::context::BundleProductInfo {
                product_name: config.product_name(),
                responsibility: resp,
            });
            let bundle = crate::context::bundle_feature_with_product(graph, id, depth, true, pi)
                .ok_or_else(|| format!("Feature {} not found", id))?;
            return Ok(serde_json::json!({
                "content": bundle,
                "type": "text",
            }));
        }
        let outcome = crate::context::template::resolve_all(repo_root);
        let resolved = match outcome.resolved.get(&target_name) {
            Some(t) => t.clone(),
            None => {
                let mut available: Vec<String> = outcome.resolved.keys().cloned().collect();
                available.sort();
                return Err(format!(
                    "error[E027]: unknown context target {:?}; available: {}",
                    target_name,
                    available.join(", "),
                ));
            }
        };
        let pi = config.responsibility().map(|resp| crate::context::template::ProductInfo {
            name: config.product_name(),
            responsibility: resp,
        });
        let rendered = crate::context::template::render_feature(graph, id, depth, &resolved, pi)
            .ok_or_else(|| format!("Feature {} not found", id))?;
        return Ok(serde_json::json!({
            "format": rendered.format,
            "target": rendered.target,
            "content": rendered.content,
            "token_count_approx": rendered.token_count_approx,
            "exceeded_target_max": rendered.exceeded_target_max,
            "exceeded_hard_max": rendered.exceeded_hard_max,
            "type": "text",
        }));
    }

    // ADR fallback (legacy bundle).
    let bundle = crate::context::bundle_adr(graph, id, depth);
    Ok(serde_json::json!({
        "content": bundle.unwrap_or_default(),
        "type": "text",
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
    // FT-071: accept an `include` parameter — `"patterns"` opts PAT nodes
    // into the ranking. The default preserves the legacy ADR-only result.
    let include = args.get("include").and_then(|v| v.as_str()).unwrap_or("");
    let include_patterns = include == "patterns";
    let centrality = graph.betweenness_centrality_with(include_patterns);
    let mut ranked: Vec<_> = graph.adrs.keys()
        .map(|id| {
            let c = centrality.get(id).copied().unwrap_or(0.0);
            let title = graph.adrs.get(id).map(|a| a.front.title.as_str()).unwrap_or("");
            serde_json::json!({"id": id, "centrality": c, "title": title, "kind": "ADR"})
        })
        .collect();
    if include_patterns {
        for p in graph.patterns.values() {
            let c = centrality.get(&p.front.id).copied().unwrap_or(0.0);
            ranked.push(serde_json::json!({
                "id": p.front.id,
                "centrality": c,
                "title": p.front.title,
                "kind": "PAT",
            }));
        }
    }
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
        "direct_adrs": impact.direct_adrs,
        "direct_deps": impact.direct_deps,
        "direct_patterns": impact.direct_patterns,
        "transitive_features": impact.transitive_features,
        "transitive_tests": impact.transitive_tests,
    }))
}

pub(crate) fn handle_schema(args: &Value) -> Result<Value, String> {
    let artifact_type = args
        .get("artifact_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let content = if artifact_type.is_empty() {
        crate::agent_context::generate_all_schemas()
    } else {
        crate::agent_context::generate_schema(artifact_type)?
    };

    // FT-062 — every response carries the canonical field allowlists so an
    // agent can introspect what `product_request_apply` will accept. Same
    // source `field_schema::known_fields_for_label` the request validator
    // consults — they cannot diverge.
    let fields = if artifact_type.is_empty() {
        serde_json::json!({
            "feature": crate::field_schema::FEATURE_FIELDS,
            "adr": crate::field_schema::ADR_FIELDS,
            "tc": crate::field_schema::TC_FIELDS,
            "dep": crate::field_schema::DEP_FIELDS,
        })
    } else {
        let key = match artifact_type {
            "test" => "tc",
            "dependency" => "dep",
            other => other,
        };
        serde_json::json!({
            key: crate::field_schema::known_fields_for_label(key),
        })
    };

    Ok(serde_json::json!({
        "content": content,
        "type": "text",
        "fields": fields,
    }))
}

pub(crate) fn handle_agent_context(
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let config = crate::config::ProductConfig::load_from_root(repo_root)
        .map_err(|e| format!("{}", e))?;
    let content = crate::agent_context::generate_agent_md(&config, graph, repo_root);
    Ok(serde_json::json!({
        "content": content,
        "type": "text"
    }))
}

fn resolve_prompts_path(repo_root: &Path) -> String {
    let cfg_path = match crate::config::find_config_in_dir(repo_root) {
        Some(p) => p,
        None => return "benchmarks/prompts".to_string(),
    };
    match crate::config::ProductConfig::load(&cfg_path) {
        Ok(c) => c.paths.prompts_resolved().to_string(),
        Err(_) => "benchmarks/prompts".to_string(),
    }
}

pub(crate) fn handle_prompts_list(repo_root: &Path) -> Result<Value, String> {
    let prompts_path = resolve_prompts_path(repo_root);
    let prompts = crate::author::prompts_list(repo_root, &prompts_path);
    let items: Vec<Value> = prompts
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "filename": p.filename,
                "version": p.version,
                "path": p.path,
            })
        })
        .collect();
    Ok(serde_json::json!({"prompts": items}))
}

pub(crate) fn handle_prompts_get(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let prompts_path = resolve_prompts_path(repo_root);
    let content = crate::author::prompts_get(repo_root, &prompts_path, name).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({
        "name": name,
        "content": content,
        "type": "text"
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
