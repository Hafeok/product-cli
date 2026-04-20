//! Feature status transitions with the ADR-010 orphan-test cascade on abandonment.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::PathBuf;

/// An updated test file produced by the abandonment cascade.
#[derive(Debug, Clone)]
pub struct OrphanedTestUpdate {
    pub test_id: String,
    pub path: PathBuf,
    pub content: String,
}

/// In-memory description of a pending status change and its cascading effects.
#[derive(Debug, Clone)]
pub struct StatusChangePlan {
    pub feature_id: String,
    pub feature_path: PathBuf,
    pub feature_content: String,
    pub new_status: types::FeatureStatus,
    /// Tests whose `validates.features` list needs `feature_id` removed
    /// (only populated when `new_status == Abandoned`).
    pub orphaned_tests: Vec<OrphanedTestUpdate>,
}

/// Pure: produce a `StatusChangePlan` from a parsed new status and the current
/// graph. Returns `NotFound` if the feature isn't in the graph.
pub fn plan_status_change(
    graph: &KnowledgeGraph,
    feature_id: &str,
    new_status: types::FeatureStatus,
) -> Result<StatusChangePlan, ProductError> {
    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    let mut front = feature.front.clone();
    front.status = new_status;
    let feature_content = parser::render_feature(&front, &feature.body);

    let orphaned_tests = if new_status == types::FeatureStatus::Abandoned {
        compute_orphaned_tests(graph, feature)
    } else {
        Vec::new()
    };

    Ok(StatusChangePlan {
        feature_id: feature_id.to_string(),
        feature_path: feature.path.clone(),
        feature_content,
        new_status,
        orphaned_tests,
    })
}

/// Pure helper: for each test linked to the feature, build an updated
/// rendered content with the feature removed from `validates.features`.
fn compute_orphaned_tests(
    graph: &KnowledgeGraph,
    feature: &types::Feature,
) -> Vec<OrphanedTestUpdate> {
    let mut updates = Vec::new();
    for test_id in &feature.front.tests {
        if let Some(tc) = graph.tests.get(test_id.as_str()) {
            let mut test_front = tc.front.clone();
            test_front.validates.features.retain(|fid| fid != &feature.front.id);
            let content = parser::render_test(&test_front, &tc.body);
            updates.push(OrphanedTestUpdate {
                test_id: test_id.clone(),
                path: tc.path.clone(),
                content,
            });
        }
    }
    updates
}

/// I/O: write the planned status change and all orphaned-test updates
/// atomically as a single batch.
pub fn apply_status_change(plan: &StatusChangePlan) -> Result<(), ProductError> {
    let mut writes: Vec<(&std::path::Path, &str)> =
        Vec::with_capacity(1 + plan.orphaned_tests.len());
    writes.push((&plan.feature_path, plan.feature_content.as_str()));
    for upd in &plan.orphaned_tests {
        writes.push((&upd.path, upd.content.as_str()));
    }
    fileops::write_batch_atomic(&writes)?;
    Ok(())
}
