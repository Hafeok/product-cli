//! Loading `.decisions/` as it stood at a git revision.
//!
//! The store-at-revision is the merge base's machinery (L3): semantic diff
//! compares two of these, and reconciliation loads the common ancestor
//! through one. Nothing is checked out — files are read straight from the
//! object store (`git show <rev>:<path>`), parsed by the same
//! [`crate::store`] rules the disk loader applies, so a store loaded at a
//! revision and the same tree checked out on disk read identically.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::store::Store;
use crate::STORE_DIR;

/// Run git in `root`, returning stdout on success.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git {} failed: {}", args.first().unwrap_or(&""), err.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Resolve a revision to its full commit id, or say why not.
pub fn resolve(root: &Path, rev: &str) -> Result<String, String> {
    let spec = format!("{rev}^{{commit}}");
    git(root, &["rev-parse", "--verify", "--quiet", &spec])
        .map(|s| s.trim().to_string())
        .map_err(|_| format!("`{rev}` does not name a commit in this repository"))
}

/// Load the store as it stood at `rev`. The returned store's `root` is the
/// live repo root (so git-dependent checks still know where they are), but
/// every file label carries the revision so findings name what was read.
pub fn load_at(root: &Path, rev: &str) -> Result<Store, String> {
    let commit = resolve(root, rev)?;
    let mut store = Store {
        root: root.to_path_buf(),
        dir: root.join(STORE_DIR),
        ..Store::default()
    };
    for path in tree_paths(root, &commit)? {
        let Some(text) = show(root, &commit, &path)? else { continue };
        let label = format!("{rev}:{path}");
        let stem = file_stem(&path);
        if path.starts_with(&format!("{STORE_DIR}/sets/")) {
            crate::store::take_set(&mut store, &label, &stem, &text);
        } else if path.starts_with(&format!("{STORE_DIR}/log/")) {
            crate::store::take_log(&mut store, PathBuf::from(&label), &label, &stem, &text);
        }
    }
    store.sets.sort_by(|a, b| a.id.cmp(&b.id));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    Ok(store)
}

/// Every `.yml`/`.yaml` path under `.decisions/` in the revision's tree.
/// A revision with no store at all is an empty store, not an error — a
/// branch that predates `ledger init` diffs as "everything was added".
fn tree_paths(root: &Path, commit: &str) -> Result<Vec<String>, String> {
    let listing = match git(root, &["ls-tree", "-r", "--name-only", commit, "--", STORE_DIR]) {
        Ok(text) => text,
        Err(_) => return Ok(Vec::new()),
    };
    let mut paths: Vec<String> = listing
        .lines()
        .map(str::trim)
        .filter(|l| l.ends_with(".yml") || l.ends_with(".yaml"))
        .map(str::to_string)
        .collect();
    paths.sort();
    Ok(paths)
}

/// One blob's content at the revision. `None` when git cannot produce it
/// (e.g. the path is a submodule entry) — skipped, never fatal.
fn show(root: &Path, commit: &str, path: &str) -> Result<Option<String>, String> {
    match git(root, &["show", &format!("{commit}:{path}")]) {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None),
    }
}

fn file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_revision_is_a_named_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = load_at(dir.path(), "no-such-rev").expect_err("no repo");
        assert!(err.contains("does not name a commit"), "{err}");
    }
}
