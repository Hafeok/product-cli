//! MCP field management tool handlers (FT-038)
//!
//! Handlers for domain, scope, supersession, source-files, runner,
//! and acknowledgement mutations via the MCP server.

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use serde_json::Value;
use std::path::Path;

// ---------------------------------------------------------------------------
// Feature domain management
// ---------------------------------------------------------------------------

pub(crate) fn handle_feature_domain(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let add = extract_string_array(args, "add");
    let remove = extract_string_array(args, "remove");

    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;

    for domain in &add {
        if !config.domains.contains_key(domain) {
            return Err(format!("E012: unknown domain '{}'. Check [domains] in product.toml", domain));
        }
    }

    let mut front = f.front.clone();
    for domain in &add {
        if !front.domains.contains(domain) {
            front.domains.push(domain.clone());
        }
    }
    for domain in &remove {
        front.domains.retain(|d| d != domain);
    }
    front.domains.sort();

    let content = crate::parser::render_feature(&front, &f.body);
    crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "domains": front.domains}))
}

// ---------------------------------------------------------------------------
// Feature acknowledgement
// ---------------------------------------------------------------------------

pub(crate) fn handle_feature_acknowledge(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let domain = args.get("domain").and_then(|v| v.as_str()).unwrap_or_default();
    let remove = args.get("remove").and_then(|v| v.as_bool()).unwrap_or(false);

    let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;
    let mut front = f.front.clone();

    if remove {
        front.domains_acknowledged.remove(domain);
    } else {
        let reason = args.get("reason").and_then(|v| v.as_str()).unwrap_or_default();
        if reason.trim().is_empty() {
            return Err("E011: acknowledgement requires non-empty reason".to_string());
        }
        front.domains_acknowledged.insert(domain.to_string(), reason.to_string());
    }

    let content = crate::parser::render_feature(&front, &f.body);
    crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "domain": domain, "removed": remove}))
}

// ---------------------------------------------------------------------------
// ADR domain management
// ---------------------------------------------------------------------------

pub(crate) fn handle_adr_domain(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let add = extract_string_array(args, "add");
    let remove = extract_string_array(args, "remove");

    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;

    for domain in &add {
        if !config.domains.contains_key(domain) {
            return Err(format!("E012: unknown domain '{}'. Check [domains] in product.toml", domain));
        }
    }

    let mut front = a.front.clone();
    for domain in &add {
        if !front.domains.contains(domain) {
            front.domains.push(domain.clone());
        }
    }
    for domain in &remove {
        front.domains.retain(|d| d != domain);
    }
    front.domains.sort();

    let content = crate::parser::render_adr(&front, &a.body);
    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "domains": front.domains}))
}

// ---------------------------------------------------------------------------
// ADR scope
// ---------------------------------------------------------------------------

pub(crate) fn handle_adr_scope(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let scope_str = args.get("scope").and_then(|v| v.as_str()).unwrap_or_default();

    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;
    let scope: crate::types::AdrScope = scope_str.parse()
        .map_err(|e: String| format!("E001: {}", e))?;

    let mut front = a.front.clone();
    front.scope = scope;

    let content = crate::parser::render_adr(&front, &a.body);
    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "scope": scope.to_string()}))
}

// ---------------------------------------------------------------------------
// ADR supersession (bidirectional)
// ---------------------------------------------------------------------------

pub(crate) fn handle_adr_supersede(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let supersedes = args.get("supersedes").and_then(|v| v.as_str());
    let remove = args.get("remove").and_then(|v| v.as_str());

    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;

    if let Some(target_id) = supersedes {
        supersede_add_mcp(id, target_id, a, graph)
    } else if let Some(target_id) = remove {
        supersede_remove_mcp(id, target_id, a, graph)
    } else {
        Err("must specify 'supersedes' or 'remove' parameter".to_string())
    }
}

fn supersede_add_mcp(
    id: &str,
    target_id: &str,
    a: &crate::types::Adr,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let target = graph.adrs.get(target_id)
        .ok_or_else(|| format!("E002: ADR {} not found", target_id))?;

    let mut new_front = a.front.clone();
    if !new_front.supersedes.contains(&target_id.to_string()) {
        new_front.supersedes.push(target_id.to_string());
    }

    let mut target_front = target.front.clone();
    if !target_front.superseded_by.contains(&id.to_string()) {
        target_front.superseded_by.push(id.to_string());
    }

    // Cycle detection
    let mut test_adrs: Vec<crate::types::Adr> = graph.adrs.values().cloned().collect();
    test_adrs.retain(|ai| ai.front.id != id && ai.front.id != target_id);
    test_adrs.push(crate::types::Adr {
        front: new_front.clone(), body: a.body.clone(), path: a.path.clone(),
    });
    test_adrs.push(crate::types::Adr {
        front: target_front.clone(), body: target.body.clone(), path: target.path.clone(),
    });
    let test_graph = KnowledgeGraph::build(vec![], test_adrs, vec![]);
    if let Some(cycle) = test_graph.detect_supersession_cycle() {
        return Err(format!("E004: supersession cycle detected: {}", cycle.join(" -> ")));
    }

    if target_front.status == crate::types::AdrStatus::Accepted {
        target_front.status = crate::types::AdrStatus::Superseded;
    }

    let content_a = crate::parser::render_adr(&new_front, &a.body);
    let content_target = crate::parser::render_adr(&target_front, &target.body);
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&a.path, &content_a),
        (&target.path, &content_target),
    ];
    crate::fileops::write_batch_atomic(&writes).map_err(|e| format!("{}", e))?;

    Ok(serde_json::json!({"id": id, "supersedes": target_id, "action": "added"}))
}

fn supersede_remove_mcp(
    id: &str,
    target_id: &str,
    a: &crate::types::Adr,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let target = graph.adrs.get(target_id)
        .ok_or_else(|| format!("E002: ADR {} not found", target_id))?;

    let mut new_front = a.front.clone();
    new_front.supersedes.retain(|s| s != target_id);

    let mut target_front = target.front.clone();
    target_front.superseded_by.retain(|s| s != id);

    let content_a = crate::parser::render_adr(&new_front, &a.body);
    let content_target = crate::parser::render_adr(&target_front, &target.body);
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&a.path, &content_a),
        (&target.path, &content_target),
    ];
    crate::fileops::write_batch_atomic(&writes).map_err(|e| format!("{}", e))?;

    Ok(serde_json::json!({"id": id, "remove": target_id, "action": "removed"}))
}

// ---------------------------------------------------------------------------
// ADR source files
// ---------------------------------------------------------------------------

pub(crate) fn handle_adr_source_files(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let add = extract_string_array(args, "add");
    let remove = extract_string_array(args, "remove");

    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;

    let mut warnings = Vec::new();
    for path_str in &add {
        let full_path = repo_root.join(path_str);
        if !full_path.exists() {
            warnings.push(format!("W012: path '{}' does not exist (yet)", path_str));
        }
    }

    let mut front = a.front.clone();
    for path_str in &add {
        if !front.source_files.contains(path_str) {
            front.source_files.push(path_str.clone());
        }
    }
    for path_str in &remove {
        front.source_files.retain(|s| s != path_str);
    }
    front.source_files.sort();

    let content = crate::parser::render_adr(&front, &a.body);
    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({
        "id": id,
        "source_files": front.source_files,
        "warnings": warnings,
    }))
}

// ---------------------------------------------------------------------------
// Test runner configuration
// ---------------------------------------------------------------------------

pub(crate) fn handle_test_runner(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let runner = args.get("runner").and_then(|v| v.as_str());
    let test_args = args.get("args").and_then(|v| v.as_str());
    let timeout = args.get("timeout").and_then(|v| v.as_str());
    let requires = extract_string_array(args, "requires");

    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let t = graph.tests.get(id).ok_or_else(|| format!("TC {} not found", id))?;

    let valid_runners = ["cargo-test", "bash", "pytest", "custom"];
    let mut front = t.front.clone();

    if let Some(r) = runner {
        if !valid_runners.contains(&r) {
            return Err(format!(
                "E001: unknown runner '{}'. Valid values: {}",
                r,
                valid_runners.join(", ")
            ));
        }
        front.runner = Some(r.to_string());
    }

    if let Some(a) = test_args {
        front.runner_args = Some(a.to_string());
    }

    if let Some(t_str) = timeout {
        let secs = t_str.trim_end_matches('s').parse::<u64>()
            .map_err(|_| format!("invalid timeout: {}", t_str))?;
        front.runner_timeout = Some(secs);
    }

    for req in &requires {
        if !config.verify.prerequisites.contains_key(req) {
            return Err(format!(
                "E001: unknown prerequisite '{}'. Check [verify.prerequisites] in product.toml",
                req
            ));
        }
        if !front.requires.contains(req) {
            front.requires.push(req.clone());
        }
    }

    let content = crate::parser::render_test(&front, &t.body);
    crate::fileops::write_file_atomic(&t.path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({
        "id": id,
        "runner": front.runner,
        "runner_args": front.runner_args,
        "runner_timeout": front.runner_timeout,
        "requires": front.requires,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract an array of strings from a JSON field (or empty vec if missing)
fn extract_string_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
