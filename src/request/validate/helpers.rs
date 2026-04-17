//! Shared validation helpers.

use super::super::types::*;
use crate::graph::KnowledgeGraph;
use serde_yaml::Value;
use std::collections::HashMap;

pub fn strip_ref_prefix(s: &str) -> Option<&str> {
    s.strip_prefix("ref:")
}

pub fn check_domains_vocab(
    field: Option<&Value>,
    vocab: &HashMap<String, String>,
    location: &str,
    findings: &mut Vec<Finding>,
) {
    let Some(Value::Sequence(seq)) = field else { return; };
    for (i, item) in seq.iter().enumerate() {
        if let Value::String(d) = item {
            if !vocab.contains_key(d) {
                findings.push(Finding::error(
                    "E012",
                    format!("unknown domain '{}' — not in [domains] vocabulary in product.toml", d),
                    format!("{}[{}]", location, i),
                ));
            }
        }
    }
}

pub fn check_id_list(
    field: Option<&Value>,
    expected: ArtifactType,
    refs: &HashMap<String, (ArtifactType, usize)>,
    graph: &KnowledgeGraph,
    location: &str,
    findings: &mut Vec<Finding>,
) {
    let Some(v) = field else { return; };
    check_id_list_value(v, expected, refs, graph, location, findings);
}

pub fn check_id_list_value(
    v: &Value,
    expected: ArtifactType,
    refs: &HashMap<String, (ArtifactType, usize)>,
    graph: &KnowledgeGraph,
    location: &str,
    findings: &mut Vec<Finding>,
) {
    let Value::Sequence(seq) = v else { return; };
    for (i, item) in seq.iter().enumerate() {
        if let Value::String(s) = item {
            check_single_id(s, expected, refs, graph, &format!("{}[{}]", location, i), findings);
        }
    }
}

pub fn check_single_id(
    s: &str,
    expected: ArtifactType,
    refs: &HashMap<String, (ArtifactType, usize)>,
    graph: &KnowledgeGraph,
    location: &str,
    findings: &mut Vec<Finding>,
) {
    if let Some(ref_name) = strip_ref_prefix(s) {
        match refs.get(ref_name) {
            Some((t, _)) if *t == expected => {}
            Some((t, _)) => {
                findings.push(Finding::error(
                    "E001",
                    format!("ref:{} resolves to a {} but {} was expected", ref_name, t, expected),
                    location.to_string(),
                ));
            }
            None => {
                findings.push(Finding::error(
                    "E002",
                    format!("ref:{} is not defined in request", ref_name),
                    location.to_string(),
                ));
            }
        }
    } else {
        let exists = match expected {
            ArtifactType::Feature => graph.features.contains_key(s),
            ArtifactType::Adr => graph.adrs.contains_key(s),
            ArtifactType::Tc => graph.tests.contains_key(s),
            ArtifactType::Dep => graph.dependencies.contains_key(s),
        };
        if !exists {
            findings.push(Finding::error(
                "E002",
                format!("{} '{}' does not exist in the graph", expected, s),
                location.to_string(),
            ));
        }
    }
}
