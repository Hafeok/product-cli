//! FT-067: one-shot helper that suggests `cross-cutting` → `platform`
//! re-classifications for ADRs whose evidence already matches the
//! "enforced by the platform itself" pattern.
//!
//! The heuristic is conservative: an ADR is suggested for `platform` when
//! all three signals are present:
//!
//! 1. It currently carries `scope: cross-cutting`.
//! 2. It has zero `features:` backlinks (no feature has explicitly opted
//!    in to it — consistent with W027's "cross-cutting ADRs usually
//!    don't list specific features" rule).
//! 3. Every linked TC is `invariant`, `chaos`, or `absence` — the three
//!    TC types whose runners enforce a project-wide assertion rather
//!    than scenario-style per-feature behaviour.
//!
//! The audit never modifies files on its own; the caller must pass
//! `--apply` to commit the suggestions, and even then each rewrite is a
//! per-file atomic write so a partial run leaves a coherent on-disk
//! state.

use crate::error::ProductError;
use crate::fileops;
use crate::graph::KnowledgeGraph;
use crate::{parser, types};
use std::path::PathBuf;

/// A single ADR's audit verdict.
#[derive(Debug, Clone)]
pub struct AuditSuggestion {
    pub adr_id: String,
    pub adr_title: String,
    pub adr_path: PathBuf,
    pub current_scope: types::AdrScope,
    pub suggested_scope: types::AdrScope,
    pub reason: String,
    pub linked_tc_count: usize,
}

/// Result of running the audit.
#[derive(Debug, Clone, Default)]
pub struct AuditPlan {
    pub suggestions: Vec<AuditSuggestion>,
    pub reviewed: usize,
}

/// Compute audit suggestions over every cross-cutting ADR in the graph.
///
/// This is pure: no I/O, no print, no enum mutation.
pub fn plan_audit(graph: &KnowledgeGraph) -> AuditPlan {
    let mut suggestions = Vec::new();
    let mut reviewed = 0usize;

    for adr in graph.adrs.values() {
        if adr.front.scope != types::AdrScope::CrossCutting {
            continue;
        }
        reviewed += 1;

        let linked_tcs: Vec<&types::TestCriterion> = graph
            .tests
            .values()
            .filter(|t| t.front.validates.adrs.contains(&adr.front.id))
            .collect();

        let no_feature_backlinks = adr.front.features.is_empty();
        let all_platform_style_tcs = !linked_tcs.is_empty()
            && linked_tcs.iter().all(|t| {
                matches!(
                    t.front.test_type,
                    types::TestType::Invariant
                        | types::TestType::Chaos
                        | types::TestType::Absence
                )
            });

        if no_feature_backlinks && all_platform_style_tcs {
            suggestions.push(AuditSuggestion {
                adr_id: adr.front.id.clone(),
                adr_title: adr.front.title.clone(),
                adr_path: adr.path.clone(),
                current_scope: types::AdrScope::CrossCutting,
                suggested_scope: types::AdrScope::Platform,
                reason: format!(
                    "no feature backlinks, {} linked TC(s) all invariant/chaos/absence",
                    linked_tcs.len()
                ),
                linked_tc_count: linked_tcs.len(),
            });
        }
    }

    suggestions.sort_by(|a, b| a.adr_id.cmp(&b.adr_id));
    AuditPlan { suggestions, reviewed }
}

/// Apply every suggestion in the plan by rewriting each ADR's `scope:`
/// field. Each file write is atomic — a failure aborts on the offending
/// path with its path printed (per the FT-067 spec: "files are written
/// atomically per-file via `fileops::atomic_write`; the user re-runs after
/// fixing").
pub fn apply_audit(
    plan: &AuditPlan,
    graph: &KnowledgeGraph,
) -> Result<usize, ProductError> {
    let mut applied = 0usize;
    for s in &plan.suggestions {
        let adr = graph
            .adrs
            .get(&s.adr_id)
            .ok_or_else(|| ProductError::NotFound(format!("ADR {}", s.adr_id)))?;
        let mut front = adr.front.clone();
        front.scope = s.suggested_scope;
        let content = parser::render_adr(&front, &adr.body);
        fileops::write_file_atomic(&s.adr_path, &content).map_err(|e| {
            ProductError::ConfigError(format!(
                "scope-audit: failed to rewrite {}: {}",
                s.adr_path.display(),
                e
            ))
        })?;
        applied += 1;
    }
    Ok(applied)
}

/// Render the dry-run report.
pub fn render_audit(plan: &AuditPlan, apply_mode: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Scope audit: reviewed {} cross-cutting ADR(s).\n",
        plan.reviewed
    ));
    if plan.suggestions.is_empty() {
        out.push_str("\nNo platform re-classifications suggested.\n");
        return out;
    }

    out.push_str("\nSuggested cross-cutting \u{2192} platform re-classifications:\n");
    for s in &plan.suggestions {
        out.push_str(&format!(
            "  \u{2022} {:<10} {} \u{2192} platform\n      reason: {}\n",
            s.adr_id, s.adr_title, s.reason
        ));
    }

    out.push('\n');
    if apply_mode {
        out.push_str(&format!(
            "Applied {} suggestion(s).\n",
            plan.suggestions.len()
        ));
    } else {
        out.push_str("(dry-run — re-run with --apply to commit changes)\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    fn adr_with_scope(id: &str, scope: AdrScope, features: Vec<&str>) -> Adr {
        Adr {
            front: AdrFrontMatter {
                id: id.to_string(),
                title: format!("ADR {}", id),
                status: AdrStatus::Accepted,
                features: features.into_iter().map(String::from).collect(),
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
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn tc_with_type(id: &str, ty: TestType, adr: &str) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: id.to_string(),
                title: format!("TC {}", id),
                test_type: ty,
                status: TestStatus::Passing,
                validates: ValidatesBlock {
                    features: vec![],
                    adrs: vec![adr.to_string()],
                },
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

    fn feat_with_adr(id: &str, adr: &str) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("{}", id),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![adr.to_string()],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: HashMap::new(),
                patterns: vec![],
                due_date: None,
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    #[test]
    fn suggests_platform_when_no_feature_backlinks_and_only_invariant_tcs() {
        let a = adr_with_scope("ADR-001", AdrScope::CrossCutting, vec![]);
        let t1 = tc_with_type("TC-001", TestType::Invariant, "ADR-001");
        let t2 = tc_with_type("TC-002", TestType::Absence, "ADR-001");
        let graph = KnowledgeGraph::build(vec![], vec![a], vec![t1, t2]);
        let plan = plan_audit(&graph);
        assert_eq!(plan.suggestions.len(), 1);
        assert_eq!(plan.suggestions[0].adr_id, "ADR-001");
        assert_eq!(plan.suggestions[0].suggested_scope, AdrScope::Platform);
    }

    #[test]
    fn skips_when_feature_backlinks_present() {
        let a = adr_with_scope("ADR-001", AdrScope::CrossCutting, vec!["FT-001"]);
        let t = tc_with_type("TC-001", TestType::Invariant, "ADR-001");
        let graph = KnowledgeGraph::build(vec![feat_with_adr("FT-001", "ADR-001")], vec![a], vec![t]);
        let plan = plan_audit(&graph);
        assert!(plan.suggestions.is_empty());
        assert_eq!(plan.reviewed, 1);
    }

    #[test]
    fn skips_when_scenario_tc_present() {
        let a = adr_with_scope("ADR-001", AdrScope::CrossCutting, vec![]);
        let t1 = tc_with_type("TC-001", TestType::Invariant, "ADR-001");
        let t2 = tc_with_type("TC-002", TestType::Scenario, "ADR-001");
        let graph = KnowledgeGraph::build(vec![], vec![a], vec![t1, t2]);
        let plan = plan_audit(&graph);
        assert!(plan.suggestions.is_empty());
    }

    #[test]
    fn skips_when_no_tcs_linked() {
        let a = adr_with_scope("ADR-001", AdrScope::CrossCutting, vec![]);
        let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
        let plan = plan_audit(&graph);
        // No linked TCs — can't safely conclude "enforced by platform"; skip.
        assert!(plan.suggestions.is_empty());
    }

    #[test]
    fn ignores_non_cross_cutting_adrs() {
        let a = adr_with_scope("ADR-001", AdrScope::FeatureSpecific, vec![]);
        let graph = KnowledgeGraph::build(vec![], vec![a], vec![]);
        let plan = plan_audit(&graph);
        assert!(plan.suggestions.is_empty());
        assert_eq!(plan.reviewed, 0);
    }
}
