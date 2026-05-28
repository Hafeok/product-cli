//! Pattern slice unit tests — pure functions only, no I/O.

use super::*;
use crate::config::PatternsConfig;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Feature, FeatureFrontMatter, FeatureStatus, Pattern, PatternFrontMatter, PatternStatus,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn pat(id: &str) -> Pattern {
    Pattern {
        front: PatternFrontMatter {
            id: id.to_string(),
            title: format!("pattern {}", id),
            status: PatternStatus::Live,
            domains: vec![],
            adrs: vec![],
            requires: vec![],
            examples: vec![],
            deprecated_by: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("docs/patterns/{}.md", id)),
    }
}

fn feat(id: &str) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: format!("feature {}", id),
            phase: 1,
            status: FeatureStatus::Planned,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: HashMap::new(),
            patterns: vec![],
            due_date: None,
            bundle: None,
            adrs_rejected: vec![],
        },
        body: String::new(),
        path: PathBuf::from(format!("docs/features/{}.md", id)),
    }
}

#[test]
fn create_plan_uses_pat_prefix_and_default_sections() {
    let cfg = PatternsConfig::default();
    let plan = plan_create("Slice + Adapter module structure", &[], "PAT", &cfg).unwrap();
    assert_eq!(plan.id, "PAT-001");
    assert!(plan.front.adrs.is_empty());
    assert!(plan.front.requires.is_empty());
    assert!(plan.front.examples.is_empty());
    assert_eq!(plan.front.status, PatternStatus::Live);
    for heading in &cfg.body_sections {
        let expected = format!("## {}", heading);
        assert!(plan.body.contains(&expected), "missing heading {}", heading);
    }
}

#[test]
fn create_plan_rejects_empty_title() {
    let cfg = PatternsConfig::default();
    let err = plan_create("   ", &[], "PAT", &cfg).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn status_change_to_deprecated_sets_deprecated_by() {
    let g = KnowledgeGraph::build(vec![], vec![], vec![]);
    let mut patterns = HashMap::new();
    patterns.insert("PAT-001".to_string(), pat("PAT-001"));
    patterns.insert("PAT-042".to_string(), pat("PAT-042"));
    let plan = plan_status_change(
        &g,
        &patterns,
        "PAT-001",
        PatternStatus::Deprecated,
        Some("PAT-042"),
    )
    .unwrap();
    assert_eq!(plan.new_status, PatternStatus::Deprecated);
    assert_eq!(plan.deprecated_by.as_deref(), Some("PAT-042"));
}

#[test]
fn status_change_to_live_clears_deprecated_by() {
    let g = KnowledgeGraph::build(vec![], vec![], vec![]);
    let mut patterns = HashMap::new();
    let mut p = pat("PAT-001");
    p.front.status = PatternStatus::Deprecated;
    p.front.deprecated_by = Some("PAT-042".to_string());
    patterns.insert("PAT-001".to_string(), p);
    patterns.insert("PAT-042".to_string(), pat("PAT-042"));
    let plan = plan_status_change(&g, &patterns, "PAT-001", PatternStatus::Live, None).unwrap();
    assert_eq!(plan.new_status, PatternStatus::Live);
    assert!(plan.deprecated_by.is_none());
    // The serialised content must not include a `deprecated-by:` line.
    assert!(
        !plan.content.contains("deprecated-by:"),
        "expected deprecated-by removed:\n{}",
        plan.content,
    );
}

#[test]
fn link_requires_cycle_returns_e003() {
    let g = KnowledgeGraph::build(vec![], vec![], vec![]);
    let mut patterns = HashMap::new();
    let mut a = pat("PAT-001");
    a.front.requires.push("PAT-002".to_string());
    let b = pat("PAT-002");
    patterns.insert("PAT-001".to_string(), a);
    patterns.insert("PAT-002".to_string(), b);
    let err = plan_link(&g, &patterns, "PAT-002", None, Some("PAT-001"), None).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::DependencyCycle { .. }));
}

#[test]
fn link_example_emits_feature_reciprocation() {
    let g = KnowledgeGraph::build(vec![feat("FT-100")], vec![], vec![]);
    let mut patterns = HashMap::new();
    patterns.insert("PAT-001".to_string(), pat("PAT-001"));
    let plan = plan_link(&g, &patterns, "PAT-001", None, None, Some("FT-100")).unwrap();
    assert_eq!(plan.writes.len(), 2, "pattern + feature");
    assert_eq!(plan.reciprocated.len(), 1);
    assert_eq!(plan.reciprocated[0].id, "FT-100");
    assert_eq!(plan.reciprocated[0].field, "patterns");
}

#[test]
fn link_unknown_adr_returns_not_found_before_write() {
    let g = KnowledgeGraph::build(vec![], vec![], vec![]);
    let mut patterns = HashMap::new();
    patterns.insert("PAT-001".to_string(), pat("PAT-001"));
    let err = plan_link(&g, &patterns, "PAT-001", Some("ADR-999"), None, None).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}
