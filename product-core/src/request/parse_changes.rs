//! Parse the `changes:` section of a request YAML (FT-041, FT-064).
//!
//! Enforces the strict closed-set key validation on `change` and `mutation`
//! blocks that FT-064 introduced — any key outside the documented set
//! surfaces as `E025` rather than being silently dropped. Empty
//! `mutations: []` is rejected with `E006` because a change that performs
//! zero mutations is undecidable intent.

use super::types::*;
use serde_yaml::{Mapping, Value};

/// Closed set of recognised keys inside a `change` block (FT-064).
const KNOWN_CHANGE_KEYS: &[&str] = &["target", "mutations"];

/// Closed set of recognised keys inside a `mutation` block (FT-064).
pub(super) const KNOWN_MUTATION_KEYS: &[&str] = &["op", "field", "value"];

pub fn parse_changes_array(map: &Mapping) -> Result<Vec<ChangeSpec>, Vec<Finding>> {
    let mut changes = Vec::new();
    let mut errors = Vec::new();
    if let Some(Value::Sequence(seq)) = map.get(Value::String("changes".into())) {
        for (i, item) in seq.iter().enumerate() {
            match parse_change(item, i) {
                Ok(c) => changes.push(c),
                Err(mut e) => errors.append(&mut e),
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(changes)
}

fn parse_change(item: &Value, index: usize) -> Result<ChangeSpec, Vec<Finding>> {
    let map = item.as_mapping().ok_or_else(|| {
        vec![Finding::error(
            "E001",
            "change must be a YAML mapping",
            format!("$.changes[{}]", index),
        )]
    })?;
    let mut findings = check_unknown_change_keys(map, index);
    let target = parse_change_target(map, index, &mut findings);
    let mutations = parse_change_mutations(map, index, &mut findings);
    enforce_non_empty_mutations(&mutations, index, &mut findings);
    if !findings.is_empty() {
        return Err(findings);
    }
    Ok(ChangeSpec {
        index,
        target: target.unwrap_or_default(),
        mutations,
    })
}

fn parse_change_target(
    map: &Mapping,
    index: usize,
    findings: &mut Vec<Finding>,
) -> Option<String> {
    match map.get(Value::String("target".into())) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => {
            findings.push(Finding::error(
                "E001",
                "change missing required field 'target'",
                format!("$.changes[{}].target", index),
            ));
            None
        }
    }
}

fn parse_change_mutations(
    map: &Mapping,
    index: usize,
    findings: &mut Vec<Finding>,
) -> Vec<Mutation> {
    match parse_mutations(map, index) {
        Ok(m) => m,
        Err(mut errs) => {
            findings.append(&mut errs);
            Vec::new()
        }
    }
}

/// FT-064 — reject `mutations: []` (or missing) on a `change` block. An
/// empty mutation list is undecidable: the apply would silently succeed
/// with 0 file changes, hiding the user's real intent.
fn enforce_non_empty_mutations(
    mutations: &[Mutation],
    index: usize,
    findings: &mut Vec<Finding>,
) {
    if !mutations.is_empty() {
        return;
    }
    let hint = "this change has no mutations — did you mean to nest `op:`/`field:`/`value:` inside a `mutations:` list?";
    findings.push(
        Finding::error(
            "E006",
            "change must contain at least one mutation",
            format!("$.changes[{}].mutations", index),
        )
        .with_hint(hint),
    );
}

fn check_unknown_change_keys(map: &Mapping, index: usize) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (k, _v) in map.iter() {
        if let Some(name) = k.as_str() {
            if !KNOWN_CHANGE_KEYS.contains(&name) {
                let hint = if KNOWN_MUTATION_KEYS.contains(&name) {
                    Some(format!(
                        "'{}' belongs inside a `mutations:` list entry, not on the change itself",
                        name
                    ))
                } else {
                    None
                };
                let mut f = Finding::error(
                    "E025",
                    format!(
                        "unknown key '{}' in change — expected one of: {}",
                        name,
                        KNOWN_CHANGE_KEYS.join(", ")
                    ),
                    format!("$.changes[{}].{}", index, name),
                );
                if let Some(h) = hint {
                    f = f.with_hint(h);
                }
                findings.push(f);
            }
        }
    }
    findings
}

fn parse_mutations(map: &Mapping, index: usize) -> Result<Vec<Mutation>, Vec<Finding>> {
    match map.get(Value::String("mutations".into())) {
        Some(Value::Sequence(seq)) => {
            let mut out = Vec::new();
            let mut errors = Vec::new();
            for (mi, m) in seq.iter().enumerate() {
                match parse_mutation(m, index, mi) {
                    Ok(mu) => out.push(mu),
                    Err(mut e) => errors.append(&mut e),
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
            Ok(out)
        }
        Some(_) => Err(vec![Finding::error(
            "E001",
            "mutations must be a sequence",
            format!("$.changes[{}].mutations", index),
        )]),
        None => Ok(Vec::new()),
    }
}

fn parse_mutation(m: &Value, change_idx: usize, idx: usize) -> Result<Mutation, Vec<Finding>> {
    let map = m.as_mapping().ok_or_else(|| {
        vec![Finding::error(
            "E001",
            "mutation must be a YAML mapping",
            format!("$.changes[{}].mutations[{}]", change_idx, idx),
        )]
    })?;

    // FT-064 — strict closed-set validation on mutation keys.
    let mut findings = check_unknown_mutation_keys(map, change_idx, idx);

    let op = match parse_mutation_op(map, change_idx, idx) {
        Ok(o) => Some(o),
        Err(mut errs) => {
            findings.append(&mut errs);
            None
        }
    };
    let field = match parse_mutation_field(map, change_idx, idx) {
        Ok(f) => Some(f),
        Err(mut errs) => {
            findings.append(&mut errs);
            None
        }
    };
    let value = map.get(Value::String("value".into())).cloned();

    if !findings.is_empty() {
        return Err(findings);
    }
    Ok(Mutation {
        index: idx,
        op: op.expect("op parsed without findings"),
        field: field.expect("field parsed without findings"),
        value,
    })
}

fn check_unknown_mutation_keys(
    map: &Mapping,
    change_idx: usize,
    idx: usize,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (k, _v) in map.iter() {
        if let Some(name) = k.as_str() {
            if !KNOWN_MUTATION_KEYS.contains(&name) {
                findings.push(Finding::error(
                    "E025",
                    format!(
                        "unknown key '{}' in mutation — expected one of: {}",
                        name,
                        KNOWN_MUTATION_KEYS.join(", ")
                    ),
                    format!("$.changes[{}].mutations[{}].{}", change_idx, idx, name),
                ));
            }
        }
    }
    findings
}

fn parse_mutation_op(
    map: &Mapping,
    change_idx: usize,
    idx: usize,
) -> Result<MutationOp, Vec<Finding>> {
    let op_str = match map.get(Value::String("op".into())) {
        Some(Value::String(s)) => s.clone(),
        _ => {
            return Err(vec![Finding::error(
                "E001",
                "mutation missing required field 'op'",
                format!("$.changes[{}].mutations[{}].op", change_idx, idx),
            )])
        }
    };
    MutationOp::parse(&op_str).ok_or_else(|| {
        vec![Finding::error(
            "E001",
            format!(
                "unknown mutation op '{}' — expected one of: set, append, remove, delete",
                op_str
            ),
            format!("$.changes[{}].mutations[{}].op", change_idx, idx),
        )]
    })
}

fn parse_mutation_field(
    map: &Mapping,
    change_idx: usize,
    idx: usize,
) -> Result<String, Vec<Finding>> {
    match map.get(Value::String("field".into())) {
        Some(Value::String(s)) => Ok(s.clone()),
        _ => Err(vec![Finding::error(
            "E001",
            "mutation missing required field 'field'",
            format!("$.changes[{}].mutations[{}].field", change_idx, idx),
        )]),
    }
}
