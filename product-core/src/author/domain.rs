//! Domain (What-capture) authoring session launch (FT-109, ADR-053).
//!
//! Starts an LLM facilitation client against the domain MCP server
//! (`product author domain <product> --serve`). The server is hosted in the
//! `product-mcp` crate; this module owns the prompt + the agent spawn, mirror
//! of [`super::start_session`] but pointed at the What graph.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::AgentCli;
use crate::error::{ProductError, Result};

/// Where the active session for a product is persisted.
pub fn session_dir(root: &Path, product: &str) -> PathBuf {
    root.join(".product").join("author-domain").join(product)
}

/// The facilitation system prompt — the §2 choreography turned into guidance
/// for the model holding the MCP server as scribe.
pub fn render_prompt(product: &str) -> String {
    include_str!("domain_prompt.md").replace("{{PRODUCT}}", product)
}

/// Launch the domain authoring session: spawn the agent CLI with the domain
/// MCP server wired in and the facilitation prompt loaded.
pub fn start_session(product: &str, cli: AgentCli, seed: Option<&Path>, root: &Path) -> Result<()> {
    let dir = session_dir(root, product);
    std::fs::create_dir_all(&dir).map_err(|e| ProductError::WriteError {
        path: dir.clone(),
        message: e.to_string(),
    })?;

    let prompt = render_prompt(product);
    let tmp = std::env::temp_dir().join(format!(
        "product-author-domain-{}-{}.md",
        product,
        chrono::Utc::now().timestamp()
    ));
    std::fs::write(&tmp, &prompt).map_err(|e| ProductError::WriteError {
        path: tmp.clone(),
        message: e.to_string(),
    })?;

    let mcp_json = mcp_config_json(product, &dir, root, seed);
    println!("Starting domain (What-capture) authoring session for '{}' ({})...", product, cli);
    println!("  Session: {}", dir.display());
    println!();

    let status = match cli {
        AgentCli::Claude => launch_claude(&tmp, &mcp_json, root),
        AgentCli::Copilot => launch_copilot(&prompt, &mcp_json, root),
    };
    report(status, cli, &tmp);
    Ok(())
}

/// Build the MCP config that points the agent at the domain server. The
/// server is this same binary re-invoked in `--serve` mode.
fn mcp_config_json(product: &str, dir: &Path, root: &Path, seed: Option<&Path>) -> String {
    let exe = std::env::current_exe().unwrap_or_else(|_| "product".into());
    let mut args = vec![
        "author".to_string(),
        "domain".to_string(),
        product.to_string(),
        "--serve".to_string(),
        "--session-dir".to_string(),
        dir.display().to_string(),
    ];
    if let Some(s) = seed {
        args.push("--seed".to_string());
        args.push(s.display().to_string());
    }
    let config = serde_json::json!({
        "mcpServers": {
            "product-author-domain": {
                "command": exe.display().to_string(),
                "args": args,
                "cwd": root.display().to_string()
            }
        }
    });
    serde_json::to_string(&config).unwrap_or_default()
}

fn launch_claude(prompt_file: &Path, mcp_json: &str, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    // NB: do NOT pass `--tools Read`. In current Claude Code, `--tools`
    // replaces the *entire* available tool set with the named built-ins and
    // drops every MCP tool — so `--tools Read` silently leaves the agent with
    // no domain tools at all. Instead, allow Read + the domain MCP server and
    // block the direct file/shell mutators so all graph writes flow through
    // MCP. `--strict-mcp-config` keeps the main `product mcp` server out.
    Command::new("claude")
        .args([
            "--system-prompt-file", &prompt_file.display().to_string(),
            "--allowedTools", "Read,mcp__product-author-domain__*",
            "--disallowedTools", "Bash,Edit,Write,NotebookEdit",
            "--mcp-config", mcp_json,
            "--strict-mcp-config",
        ])
        .current_dir(root)
        .status()
}

fn launch_copilot(prompt: &str, mcp_json: &str, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    let allowed = "read,glob,grep,list,view,product-author-domain";
    Command::new("copilot")
        .args([
            "-i", prompt,
            "--additional-mcp-config", mcp_json,
            "--available-tools", allowed,
            "--allow-tool", allowed,
            "--disable-builtin-mcps",
            "--no-custom-instructions",
        ])
        .current_dir(root)
        .status()
}

fn report(status: std::io::Result<std::process::ExitStatus>, cli: AgentCli, prompt_file: &Path) {
    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Domain authoring session complete. Run `session_finalize` produced the conformant What graph.");
        }
        Ok(s) => eprintln!("Agent exited with status: {}", s),
        Err(e) => {
            eprintln!("Could not start {}: {}", cli, e);
            eprintln!("Ensure '{}' is in your PATH.", cli);
            eprintln!("Facilitation prompt written to: {}", prompt_file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_mentions_product_and_tools() {
        let p = render_prompt("acme");
        assert!(p.contains("acme"));
        assert!(p.contains("session_start"));
        assert!(p.contains("session_finalize"));
        assert!(p.contains("open_questions"));
    }

    #[test]
    fn session_dir_is_under_product_dir() {
        let d = session_dir(Path::new("/repo"), "acme");
        assert!(d.ends_with("author-domain/acme"));
    }
}
