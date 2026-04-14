//! Authoring sessions — graph-aware specification writing (ADR-022)
//!
//! `product author feature/adr/review` starts Claude Code with a versioned
//! system prompt and Product MCP active.

pub mod prompts;

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use std::path::Path;
use std::process::Command;

// Re-export prompt types/functions at the author:: level for backward compat
pub use prompts::{PromptInfo, get as prompts_get, init as prompts_init, list as prompts_list};

/// Session types for authoring
pub enum SessionType {
    Feature,
    Adr,
    Review,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Feature => write!(f, "feature"),
            Self::Adr => write!(f, "adr"),
            Self::Review => write!(f, "review"),
        }
    }
}

/// Start an authoring session
pub fn start_session(
    session_type: SessionType,
    _config: &ProductConfig,
    root: &Path,
) -> Result<()> {
    let prompt_name = match session_type {
        SessionType::Feature => "author-feature-v1.md",
        SessionType::Adr => "author-adr-v1.md",
        SessionType::Review => "author-review-v1.md",
    };

    let prompt_path = root.join("benchmarks/prompts").join(prompt_name);
    let base_prompt = if prompt_path.exists() {
        std::fs::read_to_string(&prompt_path).unwrap_or_default()
    } else {
        prompts::default_content(&session_type.to_string().replace(' ', "-"))
    };
    let prompt = format!("{}\n\n{}", base_prompt, schema_prompt());

    // Write prompt to temp file for agent
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!(
        "product-author-{}-{}.md",
        session_type,
        chrono::Utc::now().timestamp()
    ));
    std::fs::write(&tmp_path, &prompt).map_err(|e| ProductError::WriteError {
        path: tmp_path.clone(),
        message: e.to_string(),
    })?;

    println!("Starting {} authoring session...", session_type);
    println!(
        "  System prompt: {}",
        if prompt_path.exists() {
            prompt_path.display().to_string()
        } else {
            "(default)".to_string()
        }
    );
    println!("  Repo: {}", root.display());
    println!();

    // Build inline MCP config using the current executable
    let exe = std::env::current_exe().unwrap_or_else(|_| "product".into());
    let mcp_config = serde_json::json!({
        "mcpServers": {
            "product": {
                "command": exe.display().to_string(),
                "args": ["mcp", "--write"],
                "cwd": root.display().to_string()
            }
        }
    });
    let mcp_json = serde_json::to_string(&mcp_config).unwrap_or_default();

    let status = Command::new("claude")
        .args([
            "--system-prompt-file",
            &tmp_path.display().to_string(),
            "--tools",
            "Read",
            "--allowedTools",
            "Read,mcp__product__*",
            "--mcp-config",
            &mcp_json,
            "--strict-mcp-config",
        ])
        .current_dir(root)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Authoring session complete.");
            auto_commit(&session_type, root);
        }
        Ok(s) => {
            eprintln!("Agent exited with status: {}", s);
        }
        Err(e) => {
            eprintln!("Could not start Claude Code: {}", e);
            eprintln!("Ensure 'claude' is in your PATH.");
            eprintln!();
            eprintln!("System prompt written to: {}", tmp_path.display());
            eprintln!("You can use it manually with any agent.");
        }
    }

    Ok(())
}

/// Review staged ADR files (pre-commit hook)
pub fn review_staged(root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(root)
        .output()
        .map_err(|e| ProductError::IoError(format!("git: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let staged_adrs: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("adrs/") && l.ends_with(".md"))
        .collect();

    if staged_adrs.is_empty() {
        return Ok(vec!["No staged ADR files found.".to_string()]);
    }

    let mut findings = Vec::new();
    for adr_path in &staged_adrs {
        let full_path = root.join(adr_path);
        if !full_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&full_path).unwrap_or_default();
        review_adr_content(adr_path, &content, &mut findings);
    }

    if findings.is_empty() {
        findings.push(format!(
            "Reviewed {} staged ADR(s) — no structural issues found.",
            staged_adrs.len()
        ));
    }

    Ok(findings)
}

/// Review a single ADR file (not necessarily staged — works on any path)
pub fn review_adr_file(path: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    if !path.exists() {
        findings.push(format!("warning: {} does not exist", path.display()));
        return findings;
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let display_path = path.display().to_string();
    review_adr_content(&display_path, &content, &mut findings);
    findings
}

fn review_adr_content(display_path: &str, content: &str, findings: &mut Vec<String>) {
    let required_sections = [
        ("Context", "**Context:**"),
        ("Decision", "**Decision:**"),
        ("Rationale", "**Rationale:**"),
        ("Rejected alternatives", "**Rejected alternatives:**"),
        ("Test coverage", "**Test coverage:**"),
    ];

    for (name, marker) in &required_sections {
        if !content.contains(marker) && !content.to_lowercase().contains(&name.to_lowercase()) {
            findings.push(format!(
                "warning: {} missing required section: {}",
                display_path, name
            ));
        }
    }

    if !content.contains("status:") {
        findings.push(format!(
            "warning: {} missing status field in front-matter",
            display_path
        ));
    }

    if content.contains("features: []") || !content.contains("features:") {
        findings.push(format!(
            "warning[W001]: {} has no linked features",
            display_path
        ));
    }
}

fn schema_prompt() -> String {
    "# Artifact Schemas\n\n\
     All artifacts use YAML front-matter between `---` delimiters, followed by a markdown body.\n\n\
     ## Feature (FT-XXX)\n\n\
     ```yaml\n---\nid: FT-001\ntitle: Feature Title\nphase: 1\nstatus: planned\n\
     depends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n```\n\n\
     ## ADR (ADR-XXX)\n\n\
     ```yaml\n---\nid: ADR-001\ntitle: Decision Title\nstatus: proposed\nfeatures: []\n\
     supersedes: []\nsuperseded-by: []\ndomains: []\nscope: feature-specific\n---\n```\n\n\
     ## Test Criterion (TC-XXX)\n\n\
     ```yaml\n---\nid: TC-001\ntitle: test_name\ntype: scenario\nstatus: unimplemented\n\
     validates:\n  features: []\n  adrs: []\nphase: 1\n---\n```\n"
        .to_string()
}

/// Auto-commit changed docs after a successful authoring session
fn auto_commit(session_type: &SessionType, root: &Path) {
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

#[cfg(test)]
mod tests;
