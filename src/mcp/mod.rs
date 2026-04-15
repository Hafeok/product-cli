//! MCP server — dual-transport protocol implementation (ADR-020)
//!
//! Implements the MCP (Model Context Protocol) tool surface for Product.
//! stdio: spawned by Claude Code, communicates over stdin/stdout.
//! HTTP: Streamable HTTP transport for remote access (phone, claude.ai).

pub mod registry;
mod read_handlers;
mod write_handlers;
mod field_handlers;
pub mod stdio;
pub mod http;
pub mod scaffold;
pub mod tools;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-export public API
pub use registry::ToolRegistry;
pub use stdio::run_stdio;
pub use http::run_http;
pub use scaffold::scaffold_mcp_json;

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
