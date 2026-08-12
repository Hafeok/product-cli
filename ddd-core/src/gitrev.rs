//! Git revision plumbing for repository-diff contract validation (M8).
//!
//! The repo-diff classifier takes both sides of a change from git rather
//! than from an editing session, which is what lets CI classify changes
//! that never touched the governed path (spec invariant 5). Everything
//! here shells the `git` binary — the same posture as the ledger's blame
//! check — and stays out of classification entirely.

use std::path::Path;
use std::process::Command;

/// One changed path between two trees, with git's one-letter status.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    /// `A` added, `M` modified, `D` deleted (renames are disabled, so a
    /// rename arrives as `D` + `A` — spec §8's stated behaviour).
    pub status: char,
    /// Repo-relative path, forward slashes.
    pub path: String,
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git {}: {}", args.join(" "), err.trim()));
    }
    Ok(out.stdout)
}

/// Resolve a revision to its full commit id.
pub fn rev_parse(root: &Path, rev: &str) -> Result<String, String> {
    let out = git(root, &["rev-parse", "--verify", &format!("{rev}^{{commit}}")])?;
    Ok(String::from_utf8_lossy(&out).trim().to_string())
}

/// The current HEAD commit id.
pub fn head(root: &Path) -> Result<String, String> {
    rev_parse(root, "HEAD")
}

/// Changed files between `base` and `head` (or the working tree when
/// `head` is `None`). Rename detection is off: a rename is remove + add.
pub fn changed_files(
    root: &Path,
    base: &str,
    head: Option<&str>,
) -> Result<Vec<ChangedFile>, String> {
    let mut args = vec!["diff", "--name-status", "--no-renames", "-z", base];
    if let Some(h) = head {
        args.push(h);
    }
    let out = git(root, &args)?;
    let text = String::from_utf8_lossy(&out);
    let mut fields = text.split('\0');
    let mut files = Vec::new();
    while let (Some(status), Some(path)) = (fields.next(), fields.next()) {
        let Some(letter) = status.chars().next() else { continue };
        files.push(ChangedFile { status: letter, path: path.to_string() });
    }
    Ok(files)
}

/// The file's bytes at a revision; `None` when the path is absent there.
pub fn bytes_at(root: &Path, rev: &str, path: &str) -> Result<Option<Vec<u8>>, String> {
    let spec = format!("{rev}:{path}");
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &spec])
        .output()
        .map_err(|e| format!("git show {spec}: {e}"))?;
    if out.status.success() {
        Ok(Some(out.stdout))
    } else {
        Ok(None)
    }
}

/// Whether the working-tree file differs from its state at `rev`
/// (an untracked file counts as dirty — its parent state is uncommitted).
pub fn dirty_against(root: &Path, rev: &str, path: &str) -> Result<bool, String> {
    let disk = std::fs::read(root.join(path)).ok();
    let at_rev = bytes_at(root, rev, path)?;
    Ok(disk != at_rev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_repo(dir: &Path) {
        let run = |args: &[&str]| {
            let ok = Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?}");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@example.com"]);
        run(&["config", "user.name", "T"]);
    }

    fn commit_all(dir: &Path, msg: &str) {
        for args in [vec!["add", "-A"], vec!["commit", "-qm", msg]] {
            let ok = Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(&args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?}");
        }
    }

    #[test]
    fn diff_and_show_round_trip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        init_repo(root);
        fs::write(root.join("a.css"), ":root { --x: red; }").expect("write");
        commit_all(root, "one");
        let base = head(root).expect("head");
        fs::write(root.join("a.css"), ":root { --x: blue; }").expect("write");
        fs::write(root.join("b.css"), ".chip { color: var(--x); }").expect("write");
        commit_all(root, "two");
        let tip = head(root).expect("head");

        let files = changed_files(root, &base, Some(&tip)).expect("diff");
        let mut names: Vec<(char, &str)> =
            files.iter().map(|f| (f.status, f.path.as_str())).collect();
        names.sort();
        assert_eq!(names, vec![('A', "b.css"), ('M', "a.css")]);

        let before = bytes_at(root, &base, "a.css").expect("show").expect("present");
        assert_eq!(String::from_utf8_lossy(&before), ":root { --x: red; }");
        assert_eq!(bytes_at(root, &base, "b.css").expect("show"), None);
        assert!(!dirty_against(root, &tip, "a.css").expect("dirty"));
        fs::write(root.join("a.css"), ":root { --x: green; }").expect("write");
        assert!(dirty_against(root, &tip, "a.css").expect("dirty"));
    }
}
