//! Bridge an in-process MCP tool surface into Copilot client-side tools.
//!
//! The Copilot CLI enforces an enterprise MCP allowlist that fingerprints
//! every configured MCP server against the org registry — a local stdio
//! command has no fingerprintable identity and is silently filtered, and an
//! HTTP remote must byte-match a registered URL. Client-side tools
//! (registered by the SDK host over the `--server` JSON-RPC connection) are
//! not MCP servers at all, so there is nothing to fingerprint. This module
//! turns any in-process JSON-RPC MCP handler (the workflow transport) into
//! such a tool set; the handler keeps full authority over gating (write
//! flags, phase locks) because every invocation dispatches through it
//! verbatim as a `tools/call`.

use std::sync::Arc;

use async_trait::async_trait;
use github_copilot_sdk::tool::{convert_mcp_call_tool_result, try_tool_parameters, ToolHandler};
use github_copilot_sdk::types::{DeferMode, Tool, ToolInvocation, ToolResult, ToolResultExpanded};
use serde_json::json;

use crate::tools::ToolDef;
use crate::{JsonRpcRequest, JsonRpcResponse};

/// An in-process MCP endpoint: one JSON-RPC request in, one response out
/// (`None` for notifications).
pub type RpcHandler = Arc<dyn Fn(&JsonRpcRequest) -> Option<JsonRpcResponse> + Send + Sync>;

/// Bridge every def into a Copilot client-side tool dispatching through
/// `handler`. `skip_permission` is set on each tool — the wrapped MCP layer
/// is the access control (write gating, phase gating), so the CLI must not
/// stack an interactive permission prompt on top.
pub fn bridged_tools(defs: &[ToolDef], handler: &RpcHandler) -> Vec<Tool> {
    defs.iter().map(|d| bridged_tool(d, Arc::clone(handler))).collect()
}

fn bridged_tool(def: &ToolDef, handler: RpcHandler) -> Tool {
    let mut tool = Tool::new(&def.name).with_description(&def.description);
    // The schemas are hand-maintained JSON objects; a malformed one falls
    // back to schemaless rather than panicking (`with_parameters` panics).
    tool.parameters = try_tool_parameters(def.input_schema.clone()).unwrap_or_default();
    tool.skip_permission = true;
    // Pre-load everything: the facilitation prompt calls tools by name, and
    // the current phase's surface must be visible without a tool search.
    tool.defer = Some(DeferMode::Never);
    tool.with_handler(Arc::new(Dispatcher { handler }))
}

/// Routes one tool invocation through the wrapped MCP handler.
struct Dispatcher {
    handler: RpcHandler,
}

#[async_trait]
impl ToolHandler for Dispatcher {
    async fn call(&self, inv: ToolInvocation) -> Result<ToolResult, github_copilot_sdk::Error> {
        let handler = Arc::clone(&self.handler);
        // The MCP handlers do synchronous file I/O under a repo lock — keep
        // them off the async runtime's worker threads.
        tokio::task::spawn_blocking(move || {
            dispatch(&handler, &inv.tool_name, inv.arguments.clone(), &inv.tool_call_id)
        })
        .await
        .map_err(|e| {
            github_copilot_sdk::Error::from(std::io::Error::other(format!(
                "tool dispatch task failed: {e}"
            )))
        })
    }
}

/// Dispatch one call synchronously; the response (or its absence) becomes a
/// tool result the model can read. MCP-layer errors come back as
/// `result_type: "failure"` results — not SDK errors — mirroring how an MCP
/// client surfaces them, so phase-gate rejections stay visible to the agent.
pub(crate) fn dispatch(
    handler: &RpcHandler,
    name: &str,
    arguments: serde_json::Value,
    call_id: &str,
) -> ToolResult {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(call_id)),
        method: "tools/call".to_string(),
        params: json!({ "name": name, "arguments": arguments }),
    };
    match handler(&request) {
        None => failure(format!("tool '{name}' produced no response")),
        Some(JsonRpcResponse { error: Some(err), .. }) => failure(err.message),
        Some(JsonRpcResponse { result: Some(result), .. }) => {
            convert_mcp_call_tool_result(&result)
                .unwrap_or_else(|| ToolResult::Text(result.to_string()))
        }
        Some(_) => failure(format!("tool '{name}' returned an empty response")),
    }
}

fn failure(message: String) -> ToolResult {
    ToolResult::Expanded(ToolResultExpanded {
        text_result_for_llm: message.clone(),
        result_type: "failure".to_string(),
        binary_results_for_llm: None,
        session_log: None,
        error: Some(message),
        tool_telemetry: None,
    })
}

#[cfg(test)]
#[path = "bridge_tests.rs"]
mod tests;
