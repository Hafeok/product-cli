//! The one-classifier proof (spec invariant 4): identical classification
//! from the diff path vs. the edit path on the same change.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::adapter;
use crate::classify::classify_edit;
use crate::manager::HostManager;
use crate::revdiff::diff_contracts;

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?}");
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "T"]);
}

const BEFORE: &str = ":root { --ink: #111; }\n.chip { color: var(--ink); }\n";
const AFTER: &str =
    ":root { --ink: #111; --ink-muted: #666; }\n.chip { color: var(--ink); }\n.card { margin: 0; }\n";

#[test]
fn diff_path_matches_edit_path_on_the_same_change() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_repo(root);
    std::fs::write(root.join("page.css"), BEFORE).expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "before"]);
    std::fs::write(root.join("page.css"), AFTER).expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "after"]);

    // The edit path: exactly the calls the interceptor makes —
    // classify_edit, then the adapter's enrichment hook.
    let adapter = adapter::for_language("htmlcss").expect("adapter");
    let flags = crate::adapter::AdapterFlags::default();
    let mut edit_events = classify_edit(adapter, &flags, BEFORE, &[], AFTER, &[]);
    if let Some(enrich) = adapter.enrich {
        let config = ddd_core::config::DddConfig::default();
        enrich(root, &root.join("page.css"), &config, &mut edit_events);
    }

    // The diff path over the committed transition.
    let mut manager = HostManager::with_config(root.to_path_buf(), None);
    let report = diff_contracts(&mut manager, "HEAD~1", Some("HEAD"), Duration::from_secs(5))
        .expect("diff-contracts");
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(report.files.len(), 1);
    let diff_events = &report.files[0].events;

    assert_eq!(edit_events.len(), diff_events.len());
    for (edit, diff) in edit_events.iter().zip(diff_events.iter()) {
        assert_eq!(edit.change, diff.event.change);
        assert_eq!(edit.facts, diff.event.facts);
        assert_eq!(edit.surface, diff.event.surface);
        assert_eq!(edit.rule, diff.event.rule);
    }
}

#[test]
fn worktree_diff_classifies_uncommitted_changes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_repo(root);
    std::fs::write(root.join("page.css"), BEFORE).expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "before"]);
    std::fs::write(root.join("page.css"), AFTER).expect("write");

    let mut manager = HostManager::with_config(root.to_path_buf(), None);
    let report =
        diff_contracts(&mut manager, "HEAD", None, Duration::from_secs(5)).expect("diff");
    assert_eq!(report.head, "worktree");
    assert_eq!(report.files.len(), 1);
    assert!(report.files[0].events.iter().any(|e| e.event.surface));
}

#[test]
fn deleted_file_reports_removals() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_repo(root);
    std::fs::write(root.join("page.css"), BEFORE).expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "before"]);
    std::fs::remove_file(root.join("page.css")).expect("rm");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "gone"]);

    let mut manager = HostManager::with_config(root.to_path_buf(), None);
    let report = diff_contracts(&mut manager, "HEAD~1", Some("HEAD"), Duration::from_secs(5))
        .expect("diff");
    assert_eq!(report.files.len(), 1);
    assert_eq!(report.files[0].after, ddd_core::contracts::ABSENT);
    assert!(report.files[0]
        .events
        .iter()
        .all(|e| e.event.change == ddd_core::surface::ChangeKind::Removed));
}
