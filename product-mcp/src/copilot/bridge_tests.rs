//! Unit tests for the MCP → client-side tool bridge.

use std::sync::Arc;

use github_copilot_sdk::types::{DeferMode, ToolResult};
use serde_json::json;

use super::{bridged_tools, dispatch, RpcHandler};
use crate::tools::ToolDef;
use crate::JsonRpcResponse;

fn def(name: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: format!("{name} description"),
        requires_write: false,
        input_schema: json!({
            "type": "object",
            "properties": { "id": { "type": "string" } },
            "required": ["id"]
        }),
    }
}

fn text_of(result: &ToolResult) -> String {
    match result {
        ToolResult::Text(t) => t.clone(),
        ToolResult::Expanded(e) => e.text_result_for_llm.clone(),
        _ => panic!("unexpected tool result shape"),
    }
}

fn is_failure(result: &ToolResult) -> bool {
    matches!(result, ToolResult::Expanded(e) if e.result_type == "failure")
}

#[test]
fn bridged_tool_carries_the_mcp_shape() {
    let handler: RpcHandler = Arc::new(|_req| None);
    let tools = bridged_tools(&[def("product_domain_new")], &handler);
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert_eq!(tool.name, "product_domain_new");
    assert_eq!(tool.description, "product_domain_new description");
    // The JSON-Schema object becomes the wire parameter map.
    assert_eq!(tool.parameters.get("type"), Some(&json!("object")));
    assert!(tool.parameters.contains_key("properties"));
    // The MCP layer is the access control; the CLI must not double-prompt.
    assert!(tool.skip_permission);
    // Always pre-loaded — the facilitation prompt calls tools by name.
    assert!(matches!(tool.defer, Some(DeferMode::Never)));
}

#[test]
fn dispatch_sends_a_tools_call_request() {
    let handler: RpcHandler = Arc::new(|req| {
        assert_eq!(req.method, "tools/call");
        assert_eq!(req.params["name"], "product_domain_show");
        assert_eq!(req.params["arguments"]["id"], "order");
        Some(JsonRpcResponse::success(
            req.id.clone(),
            json!({ "content": [{ "type": "text", "text": "the node" }] }),
        ))
    });
    let result = dispatch(&handler, "product_domain_show", json!({"id": "order"}), "call-1");
    assert!(!is_failure(&result));
    assert_eq!(text_of(&result), "the node");
}

#[test]
fn dispatch_surfaces_mcp_errors_as_failure_results() {
    // The model must see gate rejections (phase locks, validation) as tool
    // failures it can react to — not as SDK transport errors.
    let handler: RpcHandler = Arc::new(|req| {
        Some(JsonRpcResponse::error(req.id.clone(), -32603, "locked to the How phase"))
    });
    let result = dispatch(&handler, "product_how_add", json!({}), "call-2");
    assert!(is_failure(&result));
    assert!(text_of(&result).contains("locked to the How phase"));
}

#[test]
fn dispatch_handles_a_missing_response() {
    let handler: RpcHandler = Arc::new(|_req| None);
    let result = dispatch(&handler, "product_domain_show", json!({}), "call-3");
    assert!(is_failure(&result));
    assert!(text_of(&result).contains("no response"));
}

#[test]
fn dispatch_falls_back_to_raw_json_for_non_mcp_results() {
    let handler: RpcHandler =
        Arc::new(|req| Some(JsonRpcResponse::success(req.id.clone(), json!({"ok": true}))));
    let result = dispatch(&handler, "product_domain_show", json!({}), "call-4");
    assert!(text_of(&result).contains("\"ok\":true"));
}

#[test]
fn a_malformed_schema_degrades_to_schemaless() {
    let mut d = def("product_domain_new");
    d.input_schema = json!("not an object");
    let handler: RpcHandler = Arc::new(|_req| None);
    let tools = bridged_tools(&[d], &handler);
    assert!(tools[0].parameters.is_empty());
}
