//! Feature `depends-on` list edits with cycle detection (FT-062).
//!
//! Mirrors the granular-tool surface (`feature_domain`, `feature_acknowledge`)
//! and exposes idempotent add/remove semantics. Cycle detection reuses the
//! same hypothetical-graph + topological-sort approach the existing
//! `feature_link --dep` path uses.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::PathBuf;

/// Plan describing a pending `depends-on` edit. Exposed so callers can
/// inspect the diff before persisting and so MCP can render a structured
/// success response.
#[derive(Debug, Clone)]
pub struct DependsOnPlan {
    pub feature_id: String,
    pub feature_path: PathBuf,
    pub feature_content: String,
    /// Final, deduplicated list after applying adds and removes.
    pub final_depends_on: Vec<String>,
    /// IDs that were genuinely added (not already present).
    pub added: Vec<String>,
    /// IDs that were genuinely removed (were present pre-edit).
    pub removed: Vec<String>,
}

impl DependsOnPlan {
    /// True iff the plan would alter the feature's front-matter.
    pub fn is_changed(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }
}

/// Pure: plan a `depends-on` edit.
///
/// - **E002 broken-link** if any value in `add` is not an existing feature.
/// - **E003 dependency-cycle** if the proposed adds would close a cycle in
///   the feature DAG. The error carries the offending cycle path.
/// - Idempotent: adding an already-present ID and removing an absent ID are
///   no-ops; both reflect in `added` / `removed` only when the state changed.
pub fn plan_depends_on_edit(
    graph: &KnowledgeGraph,
    feature_id: &str,
    add: &[String],
    remove: &[String],
) -> Result<DependsOnPlan, ProductError> {
    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    // Validate every add target exists in the graph (E002).
    for target in add {
        if target == feature_id {
            // Self-edge is a degenerate cycle — surface it early as a
            // dependency cycle so the user message names the actual problem
            // rather than "broken link".
            return Err(ProductError::DependencyCycle {
                cycle: vec![feature_id.to_string(), feature_id.to_string()],
            });
        }
        if !graph.features.contains_key(target) {
            return Err(ProductError::NotFound(format!("feature {}", target)));
        }
    }

    // Apply add then remove on a working copy.
    let mut front = feature.front.clone();
    let pre = front.depends_on.clone();

    for target in add {
        if !front.depends_on.contains(target) {
            front.depends_on.push(target.clone());
        }
    }
    for target in remove {
        front.depends_on.retain(|d| d != target);
    }
    // Stable order: preserve insertion order while deduplicating. A stable
    // ordering keeps git diffs minimal across repeated edits.
    front.depends_on = dedup_preserve_order(&front.depends_on);

    // Cycle detection — build a hypothetical graph with the proposed feature
    // and run topological_sort, exactly like `feature_link --dep` does.
    let mut hypothetical: Vec<types::Feature> = graph
        .features
        .values()
        .filter(|f| f.front.id != feature_id)
        .cloned()
        .collect();
    hypothetical.push(types::Feature {
        front: front.clone(),
        body: feature.body.clone(),
        path: feature.path.clone(),
    });
    let test_graph = KnowledgeGraph::build(hypothetical, vec![], vec![]);
    if let Err(ProductError::DependencyCycle { cycle }) = test_graph.topological_sort() {
        return Err(ProductError::DependencyCycle { cycle });
    }

    // Compute the diff against the pre-edit list.
    let added: Vec<String> = front
        .depends_on
        .iter()
        .filter(|d| !pre.contains(d))
        .cloned()
        .collect();
    let removed: Vec<String> = pre
        .iter()
        .filter(|d| !front.depends_on.contains(d))
        .cloned()
        .collect();

    let final_depends_on = front.depends_on.clone();
    let feature_content = parser::render_feature(&front, &feature.body);

    Ok(DependsOnPlan {
        feature_id: feature_id.to_string(),
        feature_path: feature.path.clone(),
        feature_content,
        final_depends_on,
        added,
        removed,
    })
}

/// I/O: persist the depends-on edit to disk.
pub fn apply_depends_on_edit(plan: &DependsOnPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.feature_path, &plan.feature_content)?;
    Ok(())
}

/// Deduplicate a slice while preserving the first occurrence's position.
fn dedup_preserve_order(items: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());
    for s in items {
        if seen.insert(s.clone()) {
            out.push(s.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Feature, FeatureFrontMatter, FeatureStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn feat(id: &str, depends_on: Vec<String>) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("feature {}", id),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on,
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
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn graph_of(features: Vec<Feature>) -> KnowledgeGraph {
        KnowledgeGraph::build(features, vec![], vec![])
    }

    #[test]
    fn adding_self_is_rejected_as_cycle() {
        let g = graph_of(vec![feat("FT-001", vec![])]);
        let err = plan_depends_on_edit(&g, "FT-001", &["FT-001".to_string()], &[]).unwrap_err();
        assert!(matches!(err, ProductError::DependencyCycle { .. }));
    }

    #[test]
    fn adding_unknown_feature_is_rejected_with_not_found() {
        let g = graph_of(vec![feat("FT-001", vec![])]);
        let err =
            plan_depends_on_edit(&g, "FT-001", &["FT-DOES-NOT-EXIST".to_string()], &[]).unwrap_err();
        assert!(matches!(err, ProductError::NotFound(_)));
    }

    #[test]
    fn add_two_then_remove_one_diff_is_correct() {
        let g = graph_of(vec![
            feat("FT-001", vec![]),
            feat("FT-002", vec![]),
            feat("FT-003", vec![]),
        ]);
        let plan = plan_depends_on_edit(
            &g,
            "FT-001",
            &["FT-002".to_string(), "FT-003".to_string()],
            &[],
        )
        .unwrap();
        assert_eq!(plan.added, vec!["FT-002", "FT-003"]);
        assert!(plan.removed.is_empty());
        assert_eq!(plan.final_depends_on, vec!["FT-002", "FT-003"]);
    }

    #[test]
    fn idempotent_add_of_existing_value() {
        let g = graph_of(vec![
            feat("FT-001", vec!["FT-002".to_string()]),
            feat("FT-002", vec![]),
        ]);
        let plan =
            plan_depends_on_edit(&g, "FT-001", &["FT-002".to_string()], &[]).unwrap();
        assert!(plan.added.is_empty());
        assert!(plan.removed.is_empty());
        assert!(!plan.is_changed());
    }

    #[test]
    fn idempotent_remove_of_absent_value() {
        let g = graph_of(vec![feat("FT-001", vec![]), feat("FT-002", vec![])]);
        let plan =
            plan_depends_on_edit(&g, "FT-001", &[], &["FT-002".to_string()]).unwrap();
        assert!(plan.added.is_empty());
        assert!(plan.removed.is_empty());
    }

    #[test]
    fn cycle_through_existing_chain_is_rejected() {
        // FT-001 -> FT-002 -> FT-003. Adding FT-003 -> FT-001 closes a cycle.
        let g = graph_of(vec![
            feat("FT-001", vec!["FT-002".to_string()]),
            feat("FT-002", vec!["FT-003".to_string()]),
            feat("FT-003", vec![]),
        ]);
        let err =
            plan_depends_on_edit(&g, "FT-003", &["FT-001".to_string()], &[]).unwrap_err();
        assert!(matches!(err, ProductError::DependencyCycle { .. }));
    }
}
