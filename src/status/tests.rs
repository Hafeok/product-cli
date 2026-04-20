//! Unit tests for pure status builders.

use super::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Feature, FeatureFrontMatter, FeatureStatus, TestCriterion, TestFrontMatter, TestStatus,
    TestType, ValidatesBlock,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn empty_config() -> ProductConfig {
    toml::from_str("name = \"test\"\n").expect("parse")
}

fn feature(id: &str, title: &str, phase: u32, status: FeatureStatus, tests: Vec<String>) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: title.to_string(),
            phase,
            status,
            depends_on: vec![],
            adrs: vec![],
            tests,
            domains: vec![],
            domains_acknowledged: HashMap::new(),
            bundle: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn test(id: &str, status: TestStatus, features: Vec<String>) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: format!("test {}", id),
            test_type: TestType::Scenario,
            status,
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

// ---- build_project_summary -----------------------------------------------

#[test]
fn build_project_summary_empty_graph_has_no_phases() {
    let config = empty_config();
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let summary = summary::build_project_summary(&config, &graph, None);
    assert_eq!(summary.project, "test");
    assert!(summary.phases.is_empty());
}

#[test]
fn build_project_summary_groups_features_by_phase() {
    let config = empty_config();
    let f1 = feature("FT-001", "a", 1, FeatureStatus::Complete, vec![]);
    let f2 = feature("FT-002", "b", 2, FeatureStatus::Planned, vec![]);
    let f3 = feature("FT-003", "c", 1, FeatureStatus::InProgress, vec![]);
    let graph = KnowledgeGraph::build(vec![f1, f2, f3], vec![], vec![]);

    let summary = summary::build_project_summary(&config, &graph, None);
    assert_eq!(summary.phases.len(), 2);
    let p1 = &summary.phases[0];
    assert_eq!(p1.phase, 1);
    assert_eq!(p1.total, 2);
    assert_eq!(p1.complete, 1);
    let p2 = &summary.phases[1];
    assert_eq!(p2.phase, 2);
    assert_eq!(p2.total, 1);
    assert_eq!(p2.complete, 0);
}

#[test]
fn build_project_summary_filters_to_single_phase() {
    let config = empty_config();
    let f1 = feature("FT-001", "a", 1, FeatureStatus::Complete, vec![]);
    let f2 = feature("FT-002", "b", 2, FeatureStatus::Planned, vec![]);
    let graph = KnowledgeGraph::build(vec![f1, f2], vec![], vec![]);

    let summary = summary::build_project_summary(&config, &graph, Some(2));
    assert_eq!(summary.phases.len(), 1);
    assert_eq!(summary.phases[0].phase, 2);
}

#[test]
fn build_project_summary_counts_passing_tests_per_feature() {
    let config = empty_config();
    let f = feature(
        "FT-001",
        "a",
        1,
        FeatureStatus::InProgress,
        vec!["TC-001".into(), "TC-002".into(), "TC-003".into()],
    );
    let t1 = test("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let t2 = test("TC-002", TestStatus::Passing, vec!["FT-001".into()]);
    let t3 = test("TC-003", TestStatus::Failing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t1, t2, t3]);

    let summary = summary::build_project_summary(&config, &graph, None);
    let row = &summary.phases[0].features[0];
    assert_eq!(row.tests_passing, 2);
    assert_eq!(row.tests_total, 3);
}

// ---- build_untested_list -------------------------------------------------

#[test]
fn build_untested_list_excludes_abandoned() {
    let f1 = feature("FT-001", "a", 1, FeatureStatus::Planned, vec![]);
    let f2 = feature("FT-002", "b", 1, FeatureStatus::Abandoned, vec![]);
    let f3 = feature(
        "FT-003",
        "c",
        1,
        FeatureStatus::Planned,
        vec!["TC-001".into()],
    );
    let graph = KnowledgeGraph::build(vec![f1, f2, f3], vec![], vec![]);

    let list = summary::build_untested_list(&graph);
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].id, "FT-001");
}

// ---- build_failing_list --------------------------------------------------

#[test]
fn build_failing_list_picks_features_with_any_failing_test() {
    let f1 = feature(
        "FT-001",
        "a",
        1,
        FeatureStatus::InProgress,
        vec!["TC-001".into(), "TC-002".into()],
    );
    let f2 = feature(
        "FT-002",
        "b",
        1,
        FeatureStatus::InProgress,
        vec!["TC-003".into()],
    );
    let t1 = test("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let t2 = test("TC-002", TestStatus::Failing, vec!["FT-001".into()]);
    let t3 = test("TC-003", TestStatus::Passing, vec!["FT-002".into()]);
    let graph = KnowledgeGraph::build(vec![f1, f2], vec![], vec![t1, t2, t3]);

    let list = summary::build_failing_list(&graph);
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].id, "FT-001");
}

// ---- render ---------------------------------------------------------------

#[test]
fn render_project_summary_includes_project_name_and_phases() {
    let config = empty_config();
    let f = feature("FT-001", "hello", 1, FeatureStatus::Complete, vec![]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let summary = summary::build_project_summary(&config, &graph, None);
    let text = render::render_project_summary_text(&summary, false);
    assert!(text.contains("Project Status: test"));
    assert!(text.contains("Phase 1"));
    assert!(text.contains("FT-001"));
    assert!(text.contains("hello"));
}

#[test]
fn render_feature_list_includes_heading_and_rows() {
    let f = feature("FT-007", "untested", 3, FeatureStatus::Planned, vec![]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let list = summary::build_untested_list(&graph);
    let text = render::render_feature_list_text("Features with no linked tests:", &list);
    assert!(text.contains("Features with no linked tests:"));
    assert!(text.contains("FT-007"));
    assert!(text.contains("phase 3"));
}
