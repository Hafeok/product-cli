//! MCP stdio transport — stdin/stdout JSON-RPC (ADR-020)

use crate::error::{ProductError, Result};
use std::path::PathBuf;

use super::registry::ToolRegistry;
use super::{JsonRpcRequest, JsonRpcResponse};

/// Run MCP server over stdio (stdin/stdout)
pub fn run_stdio(repo_root: PathBuf, write_enabled: bool) -> Result<()> {
    use std::io::{BufRead, Write};

    let registry = ToolRegistry::new(repo_root, write_enabled);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| ProductError::IoError(format!("stdin read: {}", e)))?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, -32700, &format!("Parse error: {}", e));
                let json = serde_json::to_string(&resp).unwrap_or_default();
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", json);
                let _ = out.flush();
                continue;
            }
        };

        // Notifications return None — no response written (MCP spec)
        if let Some(response) = registry.handle_jsonrpc(&request) {
            let json = serde_json::to_string(&response).unwrap_or_default();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", json);
            let _ = out.flush();
        }
    }

    Ok(())
}
