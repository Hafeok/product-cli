//! `spec-mcp` — the MCP surface over product-mcp's stdio plumbing.

use std::path::PathBuf;
use std::sync::Arc;

use product_core::error::Result;
use product_mcp::registry::{CustomDispatch, ToolRegistry};

/// The registry this server runs, which the tests drive directly.
pub fn build_registry(root: PathBuf) -> ToolRegistry {
    let dispatch: CustomDispatch =
        Arc::new(crate::dispatch::dispatch);
    ToolRegistry::with_tools(root, true, "spec-mcp", crate::tools::build(), dispatch)
}

/// Serve MCP over stdio until EOF.
pub fn serve_stdio(root: PathBuf) -> Result<()> {
    product_mcp::stdio::run_stdio_registry(build_registry(root), None)
}
