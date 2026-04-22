//! Authoring sessions — graph-aware specification writing (ADR-022)
//!
//! `product author feature/adr/review` starts Claude Code with a versioned
//! system prompt and Product MCP active.

pub mod prompts;
mod commit;

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

/// Agent CLI that hosts the authoring session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCli {
    Claude,
    Copilot,
}

impl AgentCli {
    /// Parse from a config/flag string. Accepts `claude` or `copilot`
    /// (case-insensitive). Returns `ProductError::ConfigError` otherwise.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "copilot" => Ok(Self::Copilot),
            other => Err(ProductError::ConfigError(format!(
                "unknown author.cli value: {}\n  = hint: use `claude` or `copilot`",
                other
            ))),
        }
    }
}

impl std::fmt::Display for AgentCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Copilot => write!(f, "copilot"),
        }
    }
}

/// Start an authoring session
pub fn start_session(
    session_type: SessionType,
    cli: AgentCli,
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

    println!("Starting {} authoring session ({})...", session_type, cli);
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

    // Persist the MCP config to a temp file. Copilot's
    // `--additional-mcp-config` accepts a `@path` form and we use it to avoid
    // any shell-escape issues with embedded JSON on the command line (seen in
    // the wild: the server silently failed to start, so the agent fell back
    // to native tools).
    let mcp_path = tmp_dir.join(format!(
        "product-author-mcp-{}.json",
        chrono::Utc::now().timestamp()
    ));
    std::fs::write(&mcp_path, &mcp_json).map_err(|e| ProductError::WriteError {
        path: mcp_path.clone(),
        message: e.to_string(),
    })?;

    let status = match cli {
        AgentCli::Claude => launch_claude(&tmp_path, &mcp_json, root),
        AgentCli::Copilot => launch_copilot(&prompt, &mcp_path, root),
    };

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Authoring session complete.");
            commit::auto_commit(&session_type, root);
        }
        Ok(s) => {
            eprintln!("Agent exited with status: {}", s);
        }
        Err(e) => {
            eprintln!("Could not start {}: {}", cli, e);
            eprintln!("Ensure '{}' is in your PATH.", cli_binary(cli));
            eprintln!();
            eprintln!("System prompt written to: {}", tmp_path.display());
            eprintln!("You can use it manually with any agent.");
        }
    }

    Ok(())
}

fn cli_binary(cli: AgentCli) -> &'static str {
    match cli {
        AgentCli::Claude => "claude",
        AgentCli::Copilot => "copilot",
    }
}

fn launch_claude(tmp_path: &Path, mcp_json: &str, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    Command::new("claude")
        .args([
            "--system-prompt-file",
            &tmp_path.display().to_string(),
            "--tools",
            "Read",
            "--allowedTools",
            "Read,mcp__product__*",
            "--mcp-config",
            mcp_json,
            "--strict-mcp-config",
        ])
        .current_dir(root)
        .status()
}

/// Launch GitHub Copilot CLI with the authoring prompt as the initial
/// interactive message. Copilot has no `--system-prompt-file`, so we feed the
/// full prompt via `-i`.
///
/// Tool access is broad on the read side (read/glob/grep/list) so the agent
/// can discover artifacts and route through the `product` MCP server, but
/// mutations still go through MCP rather than direct file writes:
/// * `--available-tools` lists every tool the model can see.
/// * `--allow-tool` pre-approves the same set, skipping permission prompts.
/// * `--disable-builtin-mcps` removes Copilot's default `github-mcp-server`.
/// * `--no-custom-instructions` prevents repo AGENTS.md from mixing with the
///   authoring prompt.
///
/// The MCP config is passed via the `@path` form (Copilot's documented
/// alternative to inline JSON) because inline JSON on the command line
/// silently failed to start the server in practice.
fn launch_copilot(prompt: &str, mcp_config_path: &Path, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    let mcp_arg = format!("@{}", mcp_config_path.display());
    // Read-side tools for discovery + the product MCP server for all
    // mutations. Direct `write`/`edit`/`shell` are intentionally omitted:
    // spec changes must flow through product MCP to get atomic writes,
    // validation, and CHECKLIST.md regeneration.
    let allowed = "read,glob,grep,list,view,product";
    Command::new("copilot")
        .args([
            "-i",
            prompt,
            "--additional-mcp-config",
            &mcp_arg,
            "--available-tools",
            allowed,
            "--allow-tool",
            allowed,
            "--disable-builtin-mcps",
            "--no-custom-instructions",
        ])
        .current_dir(root)
        .status()
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

#[cfg(test)]
mod tests;
