//! Prompt management — init, list, get for authoring session prompts (ADR-022)

use crate::error::{ProductError, Result};
use std::path::Path;

/// Prompt metadata returned by list/get operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptInfo {
    pub name: String,
    pub filename: String,
    pub version: String,
    pub path: String,
}

/// Default prompt file definitions
pub(crate) const DEFAULT_PROMPTS: &[(&str, &str, &str)] = &[
    ("author-feature", "author-feature-v1.md", "1"),
    ("author-adr", "author-adr-v1.md", "1"),
    ("author-review", "author-review-v1.md", "1"),
    ("implement", "implement-v1.md", "1"),
    ("gap-analysis", "gap-analysis-v1.md", "1"),
    ("drift-analysis", "drift-analysis-v1.md", "1"),
    ("conflict-check", "conflict-check-v1.md", "1"),
];

/// Initialize prompt files in benchmarks/prompts/
pub fn init(root: &Path) -> Result<Vec<String>> {
    let prompts_dir = root.join("benchmarks/prompts");
    std::fs::create_dir_all(&prompts_dir).map_err(|e| ProductError::WriteError {
        path: prompts_dir.clone(),
        message: e.to_string(),
    })?;

    let mut created = Vec::new();
    for (name, filename, _version) in DEFAULT_PROMPTS {
        let path = prompts_dir.join(filename);
        if !path.exists() {
            let content = default_content(name);
            std::fs::write(&path, &content).map_err(|e| ProductError::WriteError {
                path: path.clone(),
                message: e.to_string(),
            })?;
            created.push(filename.to_string());
        }
    }
    Ok(created)
}

/// List available prompt files with version info
pub fn list(root: &Path) -> Vec<PromptInfo> {
    let prompts_dir = root.join("benchmarks/prompts");
    DEFAULT_PROMPTS
        .iter()
        .map(|(name, filename, version)| {
            let path = prompts_dir.join(filename);
            PromptInfo {
                name: name.to_string(),
                filename: filename.to_string(),
                version: version.to_string(),
                path: path.display().to_string(),
            }
        })
        .collect()
}

/// Get the content of a specific prompt by name
pub fn get(root: &Path, name: &str) -> Result<String> {
    let info = DEFAULT_PROMPTS
        .iter()
        .find(|(n, _, _)| *n == name)
        .ok_or_else(|| {
            ProductError::NotFound(format!(
                "Prompt '{}'. Available: {}",
                name,
                DEFAULT_PROMPTS.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ")
            ))
        })?;

    let prompts_dir = root.join("benchmarks/prompts");
    let path = prompts_dir.join(info.1);
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| ProductError::IoError(e.to_string()))
    } else {
        Ok(default_content(name))
    }
}

/// Get default content for a prompt by name
pub(crate) fn default_content(name: &str) -> String {
    match name {
        "author-feature" => include_str!("prompts/author_feature.txt").to_string(),
        "author-adr" => include_str!("prompts/author_adr.txt").to_string(),
        "author-review" => include_str!("prompts/author_review.txt").to_string(),
        "implement" => include_str!("prompts/implement.txt").to_string(),
        "gap-analysis" => include_str!("prompts/gap_analysis.txt").to_string(),
        "drift-analysis" => include_str!("prompts/drift_analysis.txt").to_string(),
        "conflict-check" => include_str!("prompts/conflict_check.txt").to_string(),
        _ => String::new(),
    }
}
