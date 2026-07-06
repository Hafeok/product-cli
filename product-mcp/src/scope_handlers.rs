//! MCP handlers for `product_scope_*` — parity with the §14 scope commands.
//!
//! Authoring scopes are an intake / What concept: a tool declares which
//! What-element kinds it may author. These handlers vendor a scope, re-validate
//! a stored one, run the §14.3 enforcement oracle over a submission, or run the
//! §14.4 completeness join across every stored scope. Storage mirrors the CLI:
//! `<product base>/authoring-scopes/<tool>.yaml`.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use product_core::pf::authoring_scope::{validate_scope, AuthoringScope};
use product_core::pf::authoring_scope_enforce::{enforce, Submission};
use product_core::pf::authoring_scope_join::completeness_join;
use serde_json::{json, Value};

use crate::pf_mcp::{pbase, req_str};

fn scopes_dir(base: &Path) -> PathBuf {
    base.join("authoring-scopes")
}

fn stems(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

fn load(dir: &Path, tool: &str) -> Result<AuthoringScope, String> {
    let path = dir.join(format!("{tool}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no authoring scope '{tool}' at {}", path.display()))?;
    AuthoringScope::from_yaml(&text).map_err(|e| format!("{e}"))
}

/// Reject an absolute or parent-escaping relative path.
fn safe_rel(rel: &str) -> Result<(), String> {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return Err(format!("path '{rel}' must be relative and stay inside the repo"));
    }
    Ok(())
}

fn finding_messages(scope: &AuthoringScope) -> Vec<String> {
    validate_scope(scope).into_iter().map(|v| v.message).collect()
}

pub fn handle_scope_list(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let dir = scopes_dir(&pbase(args, repo_root));
    Ok(json!({ "scopes": stems(&dir) }))
}

pub fn handle_scope_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let dir = scopes_dir(&pbase(args, repo_root));
    let scope = load(&dir, &req_str(args, "tool")?)?;
    serde_json::to_value(&scope).map_err(|e| format!("{e}"))
}

pub fn handle_scope_validate(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let dir = scopes_dir(&pbase(args, repo_root));
    let tool = req_str(args, "tool")?;
    let scope = load(&dir, &tool)?;
    let findings = finding_messages(&scope);
    Ok(json!({ "ok": findings.is_empty(), "tool": tool, "findings": findings }))
}

pub fn handle_scope_enforce(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let dir = scopes_dir(&pbase(args, repo_root));
    let scope = load(&dir, &req_str(args, "tool")?)?;
    let rel = req_str(args, "submission_path")?;
    safe_rel(&rel)?;
    let text = std::fs::read_to_string(repo_root.join(&rel))
        .map_err(|e| format!("{rel}: {e}"))?;
    let submission = Submission::from_json(&text)?;
    let (valid, findings) = enforce(&scope, &submission);
    Ok(json!({ "tool": scope.tool, "valid": valid, "findings": findings }))
}

pub fn handle_scope_join(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let dir = scopes_dir(&pbase(args, repo_root));
    let required: Vec<String> = args
        .get("required")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    if required.is_empty() {
        return Err("`required` must be a non-empty array of kinds".to_string());
    }
    let scopes: Vec<AuthoringScope> = stems(&dir).iter().filter_map(|t| load(&dir, t).ok()).collect();
    let authored = parse_authored(args);
    let (complete, report) = completeness_join(&required, &scopes, &authored);
    Ok(json!({
        "complete": complete,
        "scopes": scopes.iter().map(|s| s.tool.clone()).collect::<Vec<_>>(),
        "report": report,
    }))
}

fn parse_authored(args: &Value) -> BTreeMap<String, HashSet<String>> {
    let mut out: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    if let Some(map) = args.get("authored").and_then(|v| v.as_object()) {
        for (tool, kinds) in map {
            let set = kinds
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            out.insert(tool.clone(), set);
        }
    }
    out
}

pub fn handle_scope_add(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let rel = req_str(args, "file")?;
    safe_rel(&rel)?;
    let text = std::fs::read_to_string(repo_root.join(&rel)).map_err(|e| format!("{rel}: {e}"))?;
    let scope = AuthoringScope::from_yaml(&text).map_err(|e| format!("{e}"))?;
    let findings = finding_messages(&scope);
    if !findings.is_empty() {
        return Err(format!(
            "authoring scope '{}' is not whole (§14.2) — nothing saved:\n  - {}",
            scope.tool,
            findings.join("\n  - ")
        ));
    }
    let dir = scopes_dir(&pbase(args, repo_root));
    let path = dir.join(format!("{}.yaml", scope.tool));
    let yaml = scope.to_yaml().map_err(|e| format!("{e}"))?;
    product_core::fileops::write_file_atomic(&path, &yaml).map_err(|e| format!("{e}"))?;
    Ok(json!({
        "ok": true,
        "tool": scope.tool,
        "adapter": scope.adapter,
        "authors": scope.authors.len(),
        "excluded": scope.excluded.len(),
    }))
}
