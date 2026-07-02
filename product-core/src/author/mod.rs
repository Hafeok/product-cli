//! Domain (What-capture) authoring sessions.

pub mod domain;
pub mod workflow;

use crate::error::{ProductError, Result};

/// The MCP registry name this CLI is published under (`io.github.<owner>/<repo>`).
///
/// Used as the `mcpServers` config key when wiring a session server into the
/// Claude or Copilot CLI, so a locally-launched session names the server exactly
/// as the github.com MCP registry does — which is also the raw token Copilot
/// matches in `--available-tools` / `--allow-tool`.
pub const MCP_SERVER_NAME: &str = "io.github.Hafeok/product-cli";

/// The Claude Code `--allowedTools` glob selecting [`MCP_SERVER_NAME`]'s tools.
///
/// Claude derives MCP tool names as `mcp__<server>__<tool>`, replacing every
/// character outside `[A-Za-z0-9_-]` in the server name with `_`; the allow glob
/// must match that sanitized prefix, so `io.github.Hafeok/product-cli` becomes
/// `mcp__io_github_Hafeok_product-cli__*`. (Copilot, by contrast, matches the raw
/// name.) Get this wrong and the glob matches nothing — leaving the agent with no
/// MCP tools, silently.
pub fn claude_tools_glob() -> String {
    let sanitized: String = MCP_SERVER_NAME
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    format!("mcp__{sanitized}__*")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_name_is_the_registry_reverse_dns_name() {
        assert_eq!(MCP_SERVER_NAME, "io.github.Hafeok/product-cli");
    }

    #[test]
    fn claude_glob_matches_claudes_sanitized_tool_prefix() {
        // Claude replaces every char outside [A-Za-z0-9_-] in the server name
        // with `_` when it builds `mcp__<server>__<tool>`; the `-` survives.
        assert_eq!(claude_tools_glob(), "mcp__io_github_Hafeok_product-cli__*");
    }
}
