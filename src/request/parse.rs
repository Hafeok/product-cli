//! Parse a request YAML document into `Request` (FT-041, ADR-038).

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

pub fn parse_request_str(yaml: &str) -> Result<Request, Vec<Finding>> {
    let doc: Value = serde_yaml::from_str(yaml)
        .map_err(|e| vec![Finding::error("E001", format!("malformed YAML: {}", e), "$")])?;
    let map = doc.as_mapping().ok_or_else(|| {
        vec![Finding::error("E001", "request document must be a YAML mapping", "$")]
    })?;

    let request_type = parse_type(map)?;
    let schema_version = parse_schema_version(map)?;
    let reason = parse_reason(map);
    let artifacts = parse_artifacts_array(map)?;
    let changes = parse_changes_array(map)?;

    Ok(Request {
        request_type,
        schema_version,
        reason,
        artifacts,
        changes,
        source_yaml: yaml.to_string(),
    })
}

fn parse_type(map: &Mapping) -> Result<RequestType, Vec<Finding>> {
    let type_val = map.get(Value::String("type".into())).and_then(|v| v.as_str());
    match type_val {
        Some("create") => Ok(RequestType::Create),
        Some("change") => Ok(RequestType::Change),
        Some("create-and-change") => Ok(RequestType::CreateAndChange),
        Some(other) => Err(vec![Finding::error(
            "E001",
            format!(
                "unknown request type '{}' — expected one of: create, change, create-and-change",
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

fn parse_artifacts_array(map: &Mapping) -> Result<Vec<ArtifactSpec>, Vec<Finding>> {
    let mut artifacts = Vec::new();
    if let Some(Value::Sequence(seq)) = map.get(Value::String("artifacts".into())) {
        for (i, item) in seq.iter().enumerate() {
            artifacts.push(parse_artifact(item, i)?);
        }
    }
    Ok(artifacts)
}

fn parse_changes_array(map: &Mapping) -> Result<Vec<ChangeSpec>, Vec<Finding>> {
    let mut changes = Vec::new();
    if let Some(Value::Sequence(seq)) = map.get(Value::String("changes".into())) {
        for (i, item) in seq.iter().enumerate() {
            changes.push(parse_change(item, i)?);
        }
    }
    Ok(changes)
}

fn parse_artifact(item: &Value, index: usize) -> Result<ArtifactSpec, Vec<Finding>> {
    let map = item.as_mapping().cloned().ok_or_else(|| {
        vec![Finding::error(
            "E001",
            "artifact must be a YAML mapping",
            format!("$.artifacts[{}]", index),
        )]
    })?;

    let artifact_type = parse_artifact_type(&map, index)?;
    let ref_name = parse_ref_name(&map, index)?;

    let mut fields = Mapping::new();
    for (k, v) in map.iter() {
        if let Some(s) = k.as_str() {
            if s == "type" || s == "ref" {
                continue;
            }
        }
        fields.insert(k.clone(), v.clone());
    }

    Ok(ArtifactSpec { index, artifact_type, ref_name, fields })
}

fn parse_artifact_type(map: &Mapping, index: usize) -> Result<ArtifactType, Vec<Finding>> {
    let type_str = match map.get(Value::String("type".into())) {
        Some(Value::String(s)) => s.clone(),
        _ => {
            return Err(vec![Finding::error(
                "E001",
                "artifact missing required field 'type'",
                format!("$.artifacts[{}].type", index),
            )])
        }
    };
    ArtifactType::parse(&type_str).ok_or_else(|| {
        vec![Finding::error(
            "E001",
            format!(
                "unknown artifact type '{}' — expected one of: feature, adr, tc, dep",
                type_str
            ),
            format!("$.artifacts[{}].type", index),
        )]
    })
}

fn parse_ref_name(map: &Mapping, index: usize) -> Result<Option<String>, Vec<Finding>> {
    match map.get(Value::String("ref".into())) {
        Some(Value::String(s)) => Ok(Some(s.clone())),
        None => Ok(None),
        _ => Err(vec![Finding::error(
            "E001",
            "ref must be a string",
            format!("$.artifacts[{}].ref", index),
        )]),
    }
}

fn parse_change(item: &Value, index: usize) -> Result<ChangeSpec, Vec<Finding>> {
    let map = item.as_mapping().ok_or_else(|| {
        vec![Finding::error(
            "E001",
            "change must be a YAML mapping",
            format!("$.changes[{}]", index),
        )]
    })?;

    let target = match map.get(Value::String("target".into())) {
        Some(Value::String(s)) => s.clone(),
        _ => {
            return Err(vec![Finding::error(
                "E001",
                "change missing required field 'target'",
                format!("$.changes[{}].target", index),
            )])
        }
    };

    let mutations = parse_mutations(map, index)?;
    Ok(ChangeSpec { index, target, mutations })
}

fn parse_mutations(map: &Mapping, index: usize) -> Result<Vec<Mutation>, Vec<Finding>> {
    match map.get(Value::String("mutations".into())) {
        Some(Value::Sequence(seq)) => {
            let mut out = Vec::new();
            for (mi, m) in seq.iter().enumerate() {
                out.push(parse_mutation(m, index, mi)?);
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

    let op = parse_mutation_op(map, change_idx, idx)?;
    let field = parse_mutation_field(map, change_idx, idx)?;
    let value = map.get(Value::String("value".into())).cloned();

    Ok(Mutation { index: idx, op, field, value })
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
