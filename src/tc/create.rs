//! Test-criterion creation — plan + apply.

use crate::error::ProductError;
use crate::{fileops, parser, types};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CreatePlan {
    pub id: String,
    pub filename: String,
    pub front: types::TestFrontMatter,
    pub body: String,
}

impl CreatePlan {
    pub fn rendered(&self) -> String {
        parser::render_test(&self.front, &self.body)
    }
}

pub fn plan_create(
    title: &str,
    test_type: types::TestType,
    existing_ids: &[String],
    id_prefix: &str,
) -> Result<CreatePlan, ProductError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ProductError::ConfigError(
            "test title cannot be empty".to_string(),
        ));
    }
    let id = parser::next_id(id_prefix, existing_ids);
    let filename = parser::id_to_filename(&id, title);
    let front = types::TestFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        test_type,
        status: types::TestStatus::Unimplemented,
        validates: types::ValidatesBlock {
            features: vec![],
            adrs: vec![],
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
    };
    let body = "## Description\n\n[Describe the test criterion here.]\n".to_string();
    Ok(CreatePlan {
        id,
        filename,
        front,
        body,
    })
}

pub fn apply_create(plan: &CreatePlan, target_dir: &Path) -> Result<PathBuf, ProductError> {
    std::fs::create_dir_all(target_dir)?;
    let path = target_dir.join(&plan.filename);
    fileops::write_file_atomic(&path, &plan.rendered())?;
    Ok(path)
}
