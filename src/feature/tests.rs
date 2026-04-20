//! Unit tests for the pure feature planning functions.
//!
//! These verify domain behaviour (ID generation, cascade computation,
//! validation) without any filesystem or external dependencies.

use super::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Feature, FeatureFrontMatter, FeatureStatus, TestCriterion, TestFrontMatter, TestStatus,
    TestType, ValidatesBlock,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn empty_front(id: &str, title: &str) -> FeatureFrontMatter {
    FeatureFrontMatter {
        id: id.to_string(),
        title: title.to_string(),
        phase: 1,
        status: FeatureStatus::Planned,
        depends_on: vec![],
        adrs: vec![],
        tests: vec![],
        domains: vec![],
        domains_acknowledged: HashMap::new(),
        bundle: None,
    }
}

fn feature_at(id: &str, title: &str, tests: Vec<String>) -> Feature {
    let mut front = empty_front(id, title);
    front.tests = tests;
    Feature {
        front,
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn test_criterion(id: &str, features: Vec<String>) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: format!("test {}", id),
            test_type: TestType::Scenario,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock {
                features,
                adrs: vec![],
            },
            phase: 1,
            content_hash: None,
            runner: None,
            runner_args: None,
            runner_timeout: None,
            requires: vec![],
            last_run: None,
            failure_message: None,
            last_run_duration: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
        formal_blocks: vec![],
    }
}

/// Build a minimal in-memory ProductConfig from TOML, giving each domain a
/// dummy description. Avoids the filesystem so unit tests stay hermetic.
fn config_with_domains(names: &[&str]) -> ProductConfig {
    let domain_entries: String = names
        .iter()
        .map(|d| format!("{} = \"test domain\"", d))
        .collect::<Vec<_>>()
        .join("\n");
    let toml_str = format!(
        r#"name = "test"
[domains]
{}
"#,
        domain_entries
    );
    toml::from_str(&toml_str).expect("parse test config")
}

// ---- plan_create ---------------------------------------------------------

#[test]
fn plan_create_rejects_empty_title() {
    let err = create::plan_create("", 1, &[], "FT").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_create_rejects_whitespace_title() {
    let err = create::plan_create("   ", 1, &[], "FT").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_create_trims_title() {
    let plan = create::plan_create("  my feature  ", 1, &[], "FT").expect("plan");
    assert_eq!(plan.front.title, "my feature");
}

#[test]
fn plan_create_assigns_next_id_starting_from_001_when_empty() {
    let plan = create::plan_create("first", 1, &[], "FT").expect("plan");
    assert_eq!(plan.id, "FT-001");
    assert_eq!(plan.filename, "FT-001-first.md");
}

#[test]
fn plan_create_assigns_next_id_after_existing() {
    let existing = vec!["FT-001".to_string(), "FT-002".to_string()];
    let plan = create::plan_create("third", 2, &existing, "FT").expect("plan");
    assert_eq!(plan.id, "FT-003");
    assert_eq!(plan.front.phase, 2);
}

#[test]
fn plan_create_default_status_is_planned() {
    let plan = create::plan_create("x", 1, &[], "FT").expect("plan");
    assert_eq!(plan.front.status, FeatureStatus::Planned);
    assert!(plan.front.adrs.is_empty());
    assert!(plan.front.tests.is_empty());
}

#[test]
fn plan_create_rendered_contains_title_and_id() {
    let plan = create::plan_create("hello world", 1, &[], "FT").expect("plan");
    let rendered = plan.rendered();
    assert!(rendered.contains("FT-001"));
    assert!(rendered.contains("hello world"));
}

// ---- plan_status_change + cascade ---------------------------------------

#[test]
fn plan_status_change_not_found_when_missing() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let err = status_change::plan_status_change(
        &graph,
        "FT-999",
        FeatureStatus::InProgress,
    )
    .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}

#[test]
fn plan_status_change_to_in_progress_has_empty_cascade() {
    let feature = feature_at("FT-001", "x", vec![]);
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
    let plan = status_change::plan_status_change(
        &graph,
        "FT-001",
        FeatureStatus::InProgress,
    )
    .expect("plan");
    assert_eq!(plan.new_status, FeatureStatus::InProgress);
    assert!(plan.orphaned_tests.is_empty());
}

#[test]
fn plan_status_change_to_abandoned_orphans_linked_tests() {
    let feature = feature_at(
        "FT-001",
        "x",
        vec!["TC-001".to_string(), "TC-002".to_string()],
    );
    let tc1 = test_criterion("TC-001", vec!["FT-001".to_string()]);
    let tc2 = test_criterion("TC-002", vec!["FT-001".to_string(), "FT-002".to_string()]);
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![tc1, tc2]);

    let plan = status_change::plan_status_change(
        &graph,
        "FT-001",
        FeatureStatus::Abandoned,
    )
    .expect("plan");

    assert_eq!(plan.orphaned_tests.len(), 2);
    let tc1_update = plan
        .orphaned_tests
        .iter()
        .find(|u| u.test_id == "TC-001")
        .expect("tc1");
    assert!(!tc1_update.content.contains("FT-001"));
    let tc2_update = plan
        .orphaned_tests
        .iter()
        .find(|u| u.test_id == "TC-002")
        .expect("tc2");
    assert!(!tc2_update.content.contains("FT-001"));
    assert!(tc2_update.content.contains("FT-002"));
}

#[test]
fn plan_status_change_to_abandoned_no_tests_has_empty_cascade() {
    let feature = feature_at("FT-001", "x", vec![]);
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
    let plan = status_change::plan_status_change(
        &graph,
        "FT-001",
        FeatureStatus::Abandoned,
    )
    .expect("plan");
    assert!(plan.orphaned_tests.is_empty());
}

// ---- plan_domain_edit ---------------------------------------------------

#[test]
fn plan_domain_edit_rejects_unknown_domain_in_add() {
    let feature = feature_at("FT-001", "x", vec![]);
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
    let config = config_with_domains(&["security", "performance"]);
    let err = domain_edit::plan_domain_edit(
        &config,
        &graph,
        "FT-001",
        &["unknown".to_string()],
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_domain_edit_adds_sorted_and_deduplicates() {
    let feature = feature_at("FT-001", "x", vec![]);
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
    let config = config_with_domains(&["security", "performance", "ux"]);
    let plan = domain_edit::plan_domain_edit(
        &config,
        &graph,
        "FT-001",
        &["ux".to_string(), "security".to_string(), "ux".to_string()],
        &[],
    )
    .expect("plan");
    assert_eq!(plan.final_domains, vec!["security", "ux"]);
}

#[test]
fn plan_domain_edit_removes_idempotently() {
    let mut feature = feature_at("FT-001", "x", vec![]);
    feature.front.domains = vec!["security".to_string(), "ux".to_string()];
    let graph = KnowledgeGraph::build(vec![feature], vec![], vec![]);
    let config = config_with_domains(&["security", "ux"]);
    let plan = domain_edit::plan_domain_edit(
        &config,
        &graph,
        "FT-001",
        &[],
        &["ux".to_string(), "missing".to_string()],
    )
    .expect("plan");
    assert_eq!(plan.final_domains, vec!["security"]);
}

#[test]
fn plan_domain_edit_not_found_when_feature_missing() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let config = config_with_domains(&["security"]);
    let err = domain_edit::plan_domain_edit(&config, &graph, "FT-999", &[], &[]).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}
