//! MCP write tool handlers — mutation tool implementations.

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use serde_json::Value;
use std::path::Path;

pub(crate) fn handle_feature_new(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
    let phase = args.get("phase").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let existing: Vec<String> = graph.features.keys().cloned().collect();
    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let id = crate::parser::next_id(&config.prefixes.feature, &existing);
    let filename = crate::parser::id_to_filename(&id, title);
    let dir = config.resolve_path(repo_root, &config.paths.features);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
    let path = dir.join(&filename);
    let front = new_feature_front(id.clone(), title, phase);
    let body = format!("## Description\n\n[Describe {} here.]\n", title);
    let content = crate::parser::render_feature(&front, &body);
    crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
}

fn new_feature_front(id: String, title: &str, phase: u32) -> crate::types::FeatureFrontMatter {
    crate::types::FeatureFrontMatter {
        id,
        title: title.to_string(),
        phase,
        status: crate::types::FeatureStatus::Planned,
        depends_on: vec![],
        adrs: vec![],
        tests: vec![],
        domains: vec![],
        domains_acknowledged: std::collections::HashMap::new(),
        bundle: None,
    }
}

pub(crate) fn handle_adr_new(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
    let existing: Vec<String> = graph.adrs.keys().cloned().collect();
    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let id = crate::parser::next_id(&config.prefixes.adr, &existing);
    let filename = crate::parser::id_to_filename(&id, title);
    let dir = config.resolve_path(repo_root, &config.paths.adrs);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
    let path = dir.join(&filename);
    let front = new_adr_front(id.clone(), title);
    let body = "**Status:** Proposed\n\n**Context:**\n\n**Decision:**\n\n**Rationale:**\n\n**Rejected alternatives:**\n".to_string();
    let content = crate::parser::render_adr(&front, &body);
    crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
}

fn new_adr_front(id: String, title: &str) -> crate::types::AdrFrontMatter {
    crate::types::AdrFrontMatter {
        id,
        title: title.to_string(),
        status: crate::types::AdrStatus::Proposed,
        features: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        domains: vec![],
        scope: crate::types::AdrScope::Domain,
        content_hash: None,
        amendments: vec![],
        source_files: vec![],
        removes: vec![],
        deprecates: vec![],
    }
}

pub(crate) fn handle_test_new(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
    let test_type = args.get("test_type").and_then(|v| v.as_str()).unwrap_or("scenario");
    let existing: Vec<String> = graph.tests.keys().cloned().collect();
    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let id = crate::parser::next_id(&config.prefixes.test, &existing);
    let filename = crate::parser::id_to_filename(&id, title);
    let dir = config.resolve_path(repo_root, &config.paths.tests);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
    let path = dir.join(&filename);
    let front = new_test_front(id.clone(), title, test_type);
    let body = "## Description\n\n[Describe test here.]\n".to_string();
    let content = crate::parser::render_test(&front, &body);
    crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
    Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
}

fn new_test_front(
    id: String,
    title: &str,
    test_type: &str,
) -> crate::types::TestFrontMatter {
    let tt: crate::types::TestType = test_type.parse().unwrap_or(crate::types::TestType::Scenario);
    crate::types::TestFrontMatter {
        id,
        title: title.to_string(),
        test_type: tt,
        status: crate::types::TestStatus::Unimplemented,
        validates: crate::types::ValidatesBlock { features: vec![], adrs: vec![] },
        phase: 1,
        content_hash: None,
        runner: None,
        runner_args: None,
        runner_timeout: None,
        requires: vec![],
        last_run: None,
        failure_message: None,
        last_run_duration: None,
    }
}

pub(crate) fn handle_feature_link(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;
    let mut front = f.front.clone();
    let mut changed = false;
    if let Some(adr_id) = args.get("adr").and_then(|v| v.as_str()) {
        if !front.adrs.contains(&adr_id.to_string()) {
            front.adrs.push(adr_id.to_string());
            changed = true;
        }
    }
    if let Some(test_id) = args.get("test").and_then(|v| v.as_str()) {
        if !front.tests.contains(&test_id.to_string()) {
            front.tests.push(test_id.to_string());
            changed = true;
        }
    }
    if changed {
        let content = crate::parser::render_feature(&front, &f.body);
        crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))?;
    }
    Ok(serde_json::json!({"id": id, "linked": changed}))
}

pub(crate) fn handle_status_update(args: &Value) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let status = args.get("status").and_then(|v| v.as_str()).unwrap_or_default();
    Ok(serde_json::json!({"id": id, "status": status, "note": "Use CLI for status updates with full side-effects"}))
}

pub(crate) fn handle_body_update(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or_default();
    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    if id.starts_with(&config.prefixes.feature) {
        update_feature_body(id, body, graph)?;
    } else if id.starts_with(&config.prefixes.adr) {
        update_adr_body(id, body, graph)?;
    } else if id.starts_with(&config.prefixes.test) {
        update_test_body(id, body, graph)?;
    } else if id.starts_with(&config.prefixes.dependency) {
        update_dep_body(id, body, graph)?;
    } else {
        return Err(format!("Unknown artifact ID prefix: {}", id));
    }
    Ok(serde_json::json!({"id": id, "updated": true}))
}

fn update_feature_body(id: &str, body: &str, graph: &KnowledgeGraph) -> Result<(), String> {
    let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;
    let content = crate::parser::render_feature(&f.front, body);
    crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))
}

fn update_adr_body(id: &str, body: &str, graph: &KnowledgeGraph) -> Result<(), String> {
    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;
    // ADR-032: Protect accepted ADR body from modification via MCP
    if a.front.status == crate::types::AdrStatus::Accepted {
        return Err(format!(
            "Cannot modify body of accepted ADR {}. Use `product adr amend {} --reason \"...\"` instead.",
            id, id
        ));
    }
    let content = crate::parser::render_adr(&a.front, body);
    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))
}

fn update_test_body(id: &str, body: &str, graph: &KnowledgeGraph) -> Result<(), String> {
    let t = graph.tests.get(id).ok_or_else(|| format!("TC {} not found", id))?;
    let content = crate::parser::render_test(&t.front, body);
    crate::fileops::write_file_atomic(&t.path, &content).map_err(|e| format!("{}", e))
}

fn update_dep_body(id: &str, body: &str, graph: &KnowledgeGraph) -> Result<(), String> {
    let d = graph
        .dependencies
        .get(id)
        .ok_or_else(|| format!("Dep {} not found", id))?;
    let content = crate::parser::render_dependency(&d.front, body);
    crate::fileops::write_file_atomic(&d.path, &content).map_err(|e| format!("{}", e))
}


