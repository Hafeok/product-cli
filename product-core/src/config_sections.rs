//! Auxiliary `product.toml` section types — MCP settings, product identity.

use serde::{Deserialize, Serialize};

/// Product identity section — `[product]` in product.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSection {
    /// Product name (overrides top-level `name` if present).
    #[serde(default)]
    pub name: Option<String>,
    /// Single-statement responsibility — what the product is and is not.
    #[serde(default)]
    pub responsibility: Option<String>,
}

/// `[mcp]` section — MCP server settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Allow MCP write tools (default false).
    #[serde(default)]
    pub write: bool,
    /// Bearer token for HTTP transport.
    #[serde(default)]
    pub token: Option<String>,
    /// Default HTTP port.
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    /// Allowed CORS origins for HTTP transport.
    #[serde(rename = "cors-origins", default)]
    pub cors_origins: Vec<String>,
}

fn default_mcp_port() -> u16 {
    7777
}
