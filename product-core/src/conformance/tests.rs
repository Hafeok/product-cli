//! Unit tests for the Two Pillars conformance checks.

use super::*;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Adr, AdrFrontMatter, AdrScope, AdrStatus, Feature, FeatureFrontMatter, FeatureStatus,
    TestCriterion, TestFrontMatter, TestStatus, TestType, ValidatesBlock,
};
use std::collections::HashMap;
use std::path::PathBuf;

const CONFORMING_BODY: &str = "\
## Description

prose

## Functional Specification

### Behaviour

does the thing

### Error handling

fails loudly

## Out of scope

- everything else
";

fn project() -> ProjectDeclarations {
    ProjectDeclarations {
        name: "test".into(),
        responsibility: Some("A test system.".into()),
        features_path: "docs/features".into(),
        adrs_path: "docs/adrs".into(),
    }
}

fn feature(id: &str, status: FeatureStatus, tests: Vec<String>, body: &str) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: format!("feature {}", id),
            phase: 1,
            status,
            depends_on: vec![],
            adrs: vec![],
            tests,
            domains: vec![],
            domains_acknowledged: HashMap::new(),
            patterns: vec![],
            due_date: None,
            bundle: None,
            adrs_rejected: vec![],
        },
        body: body.to_string(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn adr(id: &str, title: &str, status: AdrStatus, scope: AdrScope, features: Vec<String>) -> Adr {
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title: title.to_string(),
            status,
            features,
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
            removes: vec![],
            deprecates: vec![],
        },
        body: "## Decision\n\nx\n\n## Rejected alternatives\n\n- y\n".to_string(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

fn tc(id: &str, status: TestStatus, features: Vec<String>) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: format!("test {}", id),
            test_type: TestType::Scenario,
            status,
            validates: ValidatesBlock { features, adrs: vec![] },
            phase: 1,
            content_hash: None,
            runner: None,
            runner_args: None,
            runner_timeout: None,
            requires: vec![],
            observes: vec![],
            last_run: None,
            failure_message: None,
            last_run_duration: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
        formal_blocks: vec![],
    }
}

fn clauses_in(report: &ConformanceReport) -> Vec<&str> {
    report.findings.iter().map(|f| f.clause.as_str()).collect()
}

#[test]
fn conforming_graph_reports_level_3() {
    let f = feature("FT-001", FeatureStatus::Planned, vec!["TC-001".into()], CONFORMING_BODY);
    let a = adr("ADR-001", "Use a thing", AdrStatus::Accepted, AdrScope::FeatureSpecific, vec!["FT-001".into()]);
    let t = tc("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![t]);

    let report = check(&graph, &project());
    assert!(report.findings.is_empty(), "unexpected findings: {:?}", report.findings);
    assert_eq!(report.profile, "level-3");
    assert!(!report.has_violations());
    assert_eq!(report.summary.clauses_passed, report.summary.clauses_checked);
}

#[test]
fn missing_identity_or_purpose_violates_what_1_what_2() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let bare = ProjectDeclarations::default();
    let report = check(&graph, &bare);
    let clauses = clauses_in(&report);
    assert!(clauses.contains(&"SPEC-WHAT-1"));
    assert!(clauses.contains(&"SPEC-WHAT-2"));
    assert_eq!(report.profile, "below-level-3");
}

#[test]
fn shared_artifact_directory_violates_split_1() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let mut p = project();
    p.adrs_path = p.features_path.clone();
    let report = check(&graph, &p);
    assert!(clauses_in(&report).contains(&"SPEC-SPLIT-1"));
}

#[test]
fn missing_body_sections_violate_what_4_what_5() {
    let f = feature("FT-001", FeatureStatus::Planned, vec!["TC-001".into()], "## Description\n\nx\n");
    let t = tc("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t]);
    let report = check(&graph, &project());
    let clauses = clauses_in(&report);
    assert!(clauses.contains(&"SPEC-WHAT-4"));
    assert!(clauses.contains(&"SPEC-WHAT-5"));
}

#[test]
fn missing_error_handling_subsection_names_it() {
    let body = "## Functional Specification\n\n### Behaviour\n\nx\n\n## Out of scope\n\n- y\n";
    let f = feature("FT-001", FeatureStatus::Planned, vec!["TC-001".into()], body);
    let t = tc("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t]);
    let report = check(&graph, &project());
    let w4 = report.findings.iter().find(|f| f.clause == "SPEC-WHAT-4").expect("SPEC-WHAT-4");
    assert!(w4.description.contains("Error handling"));
    assert!(!w4.description.contains("Behaviour,"));
}

#[test]
fn feature_without_tc_violates_what_8() {
    let f = feature("FT-001", FeatureStatus::Planned, vec![], CONFORMING_BODY);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let report = check(&graph, &project());
    assert!(clauses_in(&report).contains(&"SPEC-WHAT-8"));
}

#[test]
fn tc_linked_by_validates_satisfies_what_8() {
    let f = feature("FT-001", FeatureStatus::Planned, vec![], CONFORMING_BODY);
    let t = tc("TC-001", TestStatus::Unimplemented, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t]);
    let report = check(&graph, &project());
    assert!(!clauses_in(&report).contains(&"SPEC-WHAT-8"));
}

#[test]
fn abandoned_feature_exempt_from_what_pillar() {
    let f = feature("FT-001", FeatureStatus::Abandoned, vec![], "");
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let report = check(&graph, &project());
    assert!(report.findings.is_empty());
}

#[test]
fn complete_feature_with_failing_tc_violates_exec_close_4() {
    let f = feature("FT-001", FeatureStatus::Complete, vec!["TC-001".into()], CONFORMING_BODY);
    let t = tc("TC-001", TestStatus::Failing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t]);
    let report = check(&graph, &project());
    let f4 = report.findings.iter().find(|f| f.clause == "EXEC-CLOSE-4").expect("EXEC-CLOSE-4");
    assert_eq!(f4.severity, ClauseSeverity::Violation);
    assert!(f4.description.contains("TC-001"));
}

#[test]
fn complete_feature_with_passing_tcs_satisfies_exec_close_4() {
    let f = feature("FT-001", FeatureStatus::Complete, vec!["TC-001".into()], CONFORMING_BODY);
    let t = tc("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![t]);
    let report = check(&graph, &project());
    assert!(!clauses_in(&report).contains(&"EXEC-CLOSE-4"));
}

#[test]
fn unanchored_feature_specific_adr_violates_derive_3() {
    let a = adr("ADR-001", "Use a thing", AdrStatus::Accepted, AdrScope::FeatureSpecific, vec![]);
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let report = check(&graph, &project());
    assert!(clauses_in(&report).contains(&"SPEC-DERIVE-3"));
}

#[test]
fn cross_cutting_adr_exempt_from_derive_3() {
    let a = adr("ADR-001", "Use a thing", AdrStatus::Accepted, AdrScope::CrossCutting, vec![]);
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let report = check(&graph, &project());
    assert!(!clauses_in(&report).contains(&"SPEC-DERIVE-3"));
}

#[test]
fn adr_anchored_via_feature_side_satisfies_derive_3() {
    let mut f = feature("FT-001", FeatureStatus::Planned, vec!["TC-001".into()], CONFORMING_BODY);
    f.front.adrs = vec!["ADR-001".into()];
    let a = adr("ADR-001", "Use a thing", AdrStatus::Accepted, AdrScope::FeatureSpecific, vec![]);
    let t = tc("TC-001", TestStatus::Passing, vec!["FT-001".into()]);
    let graph = KnowledgeGraph::build(vec![f], vec![a], vec![t]);
    let report = check(&graph, &project());
    assert!(!clauses_in(&report).contains(&"SPEC-DERIVE-3"));
}

#[test]
fn adr_without_rejected_alternatives_violates_how_5() {
    let mut a = adr("ADR-001", "Use a thing", AdrStatus::Accepted, AdrScope::CrossCutting, vec![]);
    a.body = "## Decision\n\nx\n".to_string();
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let report = check(&graph, &project());
    assert!(clauses_in(&report).contains(&"SPEC-HOW-5"));
}

#[test]
fn proposed_adr_exempt_from_how_pillar() {
    let mut a = adr("ADR-001", "Use a thing and another", AdrStatus::Proposed, AdrScope::FeatureSpecific, vec![]);
    a.body = String::new();
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let report = check(&graph, &project());
    assert!(report.findings.is_empty());
}

#[test]
fn conjoined_adr_title_is_advisory_how_2_1() {
    let a = adr("ADR-001", "Parse YAML and build the graph", AdrStatus::Accepted, AdrScope::CrossCutting, vec![]);
    let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
    let report = check(&graph, &project());
    let f21 = report.findings.iter().find(|f| f.clause == "SPEC-HOW-2.1").expect("SPEC-HOW-2.1");
    assert_eq!(f21.severity, ClauseSeverity::Advisory);
    // Advisories alone do not break Level 3 conformance or fail the clause.
    assert_eq!(report.profile, "level-3");
    assert!(!report.has_violations());
    let outcome = report.clauses.iter().find(|c| c.clause == "SPEC-HOW-2.1").expect("outcome");
    assert!(outcome.passed);
    assert_eq!(outcome.findings, 1);
}

#[test]
fn report_renders_text_with_verdict() {
    let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
    let report = check(&graph, &project());
    let text = render_report_text(&report, "test");
    assert!(text.contains("Two Pillars conformance"));
    assert!(text.contains("SPEC-WHAT-1"));
    assert!(text.contains("conforms to Level 3"));
}

#[test]
fn json_report_serializes_clauses_findings_profile() {
    let f = feature("FT-001", FeatureStatus::Planned, vec![], "");
    let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
    let report = check(&graph, &project());
    let json = serde_json::to_value(&report).expect("serialize");
    assert_eq!(json["spec"], "two-pillars/0.1");
    assert_eq!(json["profile"], "below-level-3");
    assert!(json["clauses"].as_array().map(|c| !c.is_empty()).unwrap_or(false));
    assert!(json["findings"].as_array().map(|f| !f.is_empty()).unwrap_or(false));
}
