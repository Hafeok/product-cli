//! MCP handlers for the request-based write surface (FT-041, ADR-038).

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::request::{self, ApplyOptions};
use serde_json::Value;
use std::path::Path;

pub fn handle_request_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let yaml = args
        .get("request_yaml")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'request_yaml' argument".to_string())?;

    let config = ProductConfig::load_from_root(repo_root)
        .map_err(|e| format!("{}", e))?;

    let request = match request::parse_request_str(yaml) {
        Ok(r) => r,
        Err(findings) => return Ok(build_validate_result(&findings)),
    };

    // Reuse apply dry-run for full cross-check
    let result = request::apply_request(
        &request,
        &config,
        repo_root,
        ApplyOptions { dry_run: true, skip_git_identity: true },
    );

    Ok(build_validate_result(&result.findings))
}

pub fn handle_request_apply(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let yaml = args
        .get("request_yaml")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'request_yaml' argument".to_string())?;

    let config = ProductConfig::load_from_root(repo_root)
        .map_err(|e| format!("{}", e))?;

    let request = match request::parse_request_str(yaml) {
        Ok(r) => r,
        Err(findings) => {
            return Ok(serde_json::json!({
                "applied": false,
                "created": [],
                "changed": [],
                "deleted": [],
                "findings": findings,
                "graph_check_clean": false,
            }));
        }
    };

    let result = request::apply_request(
        &request,
        &config,
        repo_root,
        ApplyOptions::default(),
    );

    Ok(serde_json::json!({
        "applied": result.applied,
        "created": result.created,
        "changed": result.changed,
        "deleted": result.deleted,
        "findings": result.findings,
        "graph_check_clean": result.graph_check_clean,
    }))
}

/// FT-064 — convenience MCP tool that wraps a `type: delete` request.
/// Accepts a list of artifact IDs and a reason, builds the YAML in memory,
/// and runs it through the same apply pipeline as `product_request_apply`.
pub fn handle_request_delete(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let ids: Vec<String> = args
        .get("ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing 'ids' argument (array of artifact IDs)".to_string())?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    if ids.is_empty() {
        return Err("'ids' must be a non-empty array of artifact IDs".to_string());
    }
    let reason = args
        .get("reason")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'reason' argument".to_string())?;
    if reason.trim().is_empty() {
        return Err("'reason' must not be empty".to_string());
    }

    let escaped_reason = reason.replace('\\', "\\\\").replace('"', "\\\"");
    let mut yaml = format!(
        "type: delete\nschema-version: 1\nreason: \"{}\"\ndeletions:\n",
        escaped_reason
    );
    for id in &ids {
        yaml.push_str(&format!("  - target: {}\n", id));
    }

    let mut wrapped = serde_json::Map::new();
    wrapped.insert("request_yaml".into(), Value::String(yaml));
    handle_request_apply(&Value::Object(wrapped), repo_root)
}

fn build_validate_result(findings: &[request::Finding]) -> Value {
    let valid = !findings.iter().any(|f| f.is_error());
    serde_json::json!({
        "valid": valid,
        "findings": findings,
    })
}

// Helper no-op to silence unused KnowledgeGraph import when grep-refactoring
#[allow(dead_code)]
fn _unused(_: &KnowledgeGraph) {}
