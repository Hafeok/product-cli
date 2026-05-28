//! Unit tests for `feature::link::plan_link`.

#![cfg(test)]

use super::link::*;
use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Adr, AdrFrontMatter, AdrScope, AdrStatus, Feature, FeatureFrontMatter, FeatureStatus,
    TestCriterion, TestFrontMatter, TestStatus, TestType, ValidatesBlock,
};
use std::collections::HashMap;
use std::path::PathBuf;

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

fn adr(id: &str) -> Adr {
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title: format!("adr {}", id),
            status: AdrStatus::Proposed,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::Domain,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
            removes: vec![],
            deprecates: vec![],
        },
        body: String::new(),
        path: PathBuf::from(format!("docs/adrs/{}.md", id)),
    }
}

fn tc(id: &str) -> TestCriterion {
    TestCriterion {
        front: TestFrontMatter {
            id: id.to_string(),
            title: format!("tc {}", id),
            test_type: TestType::Scenario,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock { features: vec![], adrs: vec![] },
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
        path: PathBuf::from(format!("docs/tests/{}.md", id)),
        formal_blocks: vec![],
    }
}

#[test]
fn unknown_feature_returns_not_found() {
    let g = KnowledgeGraph::build(vec![], vec![], vec![]);
    let err = plan_link(&g, "FT-001", None, None).unwrap_err();
    assert!(matches!(err, ProductError::NotFound(_)));
}

#[test]
fn unknown_adr_target_returns_not_found_before_any_write() {
    let g = KnowledgeGraph::build(vec![feat("FT-001")], vec![], vec![]);
    let err = plan_link(&g, "FT-001", Some("ADR-999"), None).unwrap_err();
    assert!(matches!(err, ProductError::NotFound(_)));
}

#[test]
fn unknown_tc_target_returns_not_found_before_any_write() {
    let g = KnowledgeGraph::build(vec![feat("FT-001")], vec![], vec![]);
    let err = plan_link(&g, "FT-001", None, Some("TC-999")).unwrap_err();
    assert!(matches!(err, ProductError::NotFound(_)));
}

#[test]
fn link_to_tc_emits_reciprocal_write() {
    let g = KnowledgeGraph::build(vec![feat("FT-001")], vec![], vec![tc("TC-001")]);
    let plan = plan_link(&g, "FT-001", None, Some("TC-001")).unwrap();
    assert_eq!(plan.writes.len(), 2, "feature + TC");
    assert_eq!(plan.reciprocated.len(), 1);
    assert_eq!(plan.reciprocated[0].id, "TC-001");
    assert_eq!(plan.reciprocated[0].field, "validates.features");
}

#[test]
fn link_to_adr_emits_reciprocal_write() {
    let g = KnowledgeGraph::build(vec![feat("FT-001")], vec![adr("ADR-001")], vec![]);
    let plan = plan_link(&g, "FT-001", Some("ADR-001"), None).unwrap();
    assert_eq!(plan.writes.len(), 2, "feature + ADR");
    assert_eq!(plan.reciprocated.len(), 1);
    assert_eq!(plan.reciprocated[0].id, "ADR-001");
    assert_eq!(plan.reciprocated[0].field, "features");
}

#[test]
fn link_to_both_emits_three_writes() {
    let g = KnowledgeGraph::build(
        vec![feat("FT-001")],
        vec![adr("ADR-001")],
        vec![tc("TC-001")],
    );
    let plan = plan_link(&g, "FT-001", Some("ADR-001"), Some("TC-001")).unwrap();
    assert_eq!(plan.writes.len(), 3);
    assert_eq!(plan.reciprocated.len(), 2);
    // Feature write must be first.
    assert_eq!(plan.writes[0].kind, LinkWriteKind::Feature);
}

#[test]
fn idempotent_link_is_a_noop_plan() {
    // Pre-link both sides; subsequent call should produce no writes.
    let mut f = feat("FT-001");
    f.front.adrs.push("ADR-001".to_string());
    f.front.tests.push("TC-001".to_string());
    let mut a = adr("ADR-001");
    a.front.features.push("FT-001".to_string());
    let mut t = tc("TC-001");
    t.front.validates.features.push("FT-001".to_string());
    let g = KnowledgeGraph::build(vec![f], vec![a], vec![t]);
    let plan = plan_link(&g, "FT-001", Some("ADR-001"), Some("TC-001")).unwrap();
    assert!(plan.writes.is_empty());
    assert!(plan.reciprocated.is_empty());
    assert!(!plan.is_changed());
}

#[test]
fn already_linked_on_feature_side_still_reciprocates() {
    // Feature has the link but TC's back-reference is empty (legacy data).
    let mut f = feat("FT-001");
    f.front.tests.push("TC-001".to_string());
    let g = KnowledgeGraph::build(vec![f], vec![], vec![tc("TC-001")]);
    let plan = plan_link(&g, "FT-001", None, Some("TC-001")).unwrap();
    assert_eq!(plan.writes.len(), 1, "only the TC needs writing");
    assert_eq!(plan.writes[0].kind, LinkWriteKind::Tc);
    assert_eq!(plan.reciprocated.len(), 1);
}
