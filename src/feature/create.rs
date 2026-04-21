//! Feature creation — pure planning with matching I/O application.

use crate::error::ProductError;
use crate::{fileops, parser, types};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// In-memory description of a feature to be created.
/// Holds everything needed to render and write the file, but performs no I/O.
#[derive(Debug, Clone)]
pub struct CreatePlan {
    pub id: String,
    pub filename: String,
    pub front: types::FeatureFrontMatter,
    pub body: String,
}

impl CreatePlan {
    /// Render the plan to its final file content (YAML front-matter + body).
    pub fn rendered(&self) -> String {
        parser::render_feature(&self.front, &self.body)
    }
}

/// Pure: produce a `CreatePlan` from the user's input and the current set of
/// existing feature IDs. Does not touch the filesystem.
///
/// Returns `Err(ProductError::ConfigError)` if the title is empty after trim.
pub fn plan_create(
    title: &str,
    phase: u32,
    existing_ids: &[String],
    id_prefix: &str,
) -> Result<CreatePlan, ProductError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ProductError::ConfigError(
            "feature title cannot be empty".to_string(),
        ));
    }

    let id = parser::next_id(id_prefix, existing_ids);
    let filename = parser::id_to_filename(&id, title);
    let front = types::FeatureFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        phase,
        status: types::FeatureStatus::Planned,
        depends_on: vec![],
        adrs: vec![],
        tests: vec![],
        domains: vec![],
        domains_acknowledged: HashMap::new(),
        due_date: None,
        bundle: None,
    };
    let body = format!("## Description\n\n[Describe {} here.]\n", title);
    Ok(CreatePlan {
        id,
        filename,
        front,
        body,
    })
}

/// I/O: write the plan to `target_dir`, creating directories as needed.
/// Returns the final absolute path.
pub fn apply_create(plan: &CreatePlan, target_dir: &Path) -> Result<PathBuf, ProductError> {
    std::fs::create_dir_all(target_dir)?;
    let path = target_dir.join(&plan.filename);
    fileops::write_file_atomic(&path, &plan.rendered())?;
    Ok(path)
}
