//! Prompt management — init, list, get for authoring session prompts (ADR-022, ADR-048).
//!
//! The prompts directory is read from `[paths].prompts`
//! (`PathsConfig::prompts_resolved`). When the config key is unset, the
//! resolver returns `benchmarks/prompts` for backward compatibility with
//! pre-FT-057 repositories; callers pass that path through directly.

use crate::error::{ProductError, Result};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    ("author-pattern", "author-pattern-v1.md", "1"),
    ("author-review", "author-review-v1.md", "1"),
    ("implement", "implement-v2.md", "2"),
    ("gap-analysis", "gap-analysis-v1.md", "1"),
    ("drift-analysis", "drift-analysis-v1.md", "1"),
    ("conflict-check", "conflict-check-v1.md", "1"),
];

/// Resolve the configured prompts path for `root` by reading the discovered
/// `[paths].prompts` (FT-057). Returns the legacy fallback when no config
/// can be loaded or the key is unset.
pub fn resolve_prompts_path_for_root(root: &Path) -> String {
    let cfg_path = match crate::config::find_config_in_dir(root) {
        Some(p) => p,
        None => return "benchmarks/prompts".to_string(),
    };
    match crate::config::ProductConfig::load(&cfg_path) {
        Ok(c) => c.paths.prompts_resolved().to_string(),
        Err(_) => "benchmarks/prompts".to_string(),
    }
}

/// Resolve the prompts directory for `root` against the loaded `[paths].prompts`.
///
/// When the configured directory does not exist but a legacy `benchmarks/prompts`
/// directory does, fall back to the legacy directory and emit a one-shot
/// W031 warning to stderr (ADR-048). Once the config explicitly points at
/// the legacy path or the new path is in use, no warning fires.
fn resolve_prompts_dir(root: &Path, configured: &str) -> PathBuf {
    let primary = root.join(configured);
    if primary.exists() {
        return primary;
    }
    let legacy = root.join("benchmarks/prompts");
    if legacy.exists() && configured != "benchmarks/prompts" {
        emit_legacy_prompts_warning_once(configured);
        return legacy;
    }
    primary
}

fn emit_legacy_prompts_warning_once(configured: &str) {
    static FIRED: OnceLock<()> = OnceLock::new();
    if FIRED.set(()).is_ok() {
        eprintln!(
            "warning[W031]: prompts directory falling back to legacy `benchmarks/prompts/`\n  = configured: {}\n  = hint: run `product migrate consolidate --apply` to move prompts under `.product/prompts/` and clear this warning",
            configured
        );
    }
}

/// Initialize prompt files in the configured prompts directory.
pub fn init(root: &Path, prompts_path: &str) -> Result<Vec<String>> {
    let prompts_dir = root.join(prompts_path);
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

/// List available prompt files with version info, scoped to the configured prompts directory.
pub fn list(root: &Path, prompts_path: &str) -> Vec<PromptInfo> {
    let prompts_dir = resolve_prompts_dir(root, prompts_path);
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

/// Get the content of a specific prompt by name, reading from the configured prompts directory.
pub fn get(root: &Path, prompts_path: &str, name: &str) -> Result<String> {
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

    let prompts_dir = resolve_prompts_dir(root, prompts_path);
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
        "author-pattern" => include_str!("prompts/author_pattern.txt").to_string(),
        "author-review" => include_str!("prompts/author_review.txt").to_string(),
        "implement" => include_str!("prompts/implement.txt").to_string(),
        "gap-analysis" => include_str!("prompts/gap_analysis.txt").to_string(),
        "drift-analysis" => include_str!("prompts/drift_analysis.txt").to_string(),
        "conflict-check" => include_str!("prompts/conflict_check.txt").to_string(),
        _ => String::new(),
    }
}
