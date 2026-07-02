//! MCP tool registry — call_tool dispatcher (ADR-020).
//!
//! The framework-graph tools all run from `repo_root` (reading `.product/` and
//! the captured What graph), so the registry boots and dispatches without
//! building any aggregate graph up front.

use serde_json::Value;
use std::path::{Path, PathBuf};

use super::tools::{self, ToolDef};
use super::{JsonRpcRequest, JsonRpcResponse};

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

    /// Handle a tool call against the registry's repo root.
    pub fn call_tool(&self, name: &str, args: &Value) -> std::result::Result<Value, String> {
        let repo_root = self.repo_root.clone();
        self.call_tool_at(name, args, &repo_root)
    }

    /// Handle a tool call against an explicit repo root (used by the workflow
    /// transport to dispatch against the canonical repo it was launched for).
    pub fn call_tool_at(&self, name: &str, args: &Value, repo_root: &Path) -> std::result::Result<Value, String> {
        let tool = self.tools.iter().find(|t| t.name == name)
            .ok_or_else(|| format!("Tool not found: {}", name))?;
        if tool.requires_write && !self.write_enabled {
            return Err("Write tools are disabled. Set mcp.write = true in product.toml".to_string());
        }
        if name == "product_build_run" {
            return crate::build_handler::run(args, repo_root);
        }
        if name == "product_build_emit" {
            return crate::build_handler::emit(args, repo_root);
        }
        if name == "product_build_verdict" {
            return crate::build_handler::verdict(args, repo_root);
        }
        let _lock = if tool.requires_write {
            Some(product_core::fileops::RepoLock::acquire(repo_root)
                .map_err(|e| format!("{}", e))?)
        } else {
            None
        };
        dispatch_tool(name, args, repo_root)
    }

    /// Handle a JSON-RPC request in workflow mode against a session context.
    pub fn handle_jsonrpc_workflow(
        &self,
        request: &JsonRpcRequest,
        ctx: &crate::workflow::WorkflowCtx,
    ) -> crate::workflow::Outgoing {
        crate::workflow::handle(self, request, ctx)
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
        "serverInfo": { "name": product_core::author::MCP_SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
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
// Tool dispatcher — framework What/How graph families
// ---------------------------------------------------------------------------

fn dispatch_tool(name: &str, args: &Value, repo_root: &Path) -> Result<Value, String> {
    dispatch_what(name, args, repo_root)
        .or_else(|| dispatch_delivery(name, args, repo_root))
        .or_else(|| dispatch_deployable_unit(name, args, repo_root))
        .or_else(|| dispatch_framework_read(name, args, repo_root))
        .or_else(|| dispatch_framework_write(name, args, repo_root))
        .or_else(|| dispatch_framework_scaffold(name, args, repo_root))
        .unwrap_or_else(|| Err(format!("Tool handler not implemented: {}", name)))
}

/// §3.1–§3.5 — the What graph: domain, decider, projector, primitive.
fn dispatch_what(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::{decider_handlers as dc, domain_handlers as dm, primitive_handlers as pm, projector_handlers as pj};
    Some(match name {
        "product_domain_list" => dm::handle_domain_list(args, repo_root),
        "product_domain_show" => dm::handle_domain_show(args, repo_root),
        "product_domain_validate" => dm::handle_domain_validate(args, repo_root),
        "product_domain_export" => dm::handle_domain_export(args, repo_root),
        "product_domain_context" => dm::handle_domain_context(args, repo_root),
        "product_domain_new" => dm::handle_domain_new(args, repo_root),
        "product_domain_edit" => dm::handle_domain_edit(args, repo_root),
        "product_domain_rm" => dm::handle_domain_rm(args, repo_root),
        "product_decider_list" => dc::handle_decider_list(args, repo_root),
        "product_decider_show" => dc::handle_decider_show(args, repo_root),
        "product_decider_validate" => dc::handle_decider_validate(args, repo_root),
        "product_decider_simulate" => dc::handle_decider_simulate(args, repo_root),
        "product_decider_derive" => dc::handle_decider_derive(args, repo_root),
        "product_projector_list" => pj::handle_projector_list(args, repo_root),
        "product_projector_show" => pj::handle_projector_show(args, repo_root),
        "product_projector_validate" => pj::handle_projector_validate(args, repo_root),
        "product_projector_simulate" => pj::handle_projector_simulate(args, repo_root),
        "product_projector_derive" => pj::handle_projector_derive(args, repo_root),
        "product_primitive_list" => pm::handle_primitive_list(args, repo_root),
        "product_primitive_show" => pm::handle_primitive_show(args, repo_root),
        "product_primitive_validate" => pm::handle_primitive_validate(args, repo_root),
        _ => return None,
    })
}

/// §7 — delivery: slice, deliverable, release.
fn dispatch_delivery(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::delivery_handlers as d;
    Some(match name {
        "product_feature_list" => d::handle_feature_list(args, repo_root),
        "product_feature_show" => d::handle_feature_show(args, repo_root),
        "product_feature_context" => d::handle_feature_context(args, repo_root),
        "product_feature_new" => d::handle_feature_new(args, repo_root),
        "product_deliverable_list" => d::handle_deliverable_list(args, repo_root),
        "product_deliverable_show" => d::handle_deliverable_show(args, repo_root),
        "product_deliverable_done" => d::handle_deliverable_done(args, repo_root),
        "product_deliverable_new" => d::handle_deliverable_new(args, repo_root),
        "product_deliverable_accept" => d::handle_deliverable_accept(args, repo_root),
        "product_deliverable_runner" => d::handle_deliverable_runner(args, repo_root),
        "product_release_list" => d::handle_release_list(args, repo_root),
        "product_release_show" => d::handle_release_show(args, repo_root),
        "product_release_done" => d::handle_release_done(args, repo_root),
        "product_release_new" => d::handle_release_new(args, repo_root),
        "product_target_list" => d::handle_target_list(args, repo_root),
        "product_target_show" => d::handle_target_show(args, repo_root),
        "product_target_direction" => d::handle_target_direction(args, repo_root),
        "product_target_new" => d::handle_target_new(args, repo_root),
        _ => return None,
    })
}

/// §4/§4.2 — DeployableUnit: the concrete artifact a blueprint produces.
fn dispatch_deployable_unit(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::deployable_unit_handlers as d;
    Some(match name {
        "product_deployable_unit_list" => d::handle_deployable_unit_list(args, repo_root),
        "product_deployable_unit_show" => d::handle_deployable_unit_show(args, repo_root),
        "product_deployable_unit_validate" => d::handle_deployable_unit_validate(args, repo_root),
        "product_deployable_unit_new" => d::handle_deployable_unit_new(args, repo_root),
        _ => return None,
    })
}

/// §4/§5 — How families reading .product/: blueprint, cell, how, work-unit, worker.
fn dispatch_framework_read(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::framework_read_handlers as f;
    Some(match name {
        // Back-compat: the pre-v1.7.0 `product_archetype_*` names route to the same handlers.
        "product_blueprint_list" | "product_archetype_list" => f::handle_blueprint_list(args, repo_root),
        "product_blueprint_show" | "product_archetype_show" => f::handle_blueprint_show(args, repo_root),
        "product_blueprint_validate" | "product_archetype_validate" => f::handle_blueprint_validate(args, repo_root),
        "product_blueprint_check" | "product_archetype_check" => f::handle_blueprint_check(args, repo_root),
        "product_cell_show" => f::handle_cell_show(args, repo_root),
        "product_cell_validate" => f::handle_cell_validate(args, repo_root),
        "product_how_show" => f::handle_how_show(args, repo_root),
        "product_how_validate" => f::handle_how_validate(args, repo_root),
        "product_how_export" => f::handle_how_export(args, repo_root),
        "product_work_unit_show" => f::handle_work_unit_show(args, repo_root),
        "product_work_unit_validate" => f::handle_work_unit_validate(args, repo_root),
        "product_worker_list" => f::handle_worker_list(args, repo_root),
        "product_worker_resolve" => f::handle_worker_resolve(args, repo_root),
        _ => return None,
    })
}

/// §4 — How authoring: scaffold + build the Why cascade and contracts.
fn dispatch_framework_write(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::framework_write_handlers as fw;
    Some(match name {
        "product_how_init" => fw::handle_how_init(args, repo_root),
        "product_how_add" => fw::handle_how_add(args, repo_root),
        "product_how_set" => fw::handle_how_set(args, repo_root),
        "product_how_edit" => fw::handle_how_edit(args, repo_root),
        "product_how_rm" => fw::handle_how_rm(args, repo_root),
        _ => return None,
    })
}

/// §4/§5 — scaffold the delivery architecture: blueprint, cell, work-unit.
fn dispatch_framework_scaffold(name: &str, args: &Value, repo_root: &Path) -> Option<Result<Value, String>> {
    use super::framework_scaffold_handlers as fs;
    Some(match name {
        // Back-compat: pre-v1.7.0 `product_archetype_init` routes to the same handler.
        "product_blueprint_init" | "product_archetype_init" => fs::handle_blueprint_init(args, repo_root),
        "product_cell_init" => fs::handle_cell_init(args, repo_root),
        "product_cell_dispatch" => fs::handle_cell_dispatch(args, repo_root),
        "product_work_unit_init" => fs::handle_work_unit_init(args, repo_root),
        "product_work_unit_edit" => fs::handle_work_unit_edit(args, repo_root),
        _ => return None,
    })
}
