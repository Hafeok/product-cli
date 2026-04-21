//! Shared helpers used by `request_cmd.rs` — finding rendering, draft-path
//! fallback, JSON serialisation, git commit runner.

use product_lib::request::{self, ApplyResult, Finding};
use std::path::{Path, PathBuf};

/// If `file` is None, return the active draft path. Errors when no draft exists.
pub fn resolve_file_or_draft(
    file: Option<&Path>,
    root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match file {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let draft = product_lib::request::builder::draft::draft_path(root);
            if !draft.exists() {
                Err("no active draft — pass a file or run `product request new`".into())
            } else {
                Ok(draft)
            }
        }
    }
}

pub fn print_findings(findings: &[Finding], fmt: &str) {
    if fmt == "json" {
        let v = serde_json::json!({
            "findings": findings,
            "errors":   findings.iter().filter(|f| f.is_error()).count(),
            "warnings": findings.iter().filter(|f| !f.is_error()).count(),
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return;
    }
    for f in findings {
        eprintln!("{f}\n");
    }
}

pub fn print_json_result(result: &ApplyResult) {
    let v = serde_json::json!({
        "applied": result.applied,
        "created": result.created,
        "changed": result.changed,
        "findings": result.findings,
        "graph_check_clean": result.graph_check_clean,
        "started_tags": result.started_tags,
        "started_tag_warnings": result.started_tag_warnings,
    });
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
}

pub fn dedup_findings(findings: &mut Vec<Finding>) {
    let mut seen: std::collections::HashSet<(String, String, String)> =
        std::collections::HashSet::new();
    findings.retain(|f| {
        let k = (f.code.clone(), f.location.clone(), f.message.clone());
        seen.insert(k)
    });
}

pub fn print_apply_summary(result: &request::ApplyResult) {
    println!("\n  Applying:");
    for c in &result.created {
        println!("    {}  (new) -> {}", c.id, c.file);
    }
    for c in &result.changed {
        println!("    {}  ({} mutation(s)) -> {}", c.id, c.mutations, c.file);
    }
    if result.graph_check_clean {
        println!("\n  Graph check:  clean");
    } else {
        println!("\n  Graph check:  post-apply findings (inspect with `product graph check`)");
    }
    for tag in &result.started_tags {
        println!("  Tagged: {}", tag);
    }
    for warn in &result.started_tag_warnings {
        eprintln!("{}", warn);
    }
    println!(
        "  Done. {} created, {} changed.",
        result.created.len(),
        result.changed.len()
    );
}

pub fn run_git_commit(root: &Path, reason: &str) {
    let git = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .status();
    if git.map(|s| s.success()).unwrap_or(false) {
        let message = format!("product request apply\n\n{reason}");
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", &message])
            .current_dir(root)
            .status();
    }
}
