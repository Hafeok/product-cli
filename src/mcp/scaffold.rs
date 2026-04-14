//! MCP scaffold — .mcp.json generation for Claude Code integration (ADR-020)

use crate::error::Result;
use std::path::Path;

/// Generate .mcp.json for Claude Code integration
pub fn scaffold_mcp_json(repo_root: &Path) -> Result<()> {
    let content = serde_json::json!({
        "mcpServers": {
            "product": {
                "command": "product",
                "args": ["mcp"],
                "cwd": repo_root.display().to_string()
            }
        }
    });
    let json = serde_json::to_string_pretty(&content).unwrap_or_default();
    let path = repo_root.join(".mcp.json");
    crate::fileops::write_file_atomic(&path, &json)?;
    Ok(())
}
