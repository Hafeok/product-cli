//! MCP scaffold — .mcp.json generation for Claude Code integration (ADR-020)

use product_core::error::Result;
use std::path::Path;

/// Generate .mcp.json for Claude Code integration
pub fn scaffold_mcp_json(repo_root: &Path) -> Result<()> {
    // Key the server on the public registry name so the scaffolded `.mcp.json`
    // matches a registry install (and the session-launch config keys).
    let mut servers = serde_json::Map::new();
    servers.insert(
        product_core::author::MCP_SERVER_NAME.to_string(),
        serde_json::json!({
            "command": "product",
            "args": ["mcp"],
            "cwd": repo_root.display().to_string()
        }),
    );
    let content = serde_json::json!({ "mcpServers": servers });
    let json = serde_json::to_string_pretty(&content).unwrap_or_default();
    let path = repo_root.join(".mcp.json");
    product_core::fileops::write_file_atomic(&path, &json)?;
    Ok(())
}
