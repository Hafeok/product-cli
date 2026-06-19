//! MCP server crate (dual transport — ADR-020).
//!
//! Implements the MCP (Model Context Protocol) tool surface for Product.
//! stdio: spawned by Claude Code, communicates over stdin/stdout.
//! HTTP: Streamable HTTP transport for remote access (phone, claude.ai).

#![deny(clippy::unwrap_used)]

pub mod registry;
pub mod domain;
mod adr_lifecycle;
mod decider_handlers;
mod projector_handlers;
mod primitive_handlers;
mod delivery_handlers;
mod dep_handlers;
mod domain_handlers;
mod field_handlers;
mod framework_read_handlers;
mod health_handlers;
mod pattern_handlers;
mod pf_mcp;
mod read_handlers;
mod request_handlers;
mod write_handlers;
pub mod stdio;
pub mod http;
pub mod scaffold;
pub mod tools;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-export public API
pub use registry::ToolRegistry;
pub use domain::run_domain_stdio;
pub use stdio::run_stdio;
pub use http::run_http;
pub use scaffold::scaffold_mcp_json;

use product_core::error::ProductError;
use std::path::PathBuf;

/// Run the HTTP MCP server, blocking on a fresh tokio runtime. Wraps
/// `run_http` so the CLI adapter does not need its own `tokio` dependency.
pub fn serve_http_blocking(
    repo_root: PathBuf,
    write_enabled: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    cors_origins: Vec<String>,
) -> Result<(), ProductError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        ProductError::IoError(format!("Failed to create tokio runtime: {}", e))
    })?;
    rt.block_on(run_http(repo_root, write_enabled, port, bind, token, cors_origins))
}

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// MCP JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}
