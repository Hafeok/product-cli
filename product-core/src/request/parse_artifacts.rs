//! Parse the `artifacts:` section of a request YAML (FT-041).

use super::types::*;
use serde_yaml::{Mapping, Value};

pub fn parse_artifacts_array(map: &Mapping) -> Result<Vec<ArtifactSpec>, Vec<Finding>> {
    let mut artifacts = Vec::new();
    if let Some(Value::Sequence(seq)) = map.get(Value::String("artifacts".into())) {
        for (i, item) in seq.iter().enumerate() {
            artifacts.push(parse_artifact(item, i)?);
        }
    }
    Ok(artifacts)
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
                "unknown artifact type '{}' — expected one of: feature, adr, tc, dep, pattern",
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
