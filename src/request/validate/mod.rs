//! Request validation (FT-041, ADR-038).
//!
//! Every finding is reported in one pass.

pub mod artifacts;
pub mod changes;
pub mod helpers;

use super::types::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use regex::Regex;
use std::collections::HashMap;

pub struct ValidationContext<'a> {
    pub config: &'a ProductConfig,
    pub graph: &'a KnowledgeGraph,
}

pub fn validate_request(request: &Request, ctx: &ValidationContext<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_reason(request, &mut findings);
    check_section_coherence(request, &mut findings);
    let refs = collect_refs(request, &mut findings);
    for a in &request.artifacts {
        artifacts::validate_artifact(a, &refs, ctx, &mut findings);
    }
    for c in &request.changes {
        changes::validate_change(c, &refs, ctx, &mut findings);
    }
    findings
}

fn check_reason(request: &Request, findings: &mut Vec<Finding>) {
    if request.reason.trim().is_empty() {
        findings.push(Finding::error(
            "E011",
            "request 'reason' is required and must not be empty",
            "$.reason",
        ));
    }
}

fn check_section_coherence(request: &Request, findings: &mut Vec<Finding>) {
    match request.request_type {
        RequestType::Create => {
            if request.artifacts.is_empty() {
                findings.push(Finding::error("E006", "'type: create' requires at least one artifact", "$.artifacts"));
            }
            if !request.changes.is_empty() {
                findings.push(Finding::error("E006", "'type: create' must not contain a 'changes' section", "$.changes"));
            }
        }
        RequestType::Change => {
            if request.changes.is_empty() {
                findings.push(Finding::error("E006", "'type: change' requires at least one change", "$.changes"));
            }
            if !request.artifacts.is_empty() {
                findings.push(Finding::error("E006", "'type: change' must not contain an 'artifacts' section", "$.artifacts"));
            }
        }
        RequestType::CreateAndChange => {
            if request.artifacts.is_empty() && request.changes.is_empty() {
                findings.push(Finding::error(
                    "E006",
                    "'type: create-and-change' requires at least one artifact or change",
                    "$",
                ));
            }
        }
    }
}

fn collect_refs(
    request: &Request,
    findings: &mut Vec<Finding>,
) -> HashMap<String, (ArtifactType, usize)> {
    let ref_re = Regex::new(r"^[a-z][a-z0-9-]*$").expect("constant regex");
    let mut refs: HashMap<String, (ArtifactType, usize)> = HashMap::new();
    for a in &request.artifacts {
        if let Some(ref name) = a.ref_name {
            if !ref_re.is_match(name) {
                findings.push(Finding::error(
                    "E001",
                    format!("invalid ref name '{}' — must match ^[a-z][a-z0-9-]*$", name),
                    format!("$.artifacts[{}].ref", a.index),
                ));
            } else if refs.contains_key(name) {
                findings.push(Finding::error(
                    "E001",
                    format!("duplicate ref name '{}'", name),
                    format!("$.artifacts[{}].ref", a.index),
                ));
            } else {
                refs.insert(name.clone(), (a.artifact_type, a.index));
            }
        }
    }
    refs
}

/// Placeholder hook — currently unused but reserved for cross-artifact cycle checks.
pub fn validate_against_graph(
    _request: &Request,
    _ctx: &ValidationContext<'_>,
) -> Vec<Finding> {
    Vec::new()
}

// Re-exports for the apply pipeline.
pub use helpers::strip_ref_prefix;

/// Post-validation pass: every DEP must have at least one governing ADR (E013)
/// either on its own `adrs` field or referenced by another ADR's `governs` field.
pub fn check_dep_governance(
    request: &Request,
    _refs: &HashMap<String, (ArtifactType, usize)>,
    _graph: &KnowledgeGraph,
    findings: &mut Vec<Finding>,
) {
    use serde_yaml::Value;
    for a in &request.artifacts {
        if a.artifact_type != ArtifactType::Dep {
            continue;
        }
        let own_adrs = a.fields.get(Value::String("adrs".into()));
        let has_own = matches!(own_adrs, Some(Value::Sequence(s)) if !s.is_empty());
        if has_own { continue; }

        let this_ref = a.ref_name.as_deref();
        let mut found = false;
        for other in &request.artifacts {
            if other.artifact_type != ArtifactType::Adr { continue; }
            if let Some(Value::Sequence(seq)) = other.fields.get(Value::String("governs".into())) {
                for item in seq {
                    if let Value::String(s) = item {
                        if let Some(r) = strip_ref_prefix(s) {
                            if Some(r) == this_ref {
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !found {
            findings.push(Finding::error(
                "E013",
                "dependency has no governing ADR in the request — add a 'governs: [ref:...]' on an ADR or declare 'adrs: [ADR-NNN]' on the dep",
                format!("$.artifacts[{}]", a.index),
            ));
        }
    }
}
