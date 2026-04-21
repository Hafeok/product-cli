//! Shared helpers for the per-artifact `add` planners.

use super::draft::Draft;
use crate::config::ProductConfig;
use crate::request::{
    parse_request_str,
    validate::{check_dep_governance, validate_request, ValidationContext},
    Finding,
};
use serde_yaml::Value;
use std::collections::HashMap;

/// Append one artifact to the draft and run incremental validation. On any
/// E-class finding, revert the draft to its pre-append state.
pub fn append_artifact_transactional(
    draft: &mut Draft,
    m: serde_yaml::Mapping,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Result<Vec<Finding>, Vec<Finding>> {
    let snapshot = draft.to_yaml();
    draft.artifacts_mut().push(Value::Mapping(m));
    let findings = validate_draft(draft, config, graph);
    if findings.iter().any(|f| f.is_error()) {
        let _ = restore_from_yaml(draft, &snapshot);
        return Err(findings);
    }
    Ok(findings)
}

pub fn restore_from_yaml(draft: &mut Draft, snapshot: &str) -> Result<(), serde_yaml::Error> {
    let v: Value = serde_yaml::from_str(snapshot)?;
    if let Value::Mapping(m) = v {
        draft.doc = m;
    }
    Ok(())
}

/// Run the full request validator against the current in-memory draft.
///
/// This uses the same `validate_request` + `check_dep_governance` pair the
/// apply pipeline uses — structural validation only, no I/O. Forward refs
/// to artifacts not yet added are tolerated so arbitrary `add` ordering
/// works; the submit-time validator is the final gate.
pub fn validate_draft(
    draft: &Draft,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> Vec<Finding> {
    let yaml = draft.to_yaml();
    let request = match parse_request_str(&yaml) {
        Ok(r) => r,
        Err(findings) => return findings,
    };
    let ctx = ValidationContext { config, graph };
    let mut findings: Vec<Finding> = validate_request(&request, &ctx)
        .into_iter()
        .filter(|f| {
            if f.code == "E011" && f.location == "$.reason" {
                return false;
            }
            if f.code == "E002" && f.message.contains("not defined in request") {
                return false;
            }
            true
        })
        .collect();
    let mut refs: HashMap<String, (crate::request::ArtifactType, usize)> = HashMap::new();
    for a in &request.artifacts {
        if let Some(ref n) = a.ref_name {
            refs.entry(n.clone()).or_insert((a.artifact_type, a.index));
        }
    }
    check_dep_governance(&request, &refs, graph, &mut findings);
    findings
}

/// Generate a unique, spec-compliant `ref:` name for a new artifact.
///
/// The grammar is `^[a-z][a-z0-9-]*$`. We slug the title and prefix with the
/// artifact-type marker, then dedup against existing refs already in the
/// draft.
pub fn make_ref_name(prefix: &str, title: &str, draft: &Draft) -> String {
    let slug = slugify(title);
    let base = if slug.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}-{slug}")
    };
    let mut candidate = base.clone();
    let mut n = 2;
    while ref_exists(draft, &candidate) {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    candidate
}

fn ref_exists(draft: &Draft, name: &str) -> bool {
    draft.artifacts().iter().any(|a| {
        a.as_mapping()
            .and_then(|m| m.get(Value::String("ref".into())))
            .and_then(|v| v.as_str())
            == Some(name)
    })
}

fn slugify(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_dash = true;
    for ch in title.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// If the input looks like a ref name (matches the grammar and is not an ID)
/// turn it into `ref:<name>`, otherwise return it as-is (likely a real ID).
pub fn resolve_id_or_ref(s: &str) -> Value {
    if s.starts_with("ref:") {
        return Value::String(s.to_string());
    }
    if s.contains('-') && s.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
        // Looks like a real ID (FT-001, ADR-002, ...) — pass through.
        return Value::String(s.to_string());
    }
    Value::String(format!("ref:{s}"))
}
