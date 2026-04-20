//! ADR creation — pure planning with matching I/O application.

use crate::error::ProductError;
use crate::{fileops, parser, types};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CreatePlan {
    pub id: String,
    pub filename: String,
    pub front: types::AdrFrontMatter,
    pub body: String,
}

impl CreatePlan {
    pub fn rendered(&self) -> String {
        parser::render_adr(&self.front, &self.body)
    }
}

pub fn plan_create(
    title: &str,
    existing_ids: &[String],
    id_prefix: &str,
) -> Result<CreatePlan, ProductError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ProductError::ConfigError(
            "ADR title cannot be empty".to_string(),
        ));
    }
    let id = parser::next_id(id_prefix, existing_ids);
    let filename = parser::id_to_filename(&id, title);
    let front = types::AdrFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        status: types::AdrStatus::Proposed,
        features: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        domains: vec![],
        scope: types::AdrScope::Domain,
        content_hash: None,
        amendments: vec![],
        source_files: vec![],
        removes: vec![],
        deprecates: vec![],
    };
    let body = "**Status:** Proposed\n\n**Context:**\n\n[Describe the context here.]\n\n**Decision:**\n\n[Describe the decision.]\n\n**Rationale:**\n\n[Explain why.]\n\n**Rejected alternatives:**\n\n- [Alternative 1]\n".to_string();
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
