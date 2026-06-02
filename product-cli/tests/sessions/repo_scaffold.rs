//! Shared scaffolding helpers for session tests that need to write
//! feature/ADR/TC files directly or set up a git repository.
//!
//! Most sessions build state through `Session::apply` (the request
//! pipeline). These helpers exist for sessions that need specific IDs,
//! specific statuses, or pre-existing git state (drift, tags, verify
//! with real runners) that the request pipeline cannot express cleanly.

#![allow(dead_code)]

use super::harness::Session;
use std::process::{Command, Stdio};

/// Run `git <args>` inside the session directory.
pub fn git(s: &Session, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(s.dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("git")
}

/// Initialise a git repo in the session root with a local identity.
pub fn init_git(s: &Session) {
    git(s, &["init", "-q"]);
    git(s, &["config", "user.email", "test@example.com"]);
    git(s, &["config", "user.name", "test"]);
}

/// Stage everything and make a commit.
pub fn git_commit_all(s: &Session, message: &str) {
    git(s, &["add", "."]);
    git(s, &["commit", "-qm", message]);
}

/// Write a minimal feature file.
pub fn write_feature(
    s: &Session,
    id: &str,
    title: &str,
    phase: u32,
    status: &str,
    adrs: &[&str],
    tests: &[&str],
) {
    let adrs_str = if adrs.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", adrs.join(", "))
    };
    let tests_str = if tests.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", tests.join(", "))
    };
    let content = format!(
        r#"---
id: {id}
title: {title}
phase: {phase}
status: {status}
adrs: {adrs_str}
tests: {tests_str}
---

Test feature body.
"#
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/features/{}-{}.md", id, slug), &content);
}

/// Write a minimal ADR file.
pub fn write_adr(s: &Session, id: &str, title: &str, status: &str) {
    let content = format!(
        r#"---
id: {id}
title: {title}
status: {status}
domains: [api]
scope: domain
---

**Context:** Test context.

**Decision:** Test decision.

**Rationale:** Test rationale.

**Rejected alternatives:** None.

**Test coverage:** None.
"#
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

/// Write a TC file. `tc_type` is the TC type (scenario, exit-criteria, …);
/// `runner`/`runner_args` are optional (pass empty strings to omit).
#[allow(clippy::too_many_arguments)]
pub fn write_tc(
    s: &Session,
    id: &str,
    title: &str,
    tc_type: &str,
    feature: &str,
    runner: &str,
    runner_args: &str,
    status: &str,
) {
    let mut fm = format!(
        r#"---
id: {id}
title: {title}
type: {tc_type}
status: {status}
validates:
  features: [{feature}]
phase: 1
"#
    );
    if !runner.is_empty() {
        fm.push_str(&format!("runner: {runner}\n"));
        if !runner_args.is_empty() {
            fm.push_str(&format!("runner-args: \"{runner_args}\"\n"));
        }
    }
    fm.push_str("---\n\nTest criterion body.\n");
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/tests/{}-{}.md", id, slug), &fm);
}

/// Write an executable bash script that exits with the given code.
/// Returns the path relative to the session root.
pub fn write_exit_script(s: &Session, name: &str, exit_code: u8) -> String {
    let path = format!("scripts/{}.sh", name);
    s.write(&path, &format!("#!/usr/bin/env bash\nexit {}\n", exit_code));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let abs = s.dir.path().join(&path);
        let mut perms = std::fs::metadata(&abs).expect("stat").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&abs, perms).expect("chmod");
    }
    path
}
