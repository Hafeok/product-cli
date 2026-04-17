//! MCP tool registry — call_tool dispatcher (ADR-020)

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::tools::{self, ToolDef};
use super::{JsonRpcRequest, JsonRpcResponse};
use super::read_handlers;
use super::write_handlers;
use super::field_handlers;

// ---------------------------------------------------------------------------
// Tool registry
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct ToolRegistry {
    tools: Vec<ToolDef>,
    write_enabled: bool,
    repo_root: PathBuf,
}

impl ToolRegistry {
    pub fn new(repo_root: PathBuf, write_enabled: bool) -> Self {
        let tools = tools::build_tool_list();
        Self { tools, write_enabled, repo_root }
    }

    pub fn tool_list(&self) -> &[ToolDef] {
        &self.tools
    }

    /// Handle a tool call. Returns JSON result or error.
    pub fn call_tool(&self, name: &str, args: &Value) -> std::result::Result<Value, String> {
        let tool = self.tools.iter().find(|t| t.name == name)
            .ok_or_else(|| format!("Tool not found: {}", name))?;
        if tool.requires_write && !self.write_enabled {
            return Err("Write tools are disabled. Set mcp.write = true in product.toml".to_string());
        }
        let _lock = if tool.requires_write {
            Some(crate::fileops::RepoLock::acquire(&self.repo_root)
                .map_err(|e| format!("{}", e))?)
        } else {
            None
        };
        let graph = load_graph(&self.repo_root)?;
        dispatch_tool(name, args, &graph, &self.repo_root)
    }

    /// Handle a JSON-RPC request. Returns `None` for notifications.
    pub fn handle_jsonrpc(&self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        if request.method.starts_with("notifications/") {
            return None;
        }
        Some(match request.method.as_str() {
            "initialize" => handle_initialize(request),
            "tools/list" => handle_tools_list(request, self.tool_list()),
            "tools/call" => handle_tools_call(request, self),
            _ => JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                &format!("Method not found: {}", request.method),
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC method handlers
// ---------------------------------------------------------------------------

fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(request.id.clone(), serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "product", "version": env!("CARGO_PKG_VERSION") },
    }))
}

fn handle_tools_list(request: &JsonRpcRequest, tool_list: &[ToolDef]) -> JsonRpcResponse {
    let tools: Vec<Value> = tool_list.iter()
        .map(|t| serde_json::json!({
            "name": t.name,
            "description": t.description,
            "inputSchema": t.input_schema,
        }))
        .collect();
    JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "tools": tools }))
}

fn handle_tools_call(request: &JsonRpcRequest, registry: &ToolRegistry) -> JsonRpcResponse {
    let name = request.params.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let args = request.params.get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    match registry.call_tool(name, &args) {
        Ok(result) => JsonRpcResponse::success(request.id.clone(), serde_json::json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_default() }]
        })),
        Err(e) => JsonRpcResponse::error(request.id.clone(), -32603, &e),
    }
}

// ---------------------------------------------------------------------------
// Graph loading
// ---------------------------------------------------------------------------

fn load_graph(repo_root: &Path) -> Result<KnowledgeGraph, String> {
    let config = ProductConfig::load(&repo_root.join("product.toml"))
        .map_err(|e| format!("{}", e))?;
    let features_dir = config.resolve_path(repo_root, &config.paths.features);
    let adrs_dir = config.resolve_path(repo_root, &config.paths.adrs);
    let tests_dir = config.resolve_path(repo_root, &config.paths.tests);
    let deps_dir = config.resolve_path(repo_root, &config.paths.dependencies);
    let loaded = crate::parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))
        .map_err(|e| format!("{}", e))?;
    Ok(KnowledgeGraph::build_with_deps(loaded.features, loaded.adrs, loaded.tests, loaded.dependencies))
}

// ---------------------------------------------------------------------------
// Tool dispatcher
// ---------------------------------------------------------------------------

fn dispatch_tool(
    name: &str,
    args: &Value,
    graph: &KnowledgeGraph,
    repo_root: &Path,
) -> Result<Value, String> {
    match name {
        "product_responsibility" => read_handlers::handle_responsibility(repo_root),
        "product_context" => read_handlers::handle_context(args, graph),
        "product_feature_list" => read_handlers::handle_feature_list(graph),
        "product_feature_show" => read_handlers::handle_feature_show(args, graph),
        "product_feature_deps" => read_handlers::handle_feature_deps(args, graph),
        "product_adr_list" => read_handlers::handle_adr_list(graph),
        "product_adr_show" => read_handlers::handle_adr_show(args, graph),
        "product_test_show" => read_handlers::handle_test_show(args, graph),
        "product_graph_check" => {
            let mut result = graph.check();
            // W019: feature outside product responsibility (FT-039)
            let config = crate::config::ProductConfig::load(&repo_root.join("product.toml"))
                .map_err(|e| format!("{}", e))?;
            crate::graph::responsibility::check_responsibility(
                graph,
                config.responsibility(),
                &mut result,
            );
            Ok(result.to_json())
        }
        "product_graph_central" => read_handlers::handle_graph_central(args, graph),
        "product_impact" => read_handlers::handle_impact(args, graph),
        "product_gap_check" => read_handlers::handle_gap_check(args, graph, repo_root),
        "product_schema" => read_handlers::handle_schema(args),
        "product_agent_context" => read_handlers::handle_agent_context(graph, repo_root),
        "product_prompts_list" => read_handlers::handle_prompts_list(repo_root),
        "product_prompts_get" => read_handlers::handle_prompts_get(args, repo_root),
        "product_feature_new" => write_handlers::handle_feature_new(args, graph, repo_root),
        "product_adr_new" => write_handlers::handle_adr_new(args, graph, repo_root),
        "product_test_new" => write_handlers::handle_test_new(args, graph, repo_root),
        "product_feature_link" => write_handlers::handle_feature_link(args, graph),
        "product_feature_status" | "product_adr_status" | "product_test_status" => {
            write_handlers::handle_status_update(args)
        }
        "product_body_update" => write_handlers::handle_body_update(args, graph, repo_root),
        "product_adr_amend" => write_handlers::handle_adr_amend(args, graph),
        // Field management tools (FT-038)
        "product_feature_domain" => field_handlers::handle_feature_domain(args, graph, repo_root),
        "product_feature_acknowledge" => field_handlers::handle_feature_acknowledge(args, graph),
        "product_adr_domain" => field_handlers::handle_adr_domain(args, graph, repo_root),
        "product_adr_scope" => field_handlers::handle_adr_scope(args, graph),
        "product_adr_supersede" => field_handlers::handle_adr_supersede(args, graph),
        "product_adr_source_files" => field_handlers::handle_adr_source_files(args, graph, repo_root),
        "product_test_runner" => field_handlers::handle_test_runner(args, graph, repo_root),
        // Request tools (FT-041, ADR-038)
        "product_request_validate" => super::request_handlers::handle_request_validate(args, repo_root),
        "product_request_apply" => super::request_handlers::handle_request_apply(args, repo_root),
        _ => Err(format!("Tool handler not implemented: {}", name)),
    }
}
