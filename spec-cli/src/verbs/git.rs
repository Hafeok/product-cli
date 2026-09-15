//! The commit an opening pins itself to.

use std::path::Path;
use std::process::Command;

/// The repo's current HEAD, or `unknown` where there is no git to ask.
///
/// A missing revision is recorded rather than refused: the record's job is to
/// survive to the closer, and a tree with no commits yet is a legitimate
/// place to build a slice.
pub fn head_revision(root: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
