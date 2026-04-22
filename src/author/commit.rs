//! Auto-commit of docs/ changes after a successful authoring session (ADR-022).

use super::SessionType;
use std::path::Path;
use std::process::Command;

/// Auto-commit changed docs after a successful authoring session.
pub(super) fn auto_commit(session_type: &SessionType, root: &Path) {
    let changed_files = match detect_doc_changes(root) {
        Some(files) => files,
        None => return,
    };

    if changed_files.is_empty() {
        println!("No artifact changes to commit.");
        return;
    }

    let message = build_commit_message(session_type, &changed_files);
    run_git_commit(root, &message);
}

fn detect_doc_changes(root: &Path) -> Option<Vec<String>> {
    let status_output = Command::new("git")
        .args(["status", "--porcelain", "docs/"])
        .current_dir(root)
        .output();

    match status_output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Some(
                stdout
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_string())
                    .collect(),
            )
        }
        Err(_) => None,
    }
}

fn build_commit_message(session_type: &SessionType, changed_files: &[String]) -> String {
    let mut ids: Vec<String> = changed_files
        .iter()
        .filter_map(|line| {
            let path = line.get(3..)?.trim();
            let fname = std::path::Path::new(path).file_stem()?.to_str()?;
            let parts: Vec<&str> = fname.splitn(3, '-').collect();
            if parts.len() >= 2 {
                Some(format!("{}-{}", parts[0], parts[1]))
            } else {
                None
            }
        })
        .collect();
    ids.sort();
    ids.dedup();

    let id_summary = if ids.len() <= 5 {
        ids.join(", ")
    } else {
        format!("{} artifacts", ids.len())
    };
    format!(
        "author({}): {}\n\nAuto-committed by product author session.",
        session_type, id_summary
    )
}

fn run_git_commit(root: &Path, message: &str) {
    let add = Command::new("git")
        .args(["add", "docs/"])
        .current_dir(root)
        .status();
    if !matches!(add, Ok(s) if s.success()) {
        eprintln!("Failed to stage changes. Commit manually.");
        return;
    }

    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .status();
    match commit {
        Ok(s) if s.success() => {
            println!(
                "Committed: {}",
                message.lines().next().unwrap_or(message)
            );
        }
        _ => {
            eprintln!("Commit failed. Changes are staged — commit manually.");
        }
    }
}
