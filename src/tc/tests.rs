//! Unit tests for pure TC planning functions.

use super::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::types::{TestCriterion, TestFrontMatter, TestStatus, TestType, ValidatesBlock};
use std::path::PathBuf;

fn tc(id: &str, title: &str, status: TestStatus) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: title.to_string(),
            test_type: TestType::Scenario,
            status,
            validates: ValidatesBlock { features: vec![], adrs: vec![] },
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

fn config_with_prereqs(names: &[&str]) -> ProductConfig {
    let entries: String = names
        .iter()
        .map(|n| format!("{} = \"true\"", n))
        .collect::<Vec<_>>()
        .join("\n");
    let toml_str = format!("name = \"t\"\n[verify.prerequisites]\n{}\n", entries);
    toml::from_str(&toml_str).expect("parse test config")
}

#[test]
fn plan_create_rejects_empty_title() {
    let err = create::plan_create("", TestType::Scenario, &[], "TC").unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_create_generates_next_id_and_scenario_default() {
    let plan = create::plan_create("new", TestType::Scenario, &[], "TC").expect("plan");
    assert_eq!(plan.id, "TC-001");
    assert_eq!(plan.front.test_type, TestType::Scenario);
    assert_eq!(plan.front.status, TestStatus::Unimplemented);
}

#[test]
fn plan_create_respects_test_type() {
    let plan = create::plan_create("inv", TestType::Invariant, &[], "TC").expect("plan");
    assert_eq!(plan.front.test_type, TestType::Invariant);
}

#[test]
fn plan_status_change_not_found() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let err = status_change::plan_status_change(&graph, "TC-999", TestStatus::Passing).unwrap_err();
    assert!(matches!(err, crate::error::ProductError::NotFound(_)));
}

#[test]
fn plan_status_change_updates_status_in_content() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![tc("TC-001", "x", TestStatus::Unimplemented)]);
    let plan = status_change::plan_status_change(&graph, "TC-001", TestStatus::Passing).expect("plan");
    assert_eq!(plan.new_status, TestStatus::Passing);
    assert!(plan.test_content.contains("passing"));
}

#[test]
fn plan_runner_config_rejects_unknown_runner() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![tc("TC-001", "x", TestStatus::Unimplemented)]);
    let config = config_with_prereqs(&[]);
    let err = runner_config::plan_runner_config(
        &config,
        &graph,
        "TC-001",
        Some("bogus"),
        None,
        None,
        &[],
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_runner_config_parses_timeout_with_suffix() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![tc("TC-001", "x", TestStatus::Unimplemented)]);
    let config = config_with_prereqs(&[]);
    let plan = runner_config::plan_runner_config(
        &config,
        &graph,
        "TC-001",
        Some("cargo-test"),
        Some("my_test"),
        Some("60s"),
        &[],
        &[],
    )
    .expect("plan");
    assert_eq!(plan.final_runner.as_deref(), Some("cargo-test"));
    assert_eq!(plan.final_args.as_deref(), Some("my_test"));
    assert_eq!(plan.final_timeout, Some(60));
}

#[test]
fn plan_runner_config_rejects_unknown_prereq() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![tc("TC-001", "x", TestStatus::Unimplemented)]);
    let config = config_with_prereqs(&["docker"]);
    let err = runner_config::plan_runner_config(
        &config,
        &graph,
        "TC-001",
        None,
        None,
        None,
        &["not-a-prereq".to_string()],
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, crate::error::ProductError::ConfigError(_)));
}

#[test]
fn plan_runner_config_accepts_known_prereqs() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![tc("TC-001", "x", TestStatus::Unimplemented)]);
    let config = config_with_prereqs(&["docker", "net"]);
    let plan = runner_config::plan_runner_config(
        &config,
        &graph,
        "TC-001",
        None,
        None,
        None,
        &["docker".to_string(), "net".to_string()],
        &[],
    )
    .expect("plan");
    assert!(plan.test_content.contains("docker"));
    assert!(plan.test_content.contains("net"));
}
