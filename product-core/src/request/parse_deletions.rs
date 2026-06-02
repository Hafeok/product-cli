//! Parse the `deletions:` section of a request YAML (FT-064).
//!
//! Two shapes are accepted: a bare string (the artifact ID), or a mapping
//! with a `target:` key. The mapping form keeps the schema extensible — a
//! future `cascade: true` toggle can be added without a v2 schema bump.

use super::types::*;
use serde_yaml::{Mapping, Value};

const KNOWN_DELETION_KEYS: &[&str] = &["target"];

pub fn parse_deletions_array(map: &Mapping) -> Result<Vec<DeletionSpec>, Vec<Finding>> {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    if let Some(Value::Sequence(seq)) = map.get(Value::String("deletions".into())) {
        for (i, item) in seq.iter().enumerate() {
            match parse_deletion(item, i) {
                Ok(d) => out.push(d),
                Err(mut e) => errors.append(&mut e),
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(out)
}

fn parse_deletion(item: &Value, index: usize) -> Result<DeletionSpec, Vec<Finding>> {
    if let Some(s) = item.as_str() {
        return Ok(DeletionSpec { index, target: s.to_string() });
    }
    let map = item.as_mapping().ok_or_else(|| {
        vec![Finding::error(
            "E001",
            "deletion must be a YAML mapping or a bare artifact ID string",
            format!("$.deletions[{}]", index),
        )]
    })?;
    let mut findings = check_unknown_deletion_keys(map, index);
    let target = match map.get(Value::String("target".into())) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => {
            findings.push(Finding::error(
                "E001",
                "deletion missing required field 'target'",
                format!("$.deletions[{}].target", index),
            ));
            None
        }
    };
    if !findings.is_empty() {
        return Err(findings);
    }
    Ok(DeletionSpec { index, target: target.unwrap_or_default() })
}

fn check_unknown_deletion_keys(map: &Mapping, index: usize) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (k, _v) in map.iter() {
        if let Some(name) = k.as_str() {
            if !KNOWN_DELETION_KEYS.contains(&name) {
                findings.push(Finding::error(
                    "E025",
                    format!(
                        "unknown key '{}' in deletion — expected one of: {}",
                        name,
                        KNOWN_DELETION_KEYS.join(", ")
                    ),
                    format!("$.deletions[{}].{}", index, name),
                ));
            }
        }
    }
    findings
}
