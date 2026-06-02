//! Repo-root-relative path normalisation for log entries (FT-051).
//!
//! The request log is committed to git alongside the code it describes. Absolute
//! paths in the log leak the committer's username, machine layout, and produce
//! a different log for every clone location. This module normalises every
//! path-typed field to a repo-relative, POSIX-style form before serialising.
//!
//! Callers continue to pass absolute paths; relativisation happens inside the
//! `request_log` module boundary (FT-051, ADR-039 follow-on).

use std::path::{Component, Path, PathBuf};

/// Sentinel recorded in a `migrate` entry's `result.created` block to mark the
/// one-shot "rewrite absolute `file:` fields to repo-relative form" migration
/// (FT-051).
pub const PATH_RELATIVIZE_SENTINEL: &str = "path-relativize";

/// Outcome of relativising a single path.
///
/// `is_escape` is true when the input was an absolute path that did not live
/// under `repo_root`. Callers emit these loudly (W-path-absolute on the next
/// `verify`) because writes outside the repo are a bug elsewhere.
#[derive(Debug, Clone)]
pub struct RelativeResult {
    pub value: String,
    pub is_escape: bool,
}

/// Relativise `path_str` against `repo_root`.
///
/// Contract:
/// - Empty input → returned unchanged.
/// - Already-relative input → POSIX-normalised, returned.
/// - Absolute input under `repo_root` → prefix-stripped, POSIX-joined.
/// - Absolute input outside `repo_root` → returned as-is with `is_escape=true`.
///
/// Canonicalisation is best-effort: on Windows or in containers where symlink
/// resolution matters, we fall back to plain prefix stripping if canonicalise
/// fails.
pub fn path_relativize(path_str: &str, repo_root: &Path) -> RelativeResult {
    if path_str.is_empty() {
        return RelativeResult { value: String::new(), is_escape: false };
    }
    let p = Path::new(path_str);
    if p.is_relative() && !has_drive_letter(path_str) {
        return RelativeResult { value: to_posix(p), is_escape: false };
    }
    // Absolute path: try to strip `repo_root`.
    if let Some(stripped) = strip_prefix_absolute(p, repo_root) {
        return RelativeResult { value: to_posix(&stripped), is_escape: false };
    }
    // Not under repo_root — escape path. Preserve the original string so the
    // operator can see where the errant write went.
    RelativeResult { value: path_str.to_string(), is_escape: true }
}

/// Does this look like an absolute path as stored in the log?
///
/// POSIX: leading `/`. Windows: drive letter like `C:\` or `C:/`.
/// This is a string-level check because the log carries serialised strings,
/// not `Path` objects — we can't call `Path::is_absolute()` on a line from
/// `requests.jsonl` and get a cross-platform answer.
pub fn looks_absolute(s: &str) -> bool {
    s.starts_with('/') || has_drive_letter(s)
}

fn has_drive_letter(s: &str) -> bool {
    let mut chars = s.chars();
    let c1 = chars.next();
    let c2 = chars.next();
    let c3 = chars.next();
    matches!((c1, c2, c3),
        (Some(c), Some(':'), Some('/')) | (Some(c), Some(':'), Some('\\'))
            if c.is_ascii_alphabetic())
}

fn strip_prefix_absolute(p: &Path, repo_root: &Path) -> Option<PathBuf> {
    // Try canonicalised form first — handles symlinked tempdirs on macOS.
    if let (Ok(can_p), Ok(can_root)) = (p.canonicalize(), repo_root.canonicalize()) {
        if let Ok(tail) = can_p.strip_prefix(&can_root) {
            return Some(tail.to_path_buf());
        }
    }
    // Fallback: plain prefix strip using normalised components.
    let norm_p = normalize(p);
    let norm_root = normalize(repo_root);
    norm_p.strip_prefix(&norm_root).ok().map(|t| t.to_path_buf())
}

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn to_posix(p: &Path) -> String {
    let s = p.to_string_lossy();
    if !s.contains('\\') {
        return s.into_owned();
    }
    s.replace('\\', "/")
}

/// Walk a JSON value and relativise every string value at any key named `"file"`.
///
/// Mutates `value` in place. Returns the number of escape cases encountered —
/// if non-zero the caller may surface a warning on the next verify pass.
pub fn relativise_files_in_value(value: &mut serde_json::Value, repo_root: &Path) -> usize {
    let mut escapes = 0usize;
    walk(value, repo_root, &mut escapes);
    escapes
}

fn walk(v: &mut serde_json::Value, repo_root: &Path, escapes: &mut usize) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, inner) in map.iter_mut() {
                if k == "file" {
                    if let Some(s) = inner.as_str() {
                        let r = path_relativize(s, repo_root);
                        if r.is_escape {
                            *escapes += 1;
                        }
                        *inner = serde_json::Value::String(r.value);
                        continue;
                    }
                }
                walk(inner, repo_root, escapes);
            }
        }
        serde_json::Value::Array(arr) => {
            for x in arr.iter_mut() {
                walk(x, repo_root, escapes);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn relative_pass_through() {
        let r = path_relativize("docs/features/FT-001.md", &p("/tmp/repo"));
        assert_eq!(r.value, "docs/features/FT-001.md");
        assert!(!r.is_escape);
    }

    #[test]
    fn absolute_under_repo_is_stripped() {
        let dir = tempfile::tempdir().expect("tmp");
        let sub = dir.path().join("docs/features");
        std::fs::create_dir_all(&sub).expect("mkdir");
        let target = sub.join("FT-001.md");
        std::fs::write(&target, "").expect("touch");
        let r = path_relativize(&target.display().to_string(), dir.path());
        assert_eq!(r.value, "docs/features/FT-001.md");
        assert!(!r.is_escape);
    }

    #[test]
    fn escape_path_kept_with_flag() {
        let r = path_relativize("/etc/passwd", &p("/tmp/repo"));
        assert_eq!(r.value, "/etc/passwd");
        assert!(r.is_escape);
    }

    #[test]
    fn empty_passes_through() {
        let r = path_relativize("", &p("/tmp/repo"));
        assert_eq!(r.value, "");
        assert!(!r.is_escape);
    }

    #[test]
    fn looks_absolute_posix() {
        assert!(looks_absolute("/home/alice/foo"));
        assert!(!looks_absolute("docs/features/FT-001.md"));
        assert!(!looks_absolute(""));
    }

    #[test]
    fn looks_absolute_windows_drive() {
        assert!(looks_absolute("C:/Users/alice/foo"));
        assert!(looks_absolute("D:\\work\\repo"));
        assert!(!looks_absolute("docs/features/FT-001.md"));
    }

    #[test]
    fn walk_relativises_file_keys_only() {
        let dir = tempfile::tempdir().expect("tmp");
        let sub = dir.path().join("docs/features");
        std::fs::create_dir_all(&sub).expect("mkdir");
        let t = sub.join("FT-001.md");
        std::fs::write(&t, "").expect("touch");

        let mut v = serde_json::json!({
            "file": t.display().to_string(),
            "id": "FT-001",
            "nested": { "file": t.display().to_string(), "other": "x" },
            "list": [ { "file": t.display().to_string() } ]
        });
        let escapes = relativise_files_in_value(&mut v, dir.path());
        assert_eq!(escapes, 0);
        assert_eq!(v["file"], "docs/features/FT-001.md");
        assert_eq!(v["id"], "FT-001");
        assert_eq!(v["nested"]["file"], "docs/features/FT-001.md");
        assert_eq!(v["nested"]["other"], "x");
        assert_eq!(v["list"][0]["file"], "docs/features/FT-001.md");
    }

    #[test]
    fn walk_counts_escapes() {
        let mut v = serde_json::json!({
            "file": "/etc/passwd",
            "nested": { "file": "/opt/outside" }
        });
        let escapes = relativise_files_in_value(&mut v, &p("/tmp/repo"));
        assert_eq!(escapes, 2);
        // Escape paths preserved verbatim.
        assert_eq!(v["file"], "/etc/passwd");
        assert_eq!(v["nested"]["file"], "/opt/outside");
    }

    #[test]
    fn backslashes_normalised() {
        let s = to_posix(&p("docs\\features\\FT-001.md"));
        assert_eq!(s, "docs/features/FT-001.md");
    }
}
