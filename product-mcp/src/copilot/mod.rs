//! Copilot SDK session host — product tools as in-process client tools.
//!
//! Drives the Copilot CLI in `--server` mode over JSON-RPC (via
//! `github-copilot-sdk`) instead of spawning the interactive `copilot` TUI
//! with an `--additional-mcp-config`. The workflow tool surface is
//! registered as client-side tools on the session; every invocation
//! dispatches in-process through [`crate::workflow::handle`] — the same
//! phase-gated transport the MCP servers use. Copilot's enterprise MCP
//! allowlist (which fingerprints MCP servers against the org registry,
//! silently filtering local stdio commands — formerly worked around by
//! serving the tools at a fixed loopback HTTP URL registered as an
//! org-registry remote) never applies: there is no MCP server to
//! fingerprint.

mod bridge;
mod host;
mod repl;

use std::path::Path;
use std::sync::Arc;

use product_core::error::Result;

use crate::registry::ToolRegistry;
use crate::workflow::{self, WorkflowCtx};
use bridge::RpcHandler;

pub use host::resolve_cli;

/// Host a phase-gated What→How→Build session on Copilot. Every workflow tool
/// (all families plus the session controls) is bridged as a client-side
/// tool; each call routes through [`workflow::handle`], so phase gating,
/// write gating, and build recording behave exactly as they do over MCP.
/// Blocks until the user ends the session.
pub fn run_workflow_session(session_id: &str, prompt: &str, canonical_root: &Path) -> Result<()> {
    let registry = Arc::new(ToolRegistry::new(canonical_root.to_path_buf(), true));
    let defs = full_tool_list(&registry);
    let handler = workflow_rpc_handler(Arc::clone(&registry), session_id, canonical_root);
    let tools = bridge::bridged_tools(&defs, &handler);
    repl::run_blocking(repl::SessionSpec {
        cwd: canonical_root.to_path_buf(),
        prompt: prompt.to_string(),
        tools,
    })
}

/// Every tool the workflow transport can dispatch: the registry surface plus
/// the session controls. Registered once at session creation — the per-call
/// gate in [`workflow::handle`] still enforces phase locks on every dispatch.
fn full_tool_list(registry: &ToolRegistry) -> Vec<crate::tools::ToolDef> {
    let mut tools = registry.tool_list().to_vec();
    tools.extend(workflow::control_tools());
    tools
}

/// The workflow transport as an in-process RPC endpoint for `session_id`.
/// Phase-advance notifications are dropped: the SDK host registers the full
/// tool surface up front, so there is no `tools/list` cache to invalidate.
fn workflow_rpc_handler(
    registry: Arc<ToolRegistry>,
    session_id: &str,
    canonical_root: &Path,
) -> RpcHandler {
    let session_id = session_id.to_string();
    let canonical = canonical_root.to_path_buf();
    Arc::new(move |req| {
        let ctx = WorkflowCtx::resolve(&canonical, &session_id);
        workflow::handle(&registry, req, &ctx).response
    })
}

#[cfg(test)]
#[path = "copilot_tests.rs"]
mod tests;
