//! Predicate: when must a TC carry runner configuration? (FT-058 / ADR-021).
//!
//! A TC linked to a feature whose status is `in-progress` or `complete`
//! must declare both `runner` and `runner-args` in its front-matter.
//! TCs linked only to `planned` or `abandoned` features are exempt —
//! they are sketches, not executable specifications.
//!
//! This module exposes two pure predicates:
//!
//! - [`must_have_runner`] — does this single TC need runner config given
//!   the feature status it would be reported under?
//! - [`find_offenders`] — enumerate every TC linked to a feature whose
//!   `(target_status, current runner config)` pair triggers E022.
//!
//! Both predicates are pure functions over the in-memory graph and the
//! candidate feature status. Callers (verify, preflight, graph check,
//! status transition, request apply) wrap the result in their own
//! diagnostic surface.

use crate::graph::KnowledgeGraph;
use crate::types::{FeatureStatus, TestCriterion};

/// True iff a TC must declare both `runner` and `runner-args`, given the
/// feature status it would be reported under.
///
/// The rule mirrors ADR-021's amendment: runner config is required for
/// `in-progress` and `complete` features. `planned` and `abandoned`
/// features are exempt (sketches and cancelled work, respectively).
pub fn must_have_runner(tc: &TestCriterion, target_feature_status: FeatureStatus) -> bool {
    if !status_requires_runner(target_feature_status) {
        return false;
    }
    !has_runner_config(tc)
}

/// Enumerate every TC linked to `feature_id` that lacks runner config,
/// given the candidate feature status. Returns a sorted, deduplicated
/// list of TC IDs.
///
/// When `target_status` is `planned` or `abandoned`, the result is
/// always empty — these statuses are exempt from the invariant.
pub fn find_offenders(
    graph: &KnowledgeGraph,
    feature_id: &str,
    target_status: FeatureStatus,
) -> Vec<String> {
    if !status_requires_runner(target_status) {
        return Vec::new();
    }
    let Some(feature) = graph.features.get(feature_id) else {
        return Vec::new();
    };
    let mut offenders: Vec<String> = feature
        .front
        .tests
        .iter()
        .filter_map(|tc_id| graph.tests.get(tc_id.as_str()))
        .filter(|tc| !has_runner_config(tc))
        .map(|tc| tc.front.id.clone())
        .collect();
    offenders.sort();
    offenders.dedup();
    offenders
}

/// True iff the given feature status requires every linked TC to carry
/// runner configuration. `in-progress` and `complete` are required;
/// `planned` and `abandoned` are exempt.
pub fn status_requires_runner(status: FeatureStatus) -> bool {
    matches!(status, FeatureStatus::InProgress | FeatureStatus::Complete)
}

/// True iff the TC has both `runner` and `runner-args` populated.
fn has_runner_config(tc: &TestCriterion) -> bool {
    let runner_ok = tc
        .front
        .runner
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let args_ok = tc
        .front
        .runner_args
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    runner_ok && args_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraph;
    use crate::types::*;
    use std::path::PathBuf;

    fn mk_tc(id: &str, runner: Option<&str>, args: Option<&str>) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: id.to_string(),
                title: format!("title for {}", id),
                test_type: TestType::Scenario,
                status: TestStatus::Unimplemented,
                validates: ValidatesBlock::default(),
                phase: 1,
                content_hash: None,
                runner: runner.map(|s| s.to_string()),
                runner_args: args.map(|s| s.to_string()),
                runner_timeout: None,
                requires: vec![],
                observes: vec![],
                last_run: None,
                failure_message: None,
                last_run_duration: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/tests/{}-x.md", id)),
            formal_blocks: vec![],
        }
    }

    fn mk_feature(id: &str, status: FeatureStatus, tests: Vec<&str>) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("title for {}", id),
                phase: 1,
                status,
                depends_on: vec![],
                adrs: vec![],
                tests: tests.into_iter().map(String::from).collect(),
                domains: vec![],
                domains_acknowledged: Default::default(),
                patterns: vec![],
                due_date: None,
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/features/{}-x.md", id)),
        }
    }

    #[test]
    fn must_have_runner_true_for_in_progress_without_config() {
        let tc = mk_tc("TC-001", None, None);
        assert!(must_have_runner(&tc, FeatureStatus::InProgress));
    }

    #[test]
    fn must_have_runner_false_for_planned_without_config() {
        let tc = mk_tc("TC-001", None, None);
        assert!(!must_have_runner(&tc, FeatureStatus::Planned));
    }

    #[test]
    fn must_have_runner_false_for_abandoned_without_config() {
        let tc = mk_tc("TC-001", None, None);
        assert!(!must_have_runner(&tc, FeatureStatus::Abandoned));
    }

    #[test]
    fn must_have_runner_false_when_fully_configured() {
        let tc = mk_tc("TC-001", Some("cargo-test"), Some("tc_001_x"));
        assert!(!must_have_runner(&tc, FeatureStatus::InProgress));
        assert!(!must_have_runner(&tc, FeatureStatus::Complete));
    }

    #[test]
    fn must_have_runner_true_when_runner_present_but_args_missing() {
        let tc = mk_tc("TC-001", Some("cargo-test"), None);
        assert!(must_have_runner(&tc, FeatureStatus::InProgress));
    }

    #[test]
    fn must_have_runner_true_when_args_present_but_runner_missing() {
        let tc = mk_tc("TC-001", None, Some("tc_001_x"));
        assert!(must_have_runner(&tc, FeatureStatus::InProgress));
    }

    #[test]
    fn must_have_runner_treats_blank_strings_as_missing() {
        let tc = mk_tc("TC-001", Some("   "), Some("tc_001_x"));
        assert!(must_have_runner(&tc, FeatureStatus::InProgress));
    }

    #[test]
    fn find_offenders_lists_all_unconfigured_tcs_sorted() {
        let f = mk_feature(
            "FT-001",
            FeatureStatus::InProgress,
            vec!["TC-001", "TC-002", "TC-003"],
        );
        let tcs = vec![
            mk_tc("TC-001", Some("cargo-test"), Some("tc_001_x")), // ok
            mk_tc("TC-002", None, None),                            // both missing
            mk_tc("TC-003", Some("cargo-test"), None),              // args missing
        ];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        let offenders = find_offenders(&graph, "FT-001", FeatureStatus::InProgress);
        assert_eq!(offenders, vec!["TC-002".to_string(), "TC-003".to_string()]);
    }

    #[test]
    fn find_offenders_empty_when_feature_planned() {
        let f = mk_feature("FT-001", FeatureStatus::Planned, vec!["TC-001"]);
        let tcs = vec![mk_tc("TC-001", None, None)];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        assert!(find_offenders(&graph, "FT-001", FeatureStatus::Planned).is_empty());
    }

    #[test]
    fn find_offenders_empty_when_feature_abandoned() {
        let f = mk_feature("FT-001", FeatureStatus::Abandoned, vec!["TC-001"]);
        let tcs = vec![mk_tc("TC-001", None, None)];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        assert!(find_offenders(&graph, "FT-001", FeatureStatus::Abandoned).is_empty());
    }

    #[test]
    fn find_offenders_empty_for_unknown_feature() {
        let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
        assert!(find_offenders(&graph, "FT-999", FeatureStatus::InProgress).is_empty());
    }

    #[test]
    fn find_offenders_uses_target_status_not_current_status() {
        // Feature is currently planned, but target is in-progress — invariant fires.
        let f = mk_feature("FT-001", FeatureStatus::Planned, vec!["TC-001"]);
        let tcs = vec![mk_tc("TC-001", None, None)];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        let offenders = find_offenders(&graph, "FT-001", FeatureStatus::InProgress);
        assert_eq!(offenders, vec!["TC-001".to_string()]);
    }
}
