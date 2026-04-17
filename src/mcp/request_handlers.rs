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

    let config = ProductConfig::load(&repo_root.join("product.toml"))
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

    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;

    let request = match request::parse_request_str(yaml) {
        Ok(r) => r,
        Err(findings) => {
            return Ok(serde_json::json!({
                "applied": false,
                "created": [],
                "changed": [],
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
        "findings": result.findings,
        "graph_check_clean": result.graph_check_clean,
    }))
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
