//! Pattern scaffolding — `plan_create` + `apply_create` (FT-070, ADR-050).

use crate::config::PatternsConfig;
use crate::error::ProductError;
use crate::{fileops, parser, types};
use std::path::{Path, PathBuf};

/// In-memory description of a pattern to be created.
#[derive(Debug, Clone)]
pub struct CreatePlan {
    pub id: String,
    pub filename: String,
    pub front: types::PatternFrontMatter,
    pub body: String,
}

impl CreatePlan {
    /// Render the plan to its final file content.
    pub fn rendered(&self) -> String {
        parser::render_pattern(&self.front, &self.body)
    }
}

/// Pure: produce a `CreatePlan` from the title and the current set of
/// existing pattern IDs. The body scaffolds every required H2 heading from
/// the `[patterns].body-sections` list so a fresh pattern parses cleanly
/// against the body-section validator.
pub fn plan_create(
    title: &str,
    existing_ids: &[String],
    id_prefix: &str,
    config: &PatternsConfig,
) -> Result<CreatePlan, ProductError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ProductError::ConfigError(
            "pattern title cannot be empty".to_string(),
        ));
    }
    let id = parser::next_id(id_prefix, existing_ids);
    let filename = parser::id_to_filename(&id, title);
    let front = types::PatternFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        status: types::PatternStatus::Live,
        domains: vec![],
        adrs: vec![],
        requires: vec![],
        examples: vec![],
        deprecated_by: None,
    };
    let body = scaffold_body(&config.body_sections);
    Ok(CreatePlan {
        id,
        filename,
        front,
        body,
    })
}

/// Produce a body that includes every configured H2 section as a heading.
/// Each section gets a placeholder paragraph so a `live` pattern is never
/// reported as "empty section" by the body validator (FT-071).
pub fn scaffold_body(sections: &[String]) -> String {
    let mut s = String::new();
    for (i, heading) in sections.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        s.push_str("## ");
        s.push_str(heading);
        s.push_str("\n\n[Describe ");
        s.push_str(heading);
        s.push_str(" here.]\n");
    }
    s
}

/// I/O: write the plan to `target_dir`, creating directories as needed.
pub fn apply_create(plan: &CreatePlan, target_dir: &Path) -> Result<PathBuf, ProductError> {
    std::fs::create_dir_all(target_dir)?;
    let path = target_dir.join(&plan.filename);
    fileops::write_file_atomic(&path, &plan.rendered())?;
    Ok(path)
}
