//! Apply phase for `product migrate consolidate` — perform moves, rewrite
//! `[paths]`, update `.gitignore`, append the migrate entry (FT-057, ADR-048).

use super::consolidate::{
    canonical_paths_block_static, ConsolidationPlan, CANONICAL_CONFIG, CANONICAL_REQUESTS,
};
use crate::error::{ProductError, Result};
use std::path::{Path, PathBuf};

pub fn apply_consolidate(
    root: &Path,
    plan: &ConsolidationPlan,
    force_uncommitted: bool,
) -> Result<()> {
    if plan.is_noop() {
        return Ok(());
    }
    if !force_uncommitted {
        check_clean_working_tree(root, plan)?;
    }
    perform_moves(root, plan)?;
    rewrite_paths_in_config(root, plan)?;
    update_gitignore(root, plan)?;
    append_consolidate_log_entry(root, plan)?;
    Ok(())
}

fn check_clean_working_tree(root: &Path, plan: &ConsolidationPlan) -> Result<()> {
    if !root.join(".git").exists() {
        return Ok(());
    }
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map_err(|e| ProductError::IoError(format!("git status failed: {}", e)))?;
    if !output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let dirty: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if dirty.is_empty() {
        return Ok(());
    }
    let touched: Vec<&str> = plan.moves.iter().map(|m| m.from.as_str()).collect();
    let conflicting: Vec<&str> = dirty
        .iter()
        .filter(|line| {
            let path = line.get(3..).unwrap_or("");
            touched
                .iter()
                .any(|p| path == *p || path.starts_with(&format!("{}/", p)))
        })
        .copied()
        .collect();
    if conflicting.is_empty() {
        return Ok(());
    }
    let mut msg = String::from(
        "uncommitted changes in paths the migration would move — commit or stash first, or rerun with --force-uncommitted\n",
    );
    for c in &conflicting {
        msg.push_str(&format!("  {}\n", c));
    }
    Err(ProductError::ConfigError(msg))
}

fn perform_moves(root: &Path, plan: &ConsolidationPlan) -> Result<()> {
    for m in &plan.moves {
        let from = root.join(&m.from);
        let to = root.join(&m.to);
        if !from.exists() {
            continue;
        }
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ProductError::WriteError {
                path: parent.to_path_buf(),
                message: format!("failed to mkdir: {}", e),
            })?;
        }
        if to.exists() && m.is_dir {
            merge_dir(&from, &to)?;
            let _ = std::fs::remove_dir_all(&from);
        } else {
            std::fs::rename(&from, &to).map_err(|e| ProductError::WriteError {
                path: to.clone(),
                message: format!("rename {} → {} failed: {}", from.display(), to.display(), e),
            })?;
        }
    }
    Ok(())
}

fn merge_dir(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from).map_err(|e| ProductError::IoError(e.to_string()))? {
        let entry = entry.map_err(|e| ProductError::IoError(e.to_string()))?;
        let src = entry.path();
        let dest = to.join(entry.file_name());
        if src.is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| ProductError::WriteError {
                path: dest.clone(),
                message: e.to_string(),
            })?;
            merge_dir(&src, &dest)?;
        } else {
            std::fs::rename(&src, &dest).map_err(|e| ProductError::WriteError {
                path: dest.clone(),
                message: e.to_string(),
            })?;
        }
    }
    Ok(())
}

fn rewrite_paths_in_config(root: &Path, plan: &ConsolidationPlan) -> Result<()> {
    let config_path: PathBuf = root.join(&plan.canonical_config_path);
    if !config_path.exists() {
        return Err(ProductError::ConfigError(format!(
            "expected canonical config at {} after move",
            config_path.display()
        )));
    }
    let original = std::fs::read_to_string(&config_path).map_err(|e| {
        ProductError::ConfigError(format!("failed to read {}: {}", config_path.display(), e))
    })?;
    let rewritten = rewrite_paths_block(&original);
    crate::fileops::write_file_atomic(&config_path, &rewritten)?;
    let _ = CANONICAL_CONFIG; // silence dead-import lint
    Ok(())
}

/// Rewrite the `[paths]` table in a TOML string to the canonical FT-057 form.
/// Other top-level content is preserved.
pub fn rewrite_paths_block(toml: &str) -> String {
    let mut out = String::new();
    let mut in_paths = false;
    let mut wrote_paths = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_paths {
                in_paths = false;
            }
            if trimmed == "[paths]" {
                in_paths = true;
                wrote_paths = true;
                out.push_str(canonical_paths_block_static());
                continue;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_paths {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !wrote_paths {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(canonical_paths_block_static());
    }
    out
}

fn update_gitignore(root: &Path, plan: &ConsolidationPlan) -> Result<()> {
    if plan.gitignore_lines.is_empty() {
        return Ok(());
    }
    let path = root.join(".gitignore");
    let mut content = std::fs::read_to_string(&path).unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    if !content.contains("# Product CLI") {
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str("# Product CLI \u{2014} generated files\n");
    }
    for line in &plan.gitignore_lines {
        content.push_str(line);
        content.push('\n');
    }
    crate::fileops::write_file_atomic(&path, &content)?;
    Ok(())
}

fn append_consolidate_log_entry(root: &Path, plan: &ConsolidationPlan) -> Result<()> {
    use crate::request_log::{
        append::append_migrate_entry, git_identity, log_path,
        MIGRATE_LOG_SENTINEL_CONSOLIDATE,
    };
    let log_p = log_path(root, Some(CANONICAL_REQUESTS));
    let applied_by =
        git_identity::resolve_applied_by(root).unwrap_or_else(|_| "local:unknown".into());
    let commit = git_identity::resolve_commit(root);
    let sources: Vec<String> = plan.moves.iter().map(|m| m.from.clone()).collect();
    let created: Vec<String> = std::iter::once(MIGRATE_LOG_SENTINEL_CONSOLIDATE.to_string())
        .chain(plan.moves.iter().map(|m| m.to.clone()))
        .collect();
    append_migrate_entry(
        &log_p,
        &applied_by,
        &commit,
        "consolidate-paths: physical migration to .product/ layout (FT-057)",
        sources,
        created,
    )
    .map_err(|e| ProductError::IoError(format!("failed to append consolidate log entry: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_paths_block_replaces_existing_table() {
        let original = "name = \"x\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\n[prefixes]\nfeature = \"FT\"\n";
        let updated = rewrite_paths_block(original);
        assert!(updated.contains(".product/features"));
        assert!(updated.contains(".product/adrs"));
        assert!(!updated.contains("docs/features"));
        assert!(updated.contains("[prefixes]"));
    }

    #[test]
    fn rewrite_paths_block_appends_when_absent() {
        let original = "name = \"x\"\n[prefixes]\nfeature = \"FT\"\n";
        let updated = rewrite_paths_block(original);
        assert!(updated.contains("[paths]"));
        assert!(updated.contains(".product/features"));
    }
}
