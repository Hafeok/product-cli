//! Domain (What-capture) authoring sessions.

pub mod domain;

use crate::error::{ProductError, Result};

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
