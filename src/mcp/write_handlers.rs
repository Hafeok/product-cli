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
    let config = ProductConfig::load_from_root(repo_root)
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
        due_date: None,
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
    let config = ProductConfig::load_from_root(repo_root)
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
    let config = ProductConfig::load_from_root(repo_root)
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

/// FT-066: `product_feature_link` writes the feature side **and** the
/// reciprocal back-reference (TC's `validates.features` / ADR's
/// `features`) in one atomic batch. The response carries a `writes`
/// array enumerating every file touched and a `reciprocated` array
/// naming each back-reference filled in. Unknown link targets return a
/// `NotFound` error before any write.
pub(crate) fn handle_feature_link(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();

    apply_optional_depends_on(args, id, graph)?;

    let adr = args.get("adr").and_then(|v| v.as_str());
    let test = args.get("test").and_then(|v| v.as_str());

    let plan = crate::feature::plan_link(graph, id, adr, test)
        .map_err(|e| format!("{}", e))?;
    crate::feature::apply_link(&plan).map_err(|e| format!("{}", e))?;

    Ok(build_link_response(id, &plan))
}

/// FT-062 — the optional `feature` argument adds a depends-on edge using the
/// cycle-checked plan helper. Composes with the adr/test link plan below.
fn apply_optional_depends_on(
    args: &Value,
    id: &str,
    graph: &KnowledgeGraph,
) -> Result<(), String> {
    let Some(dep_id) = args.get("feature").and_then(|v| v.as_str()) else {
        return Ok(());
    };
    let dep_plan = crate::feature::plan_depends_on_edit(
        graph,
        id,
        std::slice::from_ref(&dep_id.to_string()),
        &[],
    )
    .map_err(|e| format!("{}", e))?;
    if dep_plan.is_changed() {
        crate::feature::apply_depends_on_edit(&dep_plan)
            .map_err(|e| format!("{}", e))?;
    }
    Ok(())
}

fn build_link_response(id: &str, plan: &crate::feature::LinkPlan) -> Value {
    let writes_json: Vec<Value> = plan
        .writes
        .iter()
        .map(|w| {
            serde_json::json!({
                "path": w.path.display().to_string(),
                "kind": w.kind.as_str(),
            })
        })
        .collect();
    let reciprocated_json: Vec<Value> = plan
        .reciprocated
        .iter()
        .map(|r| serde_json::json!({"id": r.id, "field": r.field}))
        .collect();
    serde_json::json!({
        "id": id,
        "writes": writes_json,
        "reciprocated": reciprocated_json,
    })
}

/// FT-066: `product_feature_status` writes the requested status to disk via
/// `feature::plan_status_change` + `apply_status_change`. Propagates
/// `NotFound`, parse errors, and FT-058 `TcRunnerMissing` from the slice
/// layer. The success response is `{ id, status, orphaned-tests: [...] }`
/// — `orphaned-tests` is empty for non-abandonment transitions.
pub(crate) fn handle_feature_status_update(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "id is required".to_string())?;
    let status_str = args
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "status is required".to_string())?;

    let new_status: crate::types::FeatureStatus =
        status_str.parse().map_err(|e: String| format!("E001: {}", e))?;

    let plan = crate::feature::plan_status_change(graph, id, new_status)
        .map_err(|e| format!("{}", e))?;
    crate::feature::apply_status_change(&plan).map_err(|e| format!("{}", e))?;

    let orphaned: Vec<Value> = plan
        .orphaned_tests
        .iter()
        .map(|t| {
            serde_json::json!({
                "test_id": t.test_id,
                "path": t.path.display().to_string(),
            })
        })
        .collect();
    Ok(serde_json::json!({
        "id": id,
        "status": new_status.to_string(),
        "orphaned-tests": orphaned,
    }))
}

/// FT-066: `product_test_status` writes the requested status to disk via
/// `tc::plan_status_change` + `apply_status_change`. Propagates `NotFound`
/// and parse errors from the slice layer.
pub(crate) fn handle_test_status_update(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "id is required".to_string())?;
    let status_str = args
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "status is required".to_string())?;

    let new_status: crate::types::TestStatus =
        status_str.parse().map_err(|e: String| format!("E001: {}", e))?;

    let plan = crate::tc::plan_status_change(graph, id, new_status)
        .map_err(|e| format!("{}", e))?;
    crate::tc::apply_status_change(&plan).map_err(|e| format!("{}", e))?;

    Ok(serde_json::json!({
        "id": id,
        "status": new_status.to_string(),
    }))
}

pub(crate) fn handle_body_update(
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or_default();
    let config = ProductConfig::load_from_root(repo_root)
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


