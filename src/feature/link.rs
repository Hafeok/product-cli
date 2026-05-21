//! Feature link operations — bidirectional ADR/TC linking with reciprocation (FT-066).
//!
//! `plan_link` is the pure planning function the CLI adapter and the MCP
//! handler both call. It produces a `LinkPlan` that carries every write
//! needed to keep the graph bidirectional: the feature's own `adrs:` /
//! `tests:` arrays, plus the target ADR's `features:` array and/or the
//! target TC's `validates.features` array.
//!
//! `apply_link` writes the whole plan atomically via `write_batch_atomic`,
//! so a partial failure cannot leave one side of a reciprocal link
//! orphaned.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser};
use std::path::PathBuf;

/// A single file the plan will touch.
#[derive(Debug, Clone)]
pub struct LinkWrite {
    pub path: PathBuf,
    pub content: String,
    pub kind: LinkWriteKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkWriteKind {
    Feature,
    Adr,
    Tc,
}

impl LinkWriteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Adr => "adr",
            Self::Tc => "tc",
        }
    }
}

/// A reciprocal back-reference that the plan filled in on a link target.
#[derive(Debug, Clone)]
pub struct LinkReciprocation {
    pub id: String,
    pub field: &'static str,
}

/// Plan describing every write needed for a `product_feature_link` call.
#[derive(Debug, Clone)]
pub struct LinkPlan {
    pub feature_id: String,
    pub writes: Vec<LinkWrite>,
    pub reciprocated: Vec<LinkReciprocation>,
}

impl LinkPlan {
    pub fn is_changed(&self) -> bool {
        !self.writes.is_empty()
    }
}

/// Pure: produce a `LinkPlan` from optional ADR / TC link arguments.
///
/// - Validates that the feature, ADR, and TC targets all exist (E002 /
///   `NotFound` before any write).
/// - Computes the feature-side update and the reciprocal back-reference
///   on each target in one pass.
/// - Idempotent: if a link is already present on both sides, no writes
///   are emitted for that target. If a link exists on the feature side
///   only (legacy data), the reciprocal write is still emitted.
pub fn plan_link(
    graph: &KnowledgeGraph,
    feature_id: &str,
    adr: Option<&str>,
    test: Option<&str>,
) -> Result<LinkPlan, ProductError> {
    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    // Validate targets exist before computing any writes — we never want
    // to half-apply a plan because one of the targets was a typo.
    if let Some(adr_id) = adr {
        if !graph.adrs.contains_key(adr_id) {
            return Err(ProductError::NotFound(format!("ADR {}", adr_id)));
        }
    }
    if let Some(tc_id) = test {
        if !graph.tests.contains_key(tc_id) {
            return Err(ProductError::NotFound(format!("test {}", tc_id)));
        }
    }

    let mut front = feature.front.clone();
    let mut writes = Vec::new();
    let mut reciprocated = Vec::new();
    let mut feature_changed = false;

    if let Some(adr_id) = adr {
        if !front.adrs.contains(&adr_id.to_string()) {
            front.adrs.push(adr_id.to_string());
            feature_changed = true;
        }
        if let Some(adr) = graph.adrs.get(adr_id) {
            if !adr.front.features.contains(&feature_id.to_string()) {
                let mut adr_front = adr.front.clone();
                adr_front.features.push(feature_id.to_string());
                let content = parser::render_adr(&adr_front, &adr.body);
                writes.push(LinkWrite {
                    path: adr.path.clone(),
                    content,
                    kind: LinkWriteKind::Adr,
                });
                reciprocated.push(LinkReciprocation {
                    id: adr_id.to_string(),
                    field: "features",
                });
            }
        }
    }

    if let Some(tc_id) = test {
        if !front.tests.contains(&tc_id.to_string()) {
            front.tests.push(tc_id.to_string());
            feature_changed = true;
        }
        if let Some(tc) = graph.tests.get(tc_id) {
            if !tc.front.validates.features.contains(&feature_id.to_string()) {
                let mut tc_front = tc.front.clone();
                tc_front.validates.features.push(feature_id.to_string());
                let content = parser::render_test(&tc_front, &tc.body);
                writes.push(LinkWrite {
                    path: tc.path.clone(),
                    content,
                    kind: LinkWriteKind::Tc,
                });
                reciprocated.push(LinkReciprocation {
                    id: tc_id.to_string(),
                    field: "validates.features",
                });
            }
        }
    }

    // The feature's own write must be inserted first so callers reporting
    // the write list lead with the primary artifact.
    if feature_changed {
        let content = parser::render_feature(&front, &feature.body);
        let feature_write = LinkWrite {
            path: feature.path.clone(),
            content,
            kind: LinkWriteKind::Feature,
        };
        writes.insert(0, feature_write);
    }

    Ok(LinkPlan {
        feature_id: feature_id.to_string(),
        writes,
        reciprocated,
    })
}

/// I/O: write every file in the plan as a single atomic batch.
pub fn apply_link(plan: &LinkPlan) -> Result<(), ProductError> {
    if plan.writes.is_empty() {
        return Ok(());
    }
    let refs: Vec<(&std::path::Path, &str)> = plan
        .writes
        .iter()
        .map(|w| (w.path.as_path(), w.content.as_str()))
        .collect();
    fileops::write_batch_atomic(&refs)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Adr, AdrFrontMatter, AdrScope, AdrStatus, Feature, FeatureFrontMatter, FeatureStatus,
        TestCriterion, TestFrontMatter, TestStatus, TestType, ValidatesBlock,
    };
    use std::collections::HashMap;

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
                due_date: None,
                bundle: None,
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
        let plan =
            plan_link(&g, "FT-001", Some("ADR-001"), Some("TC-001")).unwrap();
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
        let plan =
            plan_link(&g, "FT-001", Some("ADR-001"), Some("TC-001")).unwrap();
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
}
