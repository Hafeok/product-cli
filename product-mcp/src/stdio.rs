//! MCP stdio transport — stdin/stdout JSON-RPC (ADR-020)

use product_core::error::{ProductError, Result};
use std::path::PathBuf;

use super::registry::ToolRegistry;
use super::workflow::WorkflowCtx;
use super::{JsonRpcRequest, JsonRpcResponse};

/// Run MCP server over stdio (stdin/stdout). When `session_id` is set the server
/// runs the phase-gated workflow against that session (one session per process).
pub fn run_stdio(repo_root: PathBuf, write_enabled: bool, session_id: Option<String>) -> Result<()> {
    use std::io::BufRead;

    let registry = ToolRegistry::new(repo_root.clone(), write_enabled);
    let ctx = session_id.map(|id| WorkflowCtx::resolve(&repo_root, &id));
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
                write_line(&stdout, &serde_json::to_string(&resp).unwrap_or_default());
                continue;
            }
        };

        match &ctx {
            Some(ctx) => {
                let out = registry.handle_jsonrpc_workflow(&request, ctx);
                if let Some(response) = out.response {
                    write_line(&stdout, &serde_json::to_string(&response).unwrap_or_default());
                }
                for note in out.notifications {
                    write_line(&stdout, &serde_json::to_string(&note).unwrap_or_default());
                }
            }
            None => {
                // Notifications return None — no response written (MCP spec)
                if let Some(response) = registry.handle_jsonrpc(&request) {
                    write_line(&stdout, &serde_json::to_string(&response).unwrap_or_default());
                }
            }
        }
    }

    Ok(())
}

fn write_line(stdout: &std::io::Stdout, json: &str) {
    use std::io::Write;
    let mut out = stdout.lock();
    let _ = writeln!(out, "{}", json);
    let _ = out.flush();
}
