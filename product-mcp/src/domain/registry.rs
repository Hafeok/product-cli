//! Domain MCP tool-call dispatch over a session store.

use std::path::{Path, PathBuf};

use product_core::pf::session::DomainSession;
use serde_json::Value;

use crate::tools::ToolDef;
use crate::{JsonRpcRequest, JsonRpcResponse};

use super::tools::tool_defs;
use super::{handlers, session_handlers};

/// Hosts one What-capture session, persisted under `session_dir`.
pub struct DomainRegistry {
    session_dir: PathBuf,
    tools: Vec<ToolDef>,
}

impl DomainRegistry {
    pub fn new(session_dir: PathBuf) -> Self {
        Self { session_dir, tools: tool_defs() }
    }

    pub fn tool_list(&self) -> &[ToolDef] {
        &self.tools
    }

    fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn load(&self) -> Result<DomainSession, String> {
        DomainSession::load(&self.session_dir).map_err(|e| format!("{}", e))
    }

    /// Dispatch a single tool call to its handler.
    pub fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "session_start" => session_handlers::start(&self.session_dir, args, Self::now()),
            "session_state" => Ok(self.load()?.state_json()),
            "session_finalize" => self.finalize(),
            "validate" => Ok(self.load()?.validate_json()),
            "open_questions" => Ok(session_handlers::questions(&self.load()?, args)),
            "query" => session_handlers::run_query(&self.load()?, args),
            _ => self.dispatch_mutating(name, args),
        }
    }

    fn finalize(&self) -> Result<Value, String> {
        let mut session = self.load()?;
        let out = session_handlers::finalize(&mut session, &self.session_dir, Self::now())?;
        session.save(&self.session_dir).map_err(|e| format!("{}", e))?;
        Ok(out)
    }

    fn dispatch_mutating(&self, name: &str, args: &Value) -> Result<Value, String> {
        let mut session = self.load()?;
        let result = match name {
            "add_bounded_context" => handlers::add_bounded_context(&mut session, args),
            "add_entity" => handlers::add_entity(&mut session, args),
            "add_value_object" => handlers::add_value_object(&mut session, args),
            "add_relation" => handlers::add_relation(&mut session, args),
            "add_invariant" => handlers::add_invariant(&mut session, args),
            "add_context_mapping" => handlers::add_context_mapping(&mut session, args),
            "add_command" => handlers::add_command(&mut session, args),
            "add_event" => handlers::add_event(&mut session, args),
            "add_read_model" => handlers::add_read_model(&mut session, args),
            "add_wireframe_step" => handlers::add_wireframe_step(&mut session, args),
            "add_flow" => handlers::add_flow(&mut session, args),
            other => return Err(format!("Tool not found: {}", other)),
        }?;
        session.save(&self.session_dir).map_err(|e| format!("{}", e))?;
        Ok(result)
    }

    /// Handle a JSON-RPC request. Returns `None` for notifications.
    pub fn handle_jsonrpc(&self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        if request.method.starts_with("notifications/") {
            return None;
        }
        Some(match request.method.as_str() {
            "initialize" => initialize(request),
            "tools/list" => tools_list(request, self.tool_list()),
            "tools/call" => self.tools_call(request),
            _ => JsonRpcResponse::error(request.id.clone(), -32601,
                &format!("Method not found: {}", request.method)),
        })
    }

    fn tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let args = request.params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
        match self.call_tool(name, &args) {
            Ok(result) => JsonRpcResponse::success(request.id.clone(), serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_default() }]
            })),
            Err(e) => JsonRpcResponse::error(request.id.clone(), -32603, &e),
        }
    }
}

fn initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(request.id.clone(), serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": product_core::author::MCP_SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
    }))
}

fn tools_list(request: &JsonRpcRequest, tool_list: &[ToolDef]) -> JsonRpcResponse {
    let tools: Vec<Value> = tool_list.iter().map(|t| serde_json::json!({
        "name": t.name,
        "description": t.description,
        "inputSchema": t.input_schema,
    })).collect();
    JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "tools": tools }))
}

/// Convenience for callers that hold a `&Path`.
pub fn registry_for(session_dir: &Path) -> DomainRegistry {
    DomainRegistry::new(session_dir.to_path_buf())
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
