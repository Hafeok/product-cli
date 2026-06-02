//! Pattern status transition (`live ↔ deprecated`) with `deprecated-by` handling.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::PathBuf;

/// In-memory description of a pending pattern status change.
#[derive(Debug, Clone)]
pub struct StatusChangePlan {
    pub pattern_id: String,
    pub previous_status: types::PatternStatus,
    pub new_status: types::PatternStatus,
    pub deprecated_by: Option<String>,
    pub path: PathBuf,
    pub content: String,
}

/// Pure: produce a status-change plan. Validates the target ADR/PAT exists
/// when transitioning to `deprecated`. On transition to `live`, clears any
/// `deprecated-by`.
pub fn plan_status_change(
    graph: &KnowledgeGraph,
    patterns: &std::collections::HashMap<String, types::Pattern>,
    pattern_id: &str,
    new_status: types::PatternStatus,
    deprecated_by: Option<&str>,
) -> Result<StatusChangePlan, ProductError> {
    let pattern = patterns
        .get(pattern_id)
        .ok_or_else(|| ProductError::NotFound(format!("pattern {}", pattern_id)))?;

    let mut front = pattern.front.clone();
    let previous_status = front.status;

    match new_status {
        types::PatternStatus::Deprecated => {
            if let Some(target) = deprecated_by {
                if !patterns.contains_key(target) {
                    return Err(ProductError::NotFound(format!("pattern {}", target)));
                }
                front.deprecated_by = Some(target.to_string());
            }
        }
        types::PatternStatus::Live => {
            // Clear any existing deprecated-by pointer.
            front.deprecated_by = None;
        }
    }
    front.status = new_status;
    let content = parser::render_pattern(&front, &pattern.body);

    let _ = graph; // unused but accepted for symmetry with feature::plan_status_change
    Ok(StatusChangePlan {
        pattern_id: pattern_id.to_string(),
        previous_status,
        new_status,
        deprecated_by: front.deprecated_by.clone(),
        path: pattern.path.clone(),
        content,
    })
}

/// I/O: write the updated pattern file atomically.
pub fn apply_status_change(plan: &StatusChangePlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.path, &plan.content)?;
    Ok(())
}
