//! Test-criterion status transitions.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StatusChangePlan {
    pub test_id: String,
    pub test_path: PathBuf,
    pub test_content: String,
    pub new_status: types::TestStatus,
}

pub fn plan_status_change(
    graph: &KnowledgeGraph,
    test_id: &str,
    new_status: types::TestStatus,
) -> Result<StatusChangePlan, ProductError> {
    let t = graph
        .tests
        .get(test_id)
        .ok_or_else(|| ProductError::NotFound(format!("test {}", test_id)))?;
    let mut front = t.front.clone();
    front.status = new_status;
    let content = parser::render_test(&front, &t.body);
    Ok(StatusChangePlan {
        test_id: test_id.to_string(),
        test_path: t.path.clone(),
        test_content: content,
        new_status,
    })
}

pub fn apply_status_change(plan: &StatusChangePlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.test_path, &plan.test_content)?;
    Ok(())
}
