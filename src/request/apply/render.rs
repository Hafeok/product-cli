//! Rendering new/mutated artifacts to their final file content.

use super::super::types::*;
use crate::parser;
use crate::types::{
    AdrFrontMatter, DependencyFrontMatter, FeatureFrontMatter, TestFrontMatter,
};
use serde_yaml::{Mapping, Value};

/// Render a newly-created artifact. Extracts the optional `body` key and
/// converts the remaining mapping into the typed front-matter struct.
pub fn render_new_artifact(
    artifact_type: ArtifactType,
    mut map: Mapping,
    type_label: &str,
) -> Result<String, Vec<Finding>> {
    let title = map
        .get(Value::String("title".into()))
        .and_then(|v| v.as_str())
        .unwrap_or(type_label)
        .to_string();
    let body = map
        .remove(Value::String("body".into()))
        .and_then(|v| match v {
            Value::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| default_body(artifact_type, &title));

    render_for_type(artifact_type, &map, &body)
}

pub fn default_body(t: ArtifactType, title: &str) -> String {
    match t {
        ArtifactType::Feature => format!("## Description\n\n[Describe {} here.]\n", title),
        ArtifactType::Adr => "## Context\n\n\n## Decision\n\n\n## Rationale\n\n\n## Rejected alternatives\n\n\n## Test coverage\n\n".to_string(),
        ArtifactType::Tc => format!("## Description\n\n[Describe {} here.]\n", title),
        ArtifactType::Dep => format!("# {}\n\n[Describe this dependency.]\n", title),
    }
}

pub fn render_for_type(
    t: ArtifactType,
    map: &Mapping,
    body: &str,
) -> Result<String, Vec<Finding>> {
    match t {
        ArtifactType::Feature => {
            let value = Value::Mapping(map.clone());
            let mut front: FeatureFrontMatter = serde_yaml::from_value(value).map_err(|e| {
                vec![Finding::error("E001", format!("failed to build feature front-matter: {}", e), "$")]
            })?;
            normalize_feature(&mut front);
            Ok(parser::render_feature(&front, body))
        }
        ArtifactType::Adr => {
            let value = Value::Mapping(map.clone());
            let mut front: AdrFrontMatter = serde_yaml::from_value(value).map_err(|e| {
                vec![Finding::error("E001", format!("failed to build ADR front-matter: {}", e), "$")]
            })?;
            normalize_adr(&mut front);
            Ok(parser::render_adr(&front, body))
        }
        ArtifactType::Tc => {
            let mut m2 = map.clone();
            // Accept `tc-type` as alias for `type` during parsing
            let tc_type_key = Value::String("tc-type".into());
            if let Some(v) = m2.remove(&tc_type_key) {
                m2.insert(Value::String("type".into()), v);
            }
            let value = Value::Mapping(m2);
            let mut front: TestFrontMatter = serde_yaml::from_value(value).map_err(|e| {
                vec![Finding::error("E001", format!("failed to build TC front-matter: {}", e), "$")]
            })?;
            normalize_tc(&mut front);
            Ok(parser::render_test(&front, body))
        }
        ArtifactType::Dep => {
            let mut m2 = map.clone();
            let dep_type_key = Value::String("dep-type".into());
            if let Some(v) = m2.remove(&dep_type_key) {
                m2.insert(Value::String("type".into()), v);
            }
            let value = Value::Mapping(m2);
            let mut front: DependencyFrontMatter = serde_yaml::from_value(value).map_err(|e| {
                vec![Finding::error("E001", format!("failed to build dep front-matter: {}", e), "$")]
            })?;
            normalize_dep(&mut front);
            Ok(parser::render_dependency(&front, body))
        }
    }
}

/// Detect artifact type from `id` prefix, then render.
pub fn render_from_mapping(map: &Mapping, body: &str) -> Result<String, Vec<Finding>> {
    let id = map.get(Value::String("id".into())).and_then(|v| v.as_str()).unwrap_or("");
    let t = if id.starts_with("FT-") {
        ArtifactType::Feature
    } else if id.starts_with("ADR-") {
        ArtifactType::Adr
    } else if id.starts_with("TC-") {
        ArtifactType::Tc
    } else if id.starts_with("DEP-") {
        ArtifactType::Dep
    } else {
        ArtifactType::Feature
    };
    render_for_type(t, map, body)
}

fn normalize_feature(f: &mut FeatureFrontMatter) {
    f.depends_on.sort();
    f.depends_on.dedup();
    f.adrs.sort();
    f.adrs.dedup();
    f.tests.sort();
    f.tests.dedup();
    f.domains.sort();
    f.domains.dedup();
}

fn normalize_adr(a: &mut AdrFrontMatter) {
    a.features.sort();
    a.features.dedup();
    a.supersedes.sort();
    a.supersedes.dedup();
    a.superseded_by.sort();
    a.superseded_by.dedup();
    a.domains.sort();
    a.domains.dedup();
    a.source_files.sort();
    a.source_files.dedup();
}

fn normalize_tc(t: &mut TestFrontMatter) {
    t.validates.features.sort();
    t.validates.features.dedup();
    t.validates.adrs.sort();
    t.validates.adrs.dedup();
    t.requires.sort();
    t.requires.dedup();
}

fn normalize_dep(d: &mut DependencyFrontMatter) {
    d.features.sort();
    d.features.dedup();
    d.adrs.sort();
    d.adrs.dedup();
    d.supersedes.sort();
    d.supersedes.dedup();
}
