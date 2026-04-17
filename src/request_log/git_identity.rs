//! Git identity resolution (ADR-039 decision 8).
//!
//! `product request apply` refuses to write a log entry without a git identity.
//! The identity is read from `git config user.name` / `git config user.email`
//! at apply time and formatted as `git:{name} <{email}>`.

use std::path::Path;
use std::process::Command;

/// Resolve the applied-by string for an apply. Returns `Ok(String)` or an
/// error whose message is safe to print in `error[...]`.
pub fn resolve_applied_by(repo_root: &Path) -> Result<String, String> {
    // Env override for deterministic tests and scripted runs.
    if let Ok(v) = std::env::var("PRODUCT_LOG_APPLIED_BY") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    // If this isn't a git repository, fall back to a local user identity.
    // This keeps non-git test fixtures and non-git repositories functional —
    // ADR-039 decision 8 refuses only when git IS configured but identity
    // (user.name/user.email) is missing.
    if !repo_root.join(".git").exists() {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "local".into());
        return Ok(format!("local:{}", user));
    }
    // Git repo — identity is required.
    let name = git_config(repo_root, "user.name");
    let email = git_config(repo_root, "user.email");
    match (name, email) {
        (Some(n), Some(e)) if !n.is_empty() && !e.is_empty() => Ok(format!("git:{} <{}>", n, e)),
        _ => Err(
            "git identity not configured — set `git config user.name` and `git config user.email` before running `product request apply` (missing user.name or user.email)".into(),
        ),
    }
}

/// Read HEAD commit short-hash, or empty string on failure (commit is advisory
/// — apply is not blocked by a missing HEAD).
pub fn resolve_commit(repo_root: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn git_config(repo_root: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["config", "--get", key])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}
