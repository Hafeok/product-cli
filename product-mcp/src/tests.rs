//! Unit tests for MCP registry (ADR-020)

use super::registry::ToolRegistry;
use super::{JsonRpcRequest, JsonRpcResponse};

#[test]
fn tool_registry_has_read_tools() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let tools = registry.tool_list();
    assert!(tools.iter().any(|t| t.name == "product_context"));
    assert!(tools.iter().any(|t| t.name == "product_feature_list"));
}

#[test]
fn tool_registry_write_disabled_blocks() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n[paths]\nfeatures = \"f\"\nadrs = \"a\"\ntests = \"t\"\n").expect("write");
    std::fs::create_dir_all(dir.path().join("f")).expect("mkdir");
    std::fs::create_dir_all(dir.path().join("a")).expect("mkdir");
    std::fs::create_dir_all(dir.path().join("t")).expect("mkdir");
    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let result = registry.call_tool("product_feature_new", &serde_json::json!({"title": "test"}));
    assert!(result.is_err());
    assert!(result.err().unwrap_or_default().contains("disabled"));
}

#[test]
fn jsonrpc_initialize() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: serde_json::json!({}),
    };
    let response = registry.handle_jsonrpc(&request).expect("initialize should return a response");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn jsonrpc_tools_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
    let registry = ToolRegistry::new(dir.path().to_path_buf(), true);
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/list".to_string(),
        params: serde_json::json!({}),
    };
    let response = registry.handle_jsonrpc(&request).expect("tools/list should return a response");
    let tools = response.result.as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array());
    assert!(tools.is_some());
    assert!(tools.map(|t| t.len()).unwrap_or(0) > 10);
}

#[test]
fn jsonrpc_notification_returns_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: "notifications/initialized".to_string(),
        params: serde_json::json!({}),
    };
    assert!(registry.handle_jsonrpc(&request).is_none(), "notifications must not receive a response");
}
