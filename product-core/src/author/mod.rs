//! Domain (What-capture) authoring sessions.

pub mod domain;
pub mod workflow;

use std::path::Path;

use crate::error::{ProductError, Result};

/// Recursively copy `src` into `dst`, creating `dst` as needed. Top-level
/// entries whose file name is in `skip` are not copied (used to keep a session
/// workspace from recursing into `.product/sessions` or copying `build`
/// artifacts). Existing files at the destination are overwritten.
pub fn copy_tree(src: &Path, dst: &Path, skip: &[&str]) -> Result<()> {
    copy_tree_inner(src, dst, skip, true)
}

fn copy_tree_inner(src: &Path, dst: &Path, skip: &[&str], top: bool) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| ProductError::WriteError {
        path: dst.to_path_buf(),
        message: e.to_string(),
    })?;
    let entries = std::fs::read_dir(src).map_err(|e| ProductError::IoError(format!("read {}: {e}", src.display())))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if top {
            if let Some(n) = name.to_str() {
                if skip.contains(&n) {
                    continue;
                }
            }
        }
        let from = entry.path();
        let to = dst.join(&name);
        if from.is_dir() {
            copy_tree_inner(&from, &to, skip, false)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| ProductError::WriteError {
                path: to.clone(),
                message: e.to_string(),
            })?;
        }
    }
    Ok(())
}

/// Agent CLI that hosts the authoring session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCli {
    Claude,
    Copilot,
}

impl AgentCli {
    /// Parse from a config/flag string. Accepts `claude` or `copilot`
    /// (case-insensitive). Returns `ProductError::ConfigError` otherwise.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "copilot" => Ok(Self::Copilot),
            other => Err(ProductError::ConfigError(format!(
                "unknown author.cli value: {}\n  = hint: use `claude` or `copilot`",
                other
            ))),
        }
    }
}

impl std::fmt::Display for AgentCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Copilot => write!(f, "copilot"),
        }
    }
}
