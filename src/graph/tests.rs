//! Unit tests for knowledge graph construction, traversal, validation.

use super::*;
use crate::types::*;
use std::path::PathBuf;

fn make_feature(id: &str, deps: Vec<&str>, adrs: Vec<&str>, tests: Vec<&str>, status: FeatureStatus) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: format!("Feature {}", id),
            phase: 1,
            status,
            depends_on: deps.into_iter().map(String::from).collect(),
            adrs: adrs.into_iter().map(String::from).collect(),
            tests: tests.into_iter().map(String::from).collect(),
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
            bundle: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn make_adr(id: &str) -> Adr {
    let title = format!("ADR {}", id);
    let body = String::new();
    let hash = crate::hash::compute_adr_hash(&title, &body);
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title,
            status: AdrStatus::Accepted,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::FeatureSpecific,
            content_hash: Some(hash),
            amendments: vec![],
            source_files: vec![],
        },
        body,
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn make_test(id: &str, adrs: Vec<&str>) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: format!("Test {}", id),
            test_type: TestType::Scenario,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock {
                features: vec![],
                adrs: adrs.into_iter().map(String::from).collect(),
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

#[test]
fn topo_sort_simple() {
    let features = vec![
        make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-003", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    let order = graph.topological_sort().unwrap();
    assert_eq!(order, vec!["FT-001", "FT-002", "FT-003"]);
}

#[test]
fn topo_sort_cycle_detected() {
    let features = vec![
        make_feature("FT-001", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    assert!(graph.topological_sort().is_err());
}

#[test]
fn feature_next_uses_topo() {
    let features = vec![
        make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Complete),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::InProgress),
        make_feature("FT-003", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    let next = graph.feature_next().unwrap();
    // FT-002 is in-progress with deps complete, FT-003 is planned with deps complete
    // Both are valid; topo sort order is deterministic — FT-002 comes first alphabetically
    assert_eq!(next, Some("FT-002".to_string()));
}

#[test]
fn bfs_depth_1() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let tests = vec![make_test("TC-001", vec!["ADR-001"])];
    let graph = KnowledgeGraph::build(features, adrs, tests);
    let reachable = graph.bfs("FT-001", 1);
    assert!(reachable.contains(&"FT-001".to_string()));
    assert!(reachable.contains(&"ADR-001".to_string()));
    assert!(reachable.contains(&"TC-001".to_string()));
}

#[test]
fn impact_analysis() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::InProgress),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let tests = vec![make_test("TC-001", vec!["ADR-001"])];
    let graph = KnowledgeGraph::build(features, adrs, tests);
    let impact = graph.impact("ADR-001");
    assert!(impact.direct_features.contains(&"FT-001".to_string()));
}

#[test]
fn graph_check_broken_link() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-999"], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    let result = graph.check();
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].code == "E002");
}

#[test]
fn graph_check_clean_exits_0() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let mut tc = make_test("TC-001", vec!["ADR-001"]);
    tc.front.test_type = TestType::ExitCriteria;
    tc.front.validates.features = vec!["FT-001".to_string()];
    let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
    let result = graph.check();
    assert_eq!(result.exit_code(), 0, "clean graph should exit 0: errors={:?} warnings={:?}", result.errors, result.warnings);
}

#[test]
fn graph_check_warning_exits_2() {
    // Feature with no tests -> W002
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let graph = KnowledgeGraph::build(features, adrs, vec![]);
    let result = graph.check();
    assert!(result.errors.is_empty(), "should have no errors");
    assert!(!result.warnings.is_empty(), "should have warnings");
    assert_eq!(result.exit_code(), 2);
}

#[test]
fn graph_check_e003_cycle() {
    let features = vec![
        make_feature("FT-001", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    let result = graph.check();
    assert!(result.errors.iter().any(|e| e.code == "E003"), "should detect cycle E003");
    assert_eq!(result.exit_code(), 1);
}

#[test]
fn graph_check_w001_orphaned_adr() {
    let features = vec![
        make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")]; // not linked to any feature
    let graph = KnowledgeGraph::build(features, adrs, vec![]);
    let result = graph.check();
    assert!(result.warnings.iter().any(|w| w.code == "W001"), "should report orphan W001");
}

#[test]
fn graph_check_w002_no_tests() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let graph = KnowledgeGraph::build(features, adrs, vec![]);
    let result = graph.check();
    assert!(result.warnings.iter().any(|w| w.code == "W002"), "should report no-tests W002");
}

#[test]
fn graph_check_w003_no_exit_criteria() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let tests = vec![make_test("TC-001", vec!["ADR-001"])]; // type=Scenario, not ExitCriteria
    let graph = KnowledgeGraph::build(features, adrs, tests);
    let result = graph.check();
    assert!(result.warnings.iter().any(|w| w.code == "W003"), "should report W003");
}

#[test]
fn graph_check_w016_complete_with_unimplemented() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Complete),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let mut tc = make_test("TC-001", vec!["ADR-001"]);
    tc.front.validates.features = vec!["FT-001".to_string()];
    // TC defaults to Unimplemented status
    let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
    let result = graph.check();
    assert!(result.warnings.iter().any(|w| w.code == "W016"), "should report W016 for complete feature with unimplemented TC");
}

#[test]
fn graph_check_w016_not_fired_when_passing() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Complete),
    ];
    let adrs = vec![make_adr("ADR-001")];
    let mut tc = make_test("TC-001", vec!["ADR-001"]);
    tc.front.validates.features = vec!["FT-001".to_string()];
    tc.front.test_type = TestType::ExitCriteria;
    tc.front.status = TestStatus::Passing;
    let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
    let result = graph.check();
    assert!(!result.warnings.iter().any(|w| w.code == "W016"), "should not fire W016 when all TCs passing");
}

#[test]
fn graph_check_w005_phase_dep_mismatch() {
    let mut f1 = make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned);
    f1.front.phase = 2; // dependency in phase 2
    let mut f2 = make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned);
    f2.front.phase = 1; // feature in phase 1 depends on phase 2 feature
    let graph = KnowledgeGraph::build(vec![f1, f2], vec![], vec![]);
    let result = graph.check();
    assert!(result.warnings.iter().any(|w| w.code == "W005"), "should report W005 phase mismatch");
}

#[test]
fn centrality_returns_values() {
    let features = vec![
        make_feature("FT-001", vec![], vec!["ADR-001", "ADR-002"], vec![], FeatureStatus::Planned),
        make_feature("FT-002", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
    ];
    let adrs = vec![make_adr("ADR-001"), make_adr("ADR-002")];
    let graph = KnowledgeGraph::build(features, adrs, vec![]);
    let centrality = graph.betweenness_centrality();
    // ADR-001 is linked to both features, should have higher centrality
    let c1 = centrality.get("ADR-001").copied().unwrap_or(0.0);
    let c2 = centrality.get("ADR-002").copied().unwrap_or(0.0);
    assert!(c1 >= 0.0 && c1 <= 1.0, "centrality should be in [0,1]");
    assert!(c2 >= 0.0 && c2 <= 1.0, "centrality should be in [0,1]");
}

#[test]
fn topo_sort_parallel() {
    // FT-002 and FT-003 both depend on FT-001, no dependency between them
    let features = vec![
        make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        make_feature("FT-003", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
    ];
    let graph = KnowledgeGraph::build(features, vec![], vec![]);
    let order = graph.topological_sort().unwrap();
    let pos1 = order.iter().position(|id| id == "FT-001").unwrap();
    let pos2 = order.iter().position(|id| id == "FT-002").unwrap();
    let pos3 = order.iter().position(|id| id == "FT-003").unwrap();
    assert!(pos1 < pos2, "FT-001 must come before FT-002");
    assert!(pos1 < pos3, "FT-001 must come before FT-003");
}
