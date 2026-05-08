//! Change-spec validation.

use super::super::types::*;
use super::helpers::strip_ref_prefix;
use super::ValidationContext;
use crate::field_schema;
use serde_yaml::Value;
use std::collections::HashMap;

pub fn validate_change(
    c: &ChangeSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    let target_artifact_type = resolve_target_artifact_type(c, refs, ctx);

    if let Some(stripped) = strip_ref_prefix(&c.target) {
        if !refs.contains_key(stripped) {
            findings.push(Finding::error(
                "E002",
                format!("change target 'ref:{}' not defined in request", stripped),
                format!("$.changes[{}].target", c.index),
            ));
        }
    } else if !ctx.graph.all_ids().contains(&c.target) {
        findings.push(Finding::error(
            "E002",
            format!("change target '{}' does not exist in the graph", c.target),
            format!("$.changes[{}].target", c.index),
        ));
    }

    for m in &c.mutations {
        if m.field.trim().is_empty() {
            findings.push(Finding::error(
                "E006",
                "mutation 'field' must not be empty",
                format!("$.changes[{}].mutations[{}].field", c.index, m.index),
            ));
            continue;
        }

        // FT-062 — strict mutation-field validation. The first dot-segment of
        // `mutation.field` must match a known front-matter field for the
        // target's artifact type, or the pseudo-field `body`.
        if let Some(at) = target_artifact_type {
            if !field_schema::is_known_field(at, &m.field) {
                let head = m.field.split('.').next().unwrap_or(&m.field).to_string();
                let suggestion = field_schema::suggest_closest(at, &m.field);
                let msg = match suggestion {
                    Some(s) => format!(
                        "unknown mutation field '{}' for {} — did you mean '{}'?",
                        head, at, s
                    ),
                    None => format!(
                        "unknown mutation field '{}' for {}",
                        head, at
                    ),
                };
                findings.push(Finding::error(
                    "E026",
                    msg,
                    format!("$.changes[{}].mutations[{}].field", c.index, m.index),
                ));
                continue;
            }
        }

        match m.op {
            MutationOp::Set | MutationOp::Append | MutationOp::Remove => {
                if m.value.is_none() {
                    findings.push(Finding::error(
                        "E006",
                        format!("mutation '{}' requires a value", m.op),
                        format!("$.changes[{}].mutations[{}].value", c.index, m.index),
                    ));
                }
            }
            MutationOp::Delete => {}
        }

        if let Some(Value::String(s)) = &m.value {
            if let Some(ref_name) = strip_ref_prefix(s) {
                if !refs.contains_key(ref_name) {
                    findings.push(Finding::error(
                        "E002",
                        format!("mutation value 'ref:{}' not defined in request", ref_name),
                        format!("$.changes[{}].mutations[{}].value", c.index, m.index),
                    ));
                }
            }
        }
        if let Some(Value::Sequence(seq)) = &m.value {
            for (i, item) in seq.iter().enumerate() {
                if let Value::String(s) = item {
                    if let Some(ref_name) = strip_ref_prefix(s) {
                        if !refs.contains_key(ref_name) {
                            findings.push(Finding::error(
                                "E002",
                                format!("mutation value 'ref:{}' not defined in request", ref_name),
                                format!(
                                    "$.changes[{}].mutations[{}].value[{}]",
                                    c.index, m.index, i
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Resolve a change target to its artifact type, looking first at refs in
/// the same request and then at the existing graph. Returns `None` if the
/// target is unknown — `E026` is then suppressed for that change so the
/// caller sees the higher-priority `E002` finding rather than an avalanche
/// of "unknown field for ?" errors.
fn resolve_target_artifact_type(
    c: &ChangeSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
) -> Option<ArtifactType> {
    if let Some(stripped) = strip_ref_prefix(&c.target) {
        return refs.get(stripped).map(|(t, _)| *t);
    }
    if ctx.graph.features.contains_key(&c.target) {
        Some(ArtifactType::Feature)
    } else if ctx.graph.adrs.contains_key(&c.target) {
        Some(ArtifactType::Adr)
    } else if ctx.graph.tests.contains_key(&c.target) {
        Some(ArtifactType::Tc)
    } else if ctx.graph.dependencies.contains_key(&c.target) {
        Some(ArtifactType::Dep)
    } else {
        None
    }
}
