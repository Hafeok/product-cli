//! Blame consistency — who actually committed an acceptance (OD-3).
//!
//! `accepted-by` is a claimed identity, so on its own it is a string anyone
//! can type. This is the second mechanical corroboration: the actor an
//! acceptance names must be the author of the commit that introduced it.
//! Git is the deposition mechanism (§9.2), so the attribution it already
//! carries is free evidence, and a mismatch is verify class `L009`.
//!
//! **The limit, stated plainly:** an acceptance that is not yet committed has
//! no introducing commit, so the check is *skipped* rather than failed —
//! otherwise no acceptance could ever be committed in the first place. The
//! check therefore lands on the next run, which in practice is CI. A working
//! tree is not where this rule bites; a pushed branch is.

use std::path::Path;
use std::process::Command;

/// What git could say about the commit that introduced a needle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribution {
    /// The author email of the oldest commit whose diff added the needle.
    Author(String),
    /// The needle is in the working tree or the index, not in history.
    NotCommitted,
    /// There is no git repository here, or no usable `git`.
    NoRepository,
}

/// Find the author of the earliest commit that introduced `needle` into
/// `file`.
///
/// Uses the pickaxe (`-S`) rather than `git blame` on purpose: an acceptance
/// is a block of lines whose exact position moves with formatting, but the
/// change-set the acceptance id first appears in does not move at all.
pub fn introducing_author(root: &Path, file: &Path, needle: &str) -> Attribution {
    if !is_repository(root) {
        return Attribution::NoRepository;
    }
    let relative = file.strip_prefix(root).unwrap_or(file);
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--reverse", "--format=%ae"])
        .arg(format!("-S{needle}"))
        .arg("--")
        .arg(relative)
        .output();
    let Ok(out) = out else {
        return Attribution::NoRepository;
    };
    if !out.status.success() {
        return Attribution::NoRepository;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|email| Attribution::Author(email.to_lowercase()))
        .unwrap_or(Attribution::NotCommitted)
}

fn is_repository(root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-dir"])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[path = "blame_tests.rs"]
#[cfg(test)]
mod tests;
