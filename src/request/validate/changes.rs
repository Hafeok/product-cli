//! Change-spec validation.

use super::super::types::*;
use super::helpers::strip_ref_prefix;
use super::ValidationContext;
use serde_yaml::Value;
use std::collections::HashMap;

pub fn validate_change(
    c: &ChangeSpec,
    refs: &HashMap<String, (ArtifactType, usize)>,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
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
