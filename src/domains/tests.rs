//! Unit tests for concern domain classification (ADR-025, ADR-026)

use super::*;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn make_feature(id: &str, adrs: Vec<&str>, domains: Vec<&str>) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: format!("Feature {}", id),
            phase: 1,
            status: FeatureStatus::Planned,
            depends_on: vec![],
            adrs: adrs.into_iter().map(String::from).collect(),
            tests: vec![],
            domains: domains.into_iter().map(String::from).collect(),
            domains_acknowledged: HashMap::new(),
            bundle: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn make_adr(id: &str, scope: AdrScope, domains: Vec<&str>) -> Adr {
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title: format!("ADR {}", id),
            status: AdrStatus::Accepted,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: domains.into_iter().map(String::from).collect(),
            scope,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

#[test]
fn preflight_clean_when_all_covered() {
    let f = make_feature("FT-001", vec!["ADR-001"], vec![]);
    let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let vocab = HashMap::new();
    let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
    assert!(result.is_clean);
}

#[test]
fn preflight_detects_cross_cutting_gap() {
    let f = make_feature("FT-001", vec![], vec![]); // no ADRs linked
    let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let vocab = HashMap::new();
    let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
    assert!(!result.is_clean);
    assert!(result.cross_cutting_gaps.iter().any(|g| g.status == CoverageStatus::Gap));
}

#[test]
fn preflight_detects_domain_gap() {
    let f = make_feature("FT-001", vec![], vec!["security"]); // declares security domain
    let a = make_adr("ADR-010", AdrScope::Domain, vec!["security"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let mut vocab = HashMap::new();
    vocab.insert("security".to_string(), "Security concerns".to_string());
    let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
    assert!(!result.is_clean);
    assert!(result.domain_gaps.iter().any(|g| g.domain == "security" && g.status == CoverageStatus::Gap));
}

#[test]
fn acknowledgement_closes_gap() {
    let mut f = make_feature("FT-001", vec![], vec!["security"]);
    f.front.domains_acknowledged.insert("security".to_string(), "no trust boundaries".to_string());
    let a = make_adr("ADR-010", AdrScope::Domain, vec!["security"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let mut vocab = HashMap::new();
    vocab.insert("security".to_string(), "Security concerns".to_string());
    let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
    assert!(result.is_clean);
}

#[test]
fn empty_acknowledgement_rejected() {
    let f = make_feature("FT-001", vec![], vec![]);
    let result = acknowledge_domain(&f, "security", "");
    assert!(result.is_err());
}

#[test]
fn coverage_matrix_builds() {
    let f = make_feature("FT-001", vec!["ADR-001"], vec!["security"]);
    let a = make_adr("ADR-001", AdrScope::Domain, vec!["security"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let mut vocab = HashMap::new();
    vocab.insert("security".to_string(), "Security".to_string());
    let matrix = build_coverage_matrix(&graph, &vocab);
    assert_eq!(matrix.features.len(), 1);
    let cell = matrix.cells.get(&("FT-001".to_string(), "security".to_string()));
    assert!(matches!(cell, Some(CoverageCell::Covered)));
}

#[test]
fn e011_empty_reason_detected() {
    let mut f = make_feature("FT-001", vec![], vec![]);
    f.front.domains_acknowledged.insert("security".to_string(), "".to_string());
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let vocab = HashMap::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validate_domains(&graph, &vocab, &mut errors, &mut warnings);
    assert!(errors.iter().any(|e| e.code == "E011"));
}

#[test]
fn w010_cross_cutting_unacknowledged() {
    let f = make_feature("FT-001", vec![], vec![]);
    let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
    let vocab = HashMap::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validate_domains(&graph, &vocab, &mut errors, &mut warnings);
    assert!(warnings.iter().any(|w| w.code == "W010"));
}
