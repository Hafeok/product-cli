//! Graph root resolution — picks the active `.product/` directory from the
//! `--root` flag, then `PRODUCT_ROOT`, then a walk-up of the current path.
//!
//! Resolution order (first match wins):
//!   1. `--root <path>` flag (stored in [`ROOT_FLAG`])
//!   2. `PRODUCT_ROOT` env var (empty value treated as unset)
//!   3. Walk up from current working directory (caller's responsibility)
//!
//! Explicit values are tilde-expanded, canonicalized (resolves symlinks), and
//! validated: the path must exist, be a directory, and contain a `.product/`
//! subdirectory. As a friendly behaviour, `--root foo/.product` is redirected
//! to `foo` so users can point at either the root or `.product/` itself.

use crate::config::{find_config_in_dir, ProductConfig};
use crate::error::{ProductError, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Process-wide override set by the top-level `--root` CLI flag. Read by
/// [`resolve_active`] before consulting `PRODUCT_ROOT`. Set once at startup
/// from `main.rs`; subsequent calls are no-ops.
static ROOT_FLAG: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSource {
    Flag,
    Env,
    WalkUp,
}

impl RootSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::Env => "env",
            Self::WalkUp => "walk-up",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedRoot {
    pub path: PathBuf,
    pub source: RootSource,
}

/// Set the `--root` flag value. First call wins; later calls are silently
/// ignored. Called from `main.rs` immediately after clap parses.
pub fn set_root_flag(path: PathBuf) {
    let _ = ROOT_FLAG.set(path);
}

/// Inspect the current `--root` flag value (mainly for tests).
pub fn root_flag() -> Option<&'static Path> {
    ROOT_FLAG.get().map(PathBuf::as_path)
}

/// Resolve an explicit root from the static flag or `PRODUCT_ROOT` env var.
/// Returns `Ok(None)` when neither is set — callers fall back to walk-up.
pub fn resolve_active() -> Result<Option<ResolvedRoot>> {
    let env = std::env::var("PRODUCT_ROOT").ok();
    resolve(ROOT_FLAG.get().map(PathBuf::as_path), env.as_deref())
}

/// Pure resolver — flag wins, then env, then `None`. Validates the supplied
/// path. Exposed for unit testing.
pub fn resolve(flag: Option<&Path>, env_var: Option<&str>) -> Result<Option<ResolvedRoot>> {
    if let Some(p) = flag {
        return resolve_explicit(p, RootSource::Flag).map(Some);
    }
    if let Some(p) = env_var {
        if !p.is_empty() {
            return resolve_explicit(Path::new(p), RootSource::Env).map(Some);
        }
    }
    Ok(None)
}

fn resolve_explicit(supplied: &Path, source: RootSource) -> Result<ResolvedRoot> {
    let expanded = expand_tilde(supplied);
    let canonical = match std::fs::canonicalize(&expanded) {
        Ok(p) => p,
        Err(_) => return Err(root_error(supplied, source, REASON_MISSING)),
    };
    if !canonical.is_dir() {
        return Err(root_error(supplied, source, REASON_NOT_DIR));
    }
    // Friendly redirect: --root foo/.product → foo
    let candidate = if canonical.file_name() == Some(OsStr::new(".product")) {
        canonical.parent().unwrap_or(&canonical).to_path_buf()
    } else {
        canonical
    };
    if !candidate.join(".product").is_dir() {
        return Err(root_error(supplied, source, REASON_NO_PRODUCT_DIR));
    }
    Ok(ResolvedRoot { path: candidate, source })
}

pub(crate) const REASON_MISSING: &str = "directory does not exist";
pub(crate) const REASON_NOT_DIR: &str = "path is not a directory";
pub(crate) const REASON_NO_PRODUCT_DIR: &str = "no .product/ subdirectory found";

/// Expand a leading `~` or `~/` against `$HOME`. Returns the input unchanged
/// when expansion isn't applicable (no leading tilde, or `$HOME` unset).
pub fn expand_tilde(p: &Path) -> PathBuf {
    let s = match p.to_str() {
        Some(s) => s,
        None => return p.to_path_buf(),
    };
    if s == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    } else if let Some(rest) = s.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    p.to_path_buf()
}

fn root_error(supplied: &Path, source: RootSource, reason: &str) -> ProductError {
    ProductError::RootNotFound {
        supplied: supplied.to_path_buf(),
        source: source.as_str(),
        reason: reason.to_string(),
    }
}

/// Pre-clap E017 enforcement (ADR-042). Honours `PRODUCT_ROOT` for explicit
/// scoping and otherwise walks up from cwd. Silent when no config is found
/// — the eventual subcommand surfaces the real error. The `--root` flag is
/// not consulted because clap hasn't parsed yet; it gets validated later
/// via [`crate::config::ProductConfig::discover`].
pub fn early_check() -> Result<()> {
    if let Ok(p) = std::env::var("PRODUCT_ROOT") {
        if !p.is_empty() {
            if let Some(candidate) = find_config_in_dir(Path::new(&p)) {
                ProductConfig::load(&candidate)?;
            }
            return Ok(());
        }
    }
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            if let Some(candidate) = find_config_in_dir(&dir) {
                ProductConfig::load(&candidate)?;
                return Ok(());
            }
            if !dir.pop() {
                return Ok(());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_root(tmp: &Path) -> PathBuf {
        let root = tmp.join("graph");
        std::fs::create_dir_all(root.join(".product")).unwrap();
        root
    }

    #[test]
    fn flag_takes_precedence_over_env() {
        let tmp = tempfile::tempdir().unwrap();
        let flag_root = make_root(tmp.path());
        let env_root = {
            let p = tmp.path().join("env_graph");
            std::fs::create_dir_all(p.join(".product")).unwrap();
            p
        };
        let resolved = resolve(Some(&flag_root), env_root.to_str()).unwrap().unwrap();
        assert_eq!(resolved.source, RootSource::Flag);
        assert_eq!(resolved.path, std::fs::canonicalize(&flag_root).unwrap());
    }

    #[test]
    fn env_used_when_flag_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let env_root = make_root(tmp.path());
        let resolved = resolve(None, env_root.to_str()).unwrap().unwrap();
        assert_eq!(resolved.source, RootSource::Env);
    }

    #[test]
    fn empty_env_treated_as_unset() {
        assert!(resolve(None, Some("")).unwrap().is_none());
    }

    #[test]
    fn neither_set_returns_none() {
        assert!(resolve(None, None).unwrap().is_none());
    }

    #[test]
    fn missing_path_errors() {
        let err = resolve(Some(Path::new("/nonexistent/xyz/abc")), None).unwrap_err();
        match err {
            ProductError::RootNotFound { reason, source, .. } => {
                assert_eq!(reason, REASON_MISSING);
                assert_eq!(source, "flag");
            }
            other => panic!("expected RootNotFound, got {:?}", other),
        }
    }

    #[test]
    fn file_rejected_as_root() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();
        let err = resolve(Some(&file), None).unwrap_err();
        match err {
            ProductError::RootNotFound { reason, .. } => {
                // canonicalize succeeds on a regular file but is_dir is false
                assert_eq!(reason, REASON_NOT_DIR);
            }
            other => panic!("expected RootNotFound, got {:?}", other),
        }
    }

    #[test]
    fn directory_without_product_subdir_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("plain");
        std::fs::create_dir_all(&dir).unwrap();
        let err = resolve(Some(&dir), None).unwrap_err();
        match err {
            ProductError::RootNotFound { reason, .. } => {
                assert_eq!(reason, REASON_NO_PRODUCT_DIR);
            }
            other => panic!("expected RootNotFound, got {:?}", other),
        }
    }

    #[test]
    fn friendly_redirect_from_dot_product() {
        let tmp = tempfile::tempdir().unwrap();
        let root = make_root(tmp.path());
        let dot = root.join(".product");
        let resolved = resolve(Some(&dot), None).unwrap().unwrap();
        assert_eq!(resolved.path, std::fs::canonicalize(&root).unwrap());
        assert_eq!(resolved.source, RootSource::Flag);
    }

    #[test]
    fn symlink_resolved() {
        let tmp = tempfile::tempdir().unwrap();
        let real = make_root(tmp.path());
        let link = tmp.path().join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real, &link).unwrap();
        let resolved = resolve(Some(&link), None).unwrap().unwrap();
        assert_eq!(resolved.path, std::fs::canonicalize(&real).unwrap());
    }

    #[test]
    fn tilde_expansion_uses_home() {
        // Save and restore HOME so other tests aren't disturbed.
        let original = std::env::var_os("HOME");
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", tmp.path());
        let expanded = expand_tilde(Path::new("~/sub"));
        assert_eq!(expanded, tmp.path().join("sub"));
        let bare = expand_tilde(Path::new("~"));
        assert_eq!(bare, tmp.path());
        let plain = expand_tilde(Path::new("/abs"));
        assert_eq!(plain, PathBuf::from("/abs"));
        match original {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn relative_path_resolved_against_cwd() {
        let tmp = tempfile::tempdir().unwrap();
        // canonicalize handles cwd-relative paths naturally
        let real = make_root(tmp.path());
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let resolved = resolve(Some(Path::new("graph")), None).unwrap().unwrap();
        assert_eq!(resolved.path, std::fs::canonicalize(&real).unwrap());
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn root_source_string_repr() {
        assert_eq!(RootSource::Flag.as_str(), "flag");
        assert_eq!(RootSource::Env.as_str(), "env");
        assert_eq!(RootSource::WalkUp.as_str(), "walk-up");
    }
}
