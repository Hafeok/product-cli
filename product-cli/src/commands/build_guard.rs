//! Oracle-integrity guard — a fix-loop worker may not edit the acceptance tests.
//!
//! The §6 verify/LSP fix loops re-dispatch a worker with the failing output and
//! invite it to "change the code so every check passes". A worker can satisfy a
//! check the wrong way: by editing the *test* (the oracle) to match its code, or
//! by scattering new test files. This guard reverts any worker write to a
//! test/oracle file that is not one of its declared artifacts — restoring the
//! committed version (tracked) or removing it (new) — so `done` stays as honest
//! as the spec, not as lenient as the worker.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A path is a protected oracle if it is a Rust test file — the executable spec
/// the worker is being verified against, never something it may rewrite.
fn is_oracle(p: &Path) -> bool {
    let s = p.to_string_lossy();
    s.ends_with("_tests.rs") || s.ends_with("_test.rs") || s.contains("/tests/")
}

/// Revert every write in `dispatched` that targets a protected oracle file and is
/// not one of the worker's `allowed` artifacts. Returns the reverted paths; a
/// non-empty result means the round tampered with the oracle and must not count
/// as a fix.
pub(super) fn enforce(root: &Path, allowed: &[PathBuf], dispatched: &[PathBuf]) -> Vec<PathBuf> {
    let mut reverted = Vec::new();
    for p in dispatched {
        if allowed.iter().any(|a| a == p) || !is_oracle(p) {
            continue;
        }
        revert(root, p);
        reverted.push(p.clone());
    }
    reverted
}

/// Restore a tampered oracle: `git checkout` the committed version when tracked,
/// otherwise remove the worker-created file.
fn revert(root: &Path, p: &Path) {
    let rel = p.strip_prefix(root).unwrap_or(p);
    let tracked = Command::new("git")
        .arg("-C").arg(root)
        .args(["ls-files", "--error-unmatch"]).arg(rel)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if tracked {
        let _ = Command::new("git").arg("-C").arg(root).args(["checkout", "--"]).arg(rel).status();
    } else {
        let _ = std::fs::remove_file(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(root: &Path, args: &[&str]) {
        let _ = Command::new("git").arg("-C").arg(root).args(args).output();
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q"]);
        git(dir.path(), &["config", "user.email", "t@t"]);
        git(dir.path(), &["config", "user.name", "t"]);
        dir
    }

    #[test]
    fn reverts_worker_edit_to_tracked_oracle() {
        let dir = repo();
        let root = dir.path();
        let oracle = root.join("foo_tests.rs");
        std::fs::write(&oracle, "ORIGINAL").expect("write");
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "spec"]);
        // worker tampers with the committed oracle to make its code pass
        std::fs::write(&oracle, "TAMPERED").expect("tamper");
        let reverted = enforce(root, &[], &[oracle.clone()]);
        assert_eq!(reverted, vec![oracle.clone()]);
        assert_eq!(std::fs::read_to_string(&oracle).expect("read"), "ORIGINAL");
    }

    #[test]
    fn removes_untracked_oracle_and_spares_allowed_artifact() {
        let dir = repo();
        let root = dir.path();
        let stray = root.join("bar_tests.rs"); // never committed
        let artifact = root.join("bar.rs"); // the worker's declared output
        std::fs::write(&stray, "fn t() {}").expect("write");
        std::fs::write(&artifact, "fn impl_() {}").expect("write");
        let allowed = vec![artifact.clone()];
        let reverted = enforce(root, &allowed, &[stray.clone(), artifact.clone()]);
        assert_eq!(reverted, vec![stray.clone()]);
        assert!(!stray.exists(), "untracked oracle write removed");
        assert!(artifact.exists(), "allowed artifact untouched");
    }
}
