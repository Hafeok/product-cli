//! Scaffolding a `.decisions/` store.
//!
//! §5's layout is three directories, one of them derived. L0 builds none of
//! the derived one — there is no index at this milestone — but the ignore
//! line goes in now so an L2 rebuild cache can never be committed by
//! accident. Adopting the format from `docs/ledger-format-v1.md` alone
//! should not require anyone to guess at directory names.

use std::path::{Path, PathBuf};

use product_core::error::Result;
use product_core::fileops::write_file_atomic;

use crate::STORE_DIR;

/// The line that keeps the rebuildable index out of git (§5).
const IGNORE_LINE: &str = ".decisions/index/";

/// What an init would do, before it does it.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct InitPlan {
    /// Directories that do not exist yet.
    pub directories: Vec<PathBuf>,
    /// Whether the ignore line needs appending.
    pub add_ignore_line: bool,
    /// Whether a store is already here.
    pub already_initialised: bool,
}

/// Read the repo and decide what is missing.
pub fn plan_init(root: &Path) -> InitPlan {
    let store = root.join(STORE_DIR);
    let directories = [store.join("sets"), store.join("log")]
        .into_iter()
        .filter(|d| !d.is_dir())
        .collect::<Vec<_>>();
    InitPlan {
        already_initialised: store.is_dir() && directories.is_empty(),
        directories,
        add_ignore_line: !ignore_line_present(root),
    }
}

/// Commit the plan.
pub fn apply_init(root: &Path, plan: &InitPlan) -> Result<()> {
    for dir in &plan.directories {
        std::fs::create_dir_all(dir)?;
    }
    if plan.add_ignore_line {
        let path = root.join(".gitignore");
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let separator = if existing.is_empty() || existing.ends_with('\n') { "" } else { "\n" };
        write_file_atomic(&path, &format!("{existing}{separator}{IGNORE_LINE}\n"))?;
    }
    Ok(())
}

/// Render the plan's outcome for a human.
pub fn render_init(plan: &InitPlan) -> String {
    if plan.already_initialised && !plan.add_ignore_line {
        return format!("{STORE_DIR}/ is already initialised");
    }
    let mut lines: Vec<String> =
        plan.directories.iter().map(|d| format!("created {}", d.display())).collect();
    if plan.add_ignore_line {
        lines.push(format!("appended {IGNORE_LINE} to .gitignore"));
    }
    lines.join("\n")
}

fn ignore_line_present(root: &Path) -> bool {
    std::fs::read_to_string(root.join(".gitignore"))
        .map(|t| t.lines().any(|l| l.trim() == IGNORE_LINE))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_repo_needs_both_directories_plus_the_ignore_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let plan = plan_init(dir.path());
        assert_eq!(plan.directories.len(), 2);
        assert!(plan.add_ignore_line);
        assert!(!plan.already_initialised);
        apply_init(dir.path(), &plan).expect("apply");
        assert!(dir.path().join(".decisions/sets").is_dir());
        assert!(dir.path().join(".decisions/log").is_dir());
    }

    #[test]
    fn a_second_run_changes_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        apply_init(dir.path(), &plan_init(dir.path())).expect("apply");
        let plan = plan_init(dir.path());
        assert!(plan.already_initialised);
        assert!(plan.directories.is_empty());
        assert!(!plan.add_ignore_line);
        assert_eq!(render_init(&plan), ".decisions/ is already initialised");
    }

    #[test]
    fn an_existing_gitignore_is_appended_to_not_replaced() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".gitignore"), "target/\n").expect("write");
        apply_init(dir.path(), &plan_init(dir.path())).expect("apply");
        let text = std::fs::read_to_string(dir.path().join(".gitignore")).expect("read");
        assert!(text.contains("target/"), "{text}");
        assert!(text.contains(IGNORE_LINE), "{text}");
    }

    #[test]
    fn a_gitignore_without_a_trailing_newline_does_not_merge_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".gitignore"), "target/").expect("write");
        apply_init(dir.path(), &plan_init(dir.path())).expect("apply");
        let text = std::fs::read_to_string(dir.path().join(".gitignore")).expect("read");
        assert!(text.lines().any(|l| l == "target/"), "{text}");
        assert!(text.lines().any(|l| l == IGNORE_LINE), "{text}");
    }
}
