//! ST-126..ST-130 — drift diff + structural drift check (FT-045, ADR-023, ADR-040).

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use std::process::{Command, Stdio};

struct Run {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl Run {
    fn run(s: &Session, args: &[&str]) -> Self {
        let out = Command::new(&s.bin)
            .args(args)
            .current_dir(s.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("spawn product");
        Run {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        }
    }
}

fn git(s: &Session, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(s.dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("git")
}

fn init_git(s: &Session) {
    git(s, &["init", "-q"]);
    git(s, &["config", "user.email", "t@t"]);
    git(s, &["config", "user.name", "t"]);
}

fn write_adr(s: &Session, id: &str, title: &str, features: &[&str]) {
    let features_str = if features.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", features.join(", "))
    };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: accepted\nfeatures: {features_str}\ndomains: [api]\nscope: domain\n---\n\n**Context:** ctx.\n\n**Decision:** decision.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.\n",
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

fn write_feature(s: &Session, id: &str, title: &str, status: &str, adrs: &[&str]) {
    let adrs_str = if adrs.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", adrs.join(", "))
    };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nphase: 1\nstatus: {status}\nadrs: {adrs_str}\ntests: []\n---\n\nFeature body.\n",
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/features/{}-{}.md", id, slug), &content);
}

/// Initialise a git repo where FT-001 is complete with tag `product/FT-001/complete`
/// and a single implementation file in `src/foo.rs`. Returns nothing.
fn init_complete_feature(s: &Session) {
    init_git(s);
    write_feature(s, "FT-001", "Feature One", "complete", &["ADR-001"]);
    write_adr(s, "ADR-001", "Governing Decision", &["FT-001"]);
    s.write("src/foo.rs", "fn foo() { println!(\"v1\"); }\n");
    git(s, &["add", "."]);
    git(s, &["commit", "-qm", "initial"]);
    git(s, &["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"]);
}

// ---------------------------------------------------------------------------
// TC-569 — drift diff includes diff + governing ADRs
// ---------------------------------------------------------------------------
#[test]
fn tc_569_drift_diff_outputs_diff_and_governing_adrs() {
    let s = Session::new();
    init_complete_feature(&s);

    // Change the implementation file after the completion tag.
    s.write("src/foo.rs", "fn foo() { println!(\"v2\"); }\n");
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "post-complete change"]);

    let r = Run::run(&s, &["drift", "diff", "FT-001"]);
    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "expected exit 0 or 2; got {}; stderr: {}",
        r.exit_code, r.stderr
    );
    // Sections present.
    for section in [
        "## Instructions",
        "## Implementation Anchor",
        "## Changes Since Completion",
        "## Governing ADRs",
    ] {
        assert!(
            r.stdout.contains(section),
            "expected '{}' in bundle; got:\n{}",
            section, r.stdout
        );
    }
    // Diff codes listed.
    for code in ["D001", "D002", "D003", "D004"] {
        assert!(
            r.stdout.contains(code),
            "expected drift code {} in Instructions; got:\n{}",
            code, r.stdout
        );
    }
    // Diff text present.
    assert!(
        r.stdout.contains("v1") && r.stdout.contains("v2"),
        "expected diff to include both v1 and v2; got:\n{}",
        r.stdout
    );
    // Governing ADR appears.
    assert!(
        r.stdout.contains("ADR-001"),
        "expected ADR-001 in Governing ADRs section; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-570 — drift diff without a completion tag warns W020
// ---------------------------------------------------------------------------
#[test]
fn tc_570_drift_diff_no_tag_warns_w020() {
    let s = Session::new();
    init_git(&s);
    write_feature(&s, "FT-001", "Feature One", "in-progress", &["ADR-001"]);
    write_adr(&s, "ADR-001", "Governing Decision", &["FT-001"]);
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);

    let r = Run::run(&s, &["drift", "diff", "FT-001"]);
    assert_eq!(r.exit_code, 2, "expected exit 2 (warning); stderr: {}", r.stderr);
    assert!(
        r.stderr.contains("W020"),
        "expected W020 on stderr; got:\n{}",
        r.stderr
    );
    // Well-formed bundle still emitted.
    assert!(
        r.stdout.contains("## Instructions")
            && r.stdout.contains("## Implementation Anchor")
            && r.stdout.contains("## Governing ADRs"),
        "bundle must still be well-formed; got:\n{}",
        r.stdout
    );
    // Changes section is empty / marked.
    assert!(
        r.stdout.contains("## Changes Since Completion"),
        "expected Changes Since Completion section; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-571 — drift diff with no changes yields an empty Changes section
// ---------------------------------------------------------------------------
#[test]
fn tc_571_drift_diff_no_changes_empty_diff_section() {
    let s = Session::new();
    init_complete_feature(&s);
    // No changes after the completion tag.

    let r = Run::run(&s, &["drift", "diff", "FT-001"]);
    assert_eq!(r.exit_code, 0, "expected exit 0 when clean; stderr: {}", r.stderr);
    // All sections present.
    for section in [
        "## Instructions",
        "## Implementation Anchor",
        "## Changes Since Completion",
        "## Governing ADRs",
    ] {
        assert!(
            r.stdout.contains(section),
            "expected '{}' section; got:\n{}",
            section, r.stdout
        );
    }
    // Changes section is marked empty.
    assert!(
        r.stdout.contains("(no changes since completion)"),
        "expected empty-changes marker; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-572 — drift check (structural) reports changed files + exits 2
// ---------------------------------------------------------------------------
#[test]
fn tc_572_drift_check_structural_reports_file_changes() {
    let s = Session::new();
    // Fixture: two implementation files exist at tagging time.
    init_git(&s);
    write_feature(&s, "FT-001", "Feature One", "complete", &["ADR-001"]);
    write_adr(&s, "ADR-001", "Governing Decision", &["FT-001"]);
    s.write("src/foo.rs", "fn foo() { println!(\"v1\"); }\n");
    s.write("src/bar.rs", "fn bar() { println!(\"v1\"); }\n");
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "initial"]);
    git(&s, &["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"]);

    // Modify both files after the tag.
    s.write("src/foo.rs", "fn foo() { println!(\"v2\"); }\n");
    s.write("src/bar.rs", "fn bar() { println!(\"v2\"); }\n");
    git(&s, &["add", "."]);
    git(&s, &["commit", "-qm", "post-complete"]);

    let r = Run::run(&s, &["drift", "check", "FT-001"]);
    assert_eq!(r.exit_code, 2, "expected exit 2 when changes detected");
    assert!(
        r.stdout.contains("src/foo.rs"),
        "expected src/foo.rs in output; got:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("src/bar.rs"),
        "expected src/bar.rs in output; got:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("product/FT-001/complete"),
        "expected completion tag name in output; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-573 — drift check exits 0 when no changes
// ---------------------------------------------------------------------------
#[test]
fn tc_573_drift_check_no_changes_exits_0() {
    let s = Session::new();
    init_complete_feature(&s);
    // No changes after the completion tag.

    let r = Run::run(&s, &["drift", "check", "FT-001"]);
    assert_eq!(r.exit_code, 0, "expected exit 0 when no changes; stderr: {}", r.stderr);
    assert!(
        r.stdout.contains("No changes since completion"),
        "expected 'No changes since completion' marker; got:\n{}",
        r.stdout
    );
}
