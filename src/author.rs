//! Authoring sessions — graph-aware specification writing (ADR-022)
//!
//! `product author feature/adr/review` starts Claude Code with a versioned
//! system prompt and Product MCP active.

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use std::path::Path;
use std::process::Command;

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
    let prompt = if prompt_path.exists() {
        std::fs::read_to_string(&prompt_path).unwrap_or_default()
    } else {
        default_prompt(&session_type)
    };

    // Write prompt to temp file for agent
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("product-author-{}-{}.md", session_type, chrono::Utc::now().timestamp()));
    std::fs::write(&tmp_path, &prompt).map_err(|e| ProductError::WriteError {
        path: tmp_path.clone(),
        message: e.to_string(),
    })?;

    println!("Starting {} authoring session...", session_type);
    println!("  System prompt: {}", if prompt_path.exists() { prompt_path.display().to_string() } else { "(default)".to_string() });
    println!("  Repo: {}", root.display());
    println!();

    // Invoke Claude Code with the system prompt
    let status = Command::new("claude")
        .args(["--system-prompt-file", &tmp_path.display().to_string()])
        .current_dir(root)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Authoring session complete.");
            println!("Next steps:");
            println!("  product graph check   — verify structural health");
            println!("  git add . && git commit");
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

        // Structural checks (local, instant)
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
                    adr_path, name
                ));
            }
        }

        // Check for status field
        if !content.contains("status:") {
            findings.push(format!(
                "warning: {} missing status field in front-matter",
                adr_path
            ));
        }

        // Check for linked features
        if content.contains("features: []") || !content.contains("features:") {
            findings.push(format!(
                "warning: {} has no linked features",
                adr_path
            ));
        }
    }

    if findings.is_empty() {
        findings.push(format!(
            "Reviewed {} staged ADR(s) — no structural issues found.",
            staged_adrs.len()
        ));
    }

    Ok(findings)
}

fn default_prompt(session_type: &SessionType) -> String {
    match session_type {
        SessionType::Feature => r#"# Product Authoring Session: Feature

Before writing any content:
1. Call product_feature_list to understand what features exist
2. Call product_graph_central to identify the top-5 foundational ADRs
3. Call product_context on the most related existing feature (if any)
4. Ask the user clarifying questions based on what you found

Only after completing these steps should you scaffold any files.
After scaffolding, run product_graph_check to verify structural health.
"#.to_string(),
        SessionType::Adr => r#"# Product Authoring Session: ADR

Before writing any content:
1. Call product_graph_central — read the top-5 ADRs by centrality
2. Call product_adr_list to see what decisions already exist
3. Call product_impact on the area you're about to decide — understand blast radius
4. Check for potential contradictions with existing linked ADRs

Every ADR must include: Context, Decision, Rationale, Rejected alternatives, Test coverage.
Do not end the session without all five sections present.
"#.to_string(),
        SessionType::Review => r#"# Product Authoring Session: Review

Your goal is to improve specification coverage without adding new features.
Start by:
1. Call product_graph_check — fix structural issues first
2. Walk features by lowest test coverage — propose test criteria
3. Find orphaned ADRs — propose feature links
4. Find features with no exit-criteria TC — propose them

Do not create new features or ADRs unless fixing a specific identified gap.
"#.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prompts_not_empty() {
        assert!(!default_prompt(&SessionType::Feature).is_empty());
        assert!(!default_prompt(&SessionType::Adr).is_empty());
        assert!(!default_prompt(&SessionType::Review).is_empty());
    }
}
