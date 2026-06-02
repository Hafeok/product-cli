//! Parse a request YAML document into `Request` (FT-041, ADR-038, FT-064).
//!
//! Top-level orchestration. Section parsers live in sibling modules:
//! `parse_artifacts`, `parse_changes`, `parse_deletions`.

use super::parse_artifacts::parse_artifacts_array;
use super::parse_changes::parse_changes_array;
use super::parse_deletions::parse_deletions_array;
use super::types::*;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub fn parse_request(path: &Path) -> Result<Request, Vec<Finding>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        vec![Finding::error(
            "E001",
            format!("failed to read request file {}: {}", path.display(), e),
            "$",
        )]
    })?;
    parse_request_str(&content)
}

/// Closed set of recognised top-level request keys. Any other key surfaces
/// as **E025 unknown-request-key** (FT-062, FT-064).
const KNOWN_TOP_LEVEL_KEYS: &[&str] = &[
    "type",
    "schema-version",
    "reason",
    "artifacts",
    "changes",
    "deletions",
];

pub fn parse_request_str(yaml: &str) -> Result<Request, Vec<Finding>> {
    let doc: Value = serde_yaml::from_str(yaml)
        .map_err(|e| vec![Finding::error("E001", format!("malformed YAML: {}", e), "$")])?;
    let map = doc.as_mapping().ok_or_else(|| {
        vec![Finding::error("E001", "request document must be a YAML mapping", "$")]
    })?;
    let mut unknown = check_unknown_top_level_keys(map);
    let request_type = combine(&mut unknown, parse_type(map))?;
    let schema_version = combine(&mut unknown, parse_schema_version(map))?;
    let reason = parse_reason(map);
    let artifacts = combine(&mut unknown, parse_artifacts_array(map))?;
    let changes = combine(&mut unknown, parse_changes_array(map))?;
    let deletions = combine(&mut unknown, parse_deletions_array(map))?;
    if !unknown.is_empty() {
        return Err(unknown);
    }
    Ok(Request {
        request_type,
        schema_version,
        reason,
        artifacts,
        changes,
        deletions,
        source_yaml: yaml.to_string(),
    })
}

/// Merge an accumulator of pending findings with a fallible parser result.
/// On error, the parser's findings are appended to the accumulator and the
/// combined list is returned as a single error.
fn combine<T>(
    pending: &mut Vec<Finding>,
    result: Result<T, Vec<Finding>>,
) -> Result<T, Vec<Finding>> {
    match result {
        Ok(v) => Ok(v),
        Err(mut errs) => {
            errs.append(pending);
            Err(errs)
        }
    }
}

/// Emit **E025** for every top-level key not in `KNOWN_TOP_LEVEL_KEYS`.
fn check_unknown_top_level_keys(map: &Mapping) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (k, _v) in map.iter() {
        if let Some(name) = k.as_str() {
            if !KNOWN_TOP_LEVEL_KEYS.contains(&name) {
                findings.push(Finding::error(
                    "E025",
                    format!(
                        "unknown top-level key '{}' in request — expected one of: {}",
                        name,
                        KNOWN_TOP_LEVEL_KEYS.join(", ")
                    ),
                    format!("$.{}", name),
                ));
            }
        }
    }
    findings
}

fn parse_type(map: &Mapping) -> Result<RequestType, Vec<Finding>> {
    let type_val = map.get(Value::String("type".into())).and_then(|v| v.as_str());
    match type_val {
        Some("create") => Ok(RequestType::Create),
        Some("change") => Ok(RequestType::Change),
        Some("create-and-change") => Ok(RequestType::CreateAndChange),
        Some("delete") => Ok(RequestType::Delete),
        Some(other) => Err(vec![Finding::error(
            "E001",
            format!(
                "unknown request type '{}' — expected one of: create, change, create-and-change, delete",
                other
            ),
            "$.type",
        )]),
        None => Err(vec![Finding::error(
            "E001",
            "missing required field 'type'",
            "$.type",
        )]),
    }
}

fn parse_schema_version(map: &Mapping) -> Result<u32, Vec<Finding>> {
    let version = match map.get(Value::String("schema-version".into())) {
        None => CURRENT_REQUEST_SCHEMA,
        Some(Value::Number(n)) => match n.as_u64() {
            Some(v) if v <= u32::MAX as u64 => v as u32,
            _ => {
                return Err(vec![Finding::error(
                    "E001",
                    "schema-version must be a non-negative integer",
                    "$.schema-version",
                )])
            }
        },
        Some(_) => {
            return Err(vec![Finding::error(
                "E001",
                "schema-version must be an integer",
                "$.schema-version",
            )])
        }
    };
    if version != CURRENT_REQUEST_SCHEMA {
        return Err(vec![Finding::error(
            "E001",
            format!(
                "unsupported schema-version {} — this binary supports schema-version {}",
                version, CURRENT_REQUEST_SCHEMA
            ),
            "$.schema-version",
        )
        .with_hint(format!(
            "this request was written for schema v{}; upgrade Product, or rewrite the request for schema v{}",
            version, CURRENT_REQUEST_SCHEMA
        ))]);
    }
    Ok(version)
}

fn parse_reason(map: &Mapping) -> String {
    match map.get(Value::String("reason".into())) {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    }
}
