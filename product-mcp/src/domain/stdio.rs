//! Domain MCP stdio transport.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use product_core::error::{ProductError, Result};

use super::registry::DomainRegistry;
use crate::{JsonRpcRequest, JsonRpcResponse};

/// Run the domain authoring MCP server over stdin/stdout, persisting the
/// active session under `session_dir`.
pub fn run_domain_stdio(session_dir: PathBuf) -> Result<()> {
    let registry = DomainRegistry::new(session_dir);
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
                write_line(&stdout, &JsonRpcResponse::error(None, -32700, &format!("Parse error: {}", e)));
                continue;
            }
        };
        if let Some(response) = registry.handle_jsonrpc(&request) {
            write_line(&stdout, &response);
        }
    }
    Ok(())
}

fn write_line(stdout: &std::io::Stdout, response: &JsonRpcResponse) {
    let json = serde_json::to_string(response).unwrap_or_default();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{}", json);
    let _ = out.flush();
}
