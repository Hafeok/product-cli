//! Sourcing the author identity from git configuration (OD-3, resolved).
//!
//! There is no `--as` flag anywhere in the CLI: `accepted-by` is a claimed
//! identity sourced from git config at act time, so changing identity means
//! changing git config — one source of truth. The same source names
//! `created_by` on every change-set an authoring verb writes; the difference
//! is that a model identity is legal there (models may *author*, §9.3) and
//! refused where anything is *signed* (`L006`, `L010`).

use std::path::Path;
use std::process::Command;

use crate::identity::Identity;

/// The identity git config claims for this actor, at this root.
pub fn git_identity(root: &Path) -> Result<Identity, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "user.email"])
        .output()
        .map_err(|e| format!("could not run git to read user.email: {e}"))?;
    if !out.status.success() {
        return Err(
            "git config user.email is unset — the author identity comes from git config (OD-3); \
             set it with `git config user.email <address>`"
                .to_string(),
        );
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let trimmed = raw.trim();
    trimmed
        .parse()
        .map_err(|e| format!("git config user.email does not parse as an identity: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    #[test]
    fn the_identity_is_read_from_local_git_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "--initial-branch=main"]);
        git(dir.path(), &["config", "user.email", "Fixture-Human@Example"]);
        let who = git_identity(dir.path()).expect("identity");
        // Normalised at parse, like every identity.
        assert_eq!(who.as_str(), "fixture-human@example");
    }

    #[test]
    fn an_unparsable_configured_address_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "--initial-branch=main"]);
        git(dir.path(), &["config", "user.email", "not an address"]);
        let err = git_identity(dir.path()).expect_err("bad address");
        assert!(err.contains("does not parse"), "{err}");
    }
}
