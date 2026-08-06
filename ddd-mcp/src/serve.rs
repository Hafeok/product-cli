//! `ddd serve` — the MCP surface over product-mcp's stdio plumbing.
//!
//! Builds a registry carrying only the `ddd_*` tools (a governed C#/Bicep
//! repo's agent has no use for the `product_*` framework surface) with a
//! dispatcher bound to one [`ServeState`] — the hosts plus session
//! declarations live for the process, which defines "same session" for
//! the interceptor (PRD §8).

use std::path::PathBuf;
use std::sync::Arc;

use product_core::error::Result;
use product_mcp::registry::{CustomDispatch, ToolRegistry};

use crate::state::ServeState;

/// The registry `ddd serve` runs (also the seam tests drive directly).
pub fn build_registry(root: PathBuf) -> (Arc<ServeState>, ToolRegistry) {
    let state = Arc::new(ServeState::new(root.clone()));
    let dispatch_state = state.clone();
    let dispatch: CustomDispatch =
        Arc::new(move |name, args, _root| crate::dispatch::dispatch(&dispatch_state, name, args));
    let registry =
        ToolRegistry::with_tools(root, true, "ddd-mcp", crate::tools::build(), dispatch);
    (state, registry)
}

/// Serve MCP over stdio until EOF.
pub fn serve_stdio(root: PathBuf) -> Result<()> {
    let (_state, registry) = build_registry(root);
    product_mcp::stdio::run_stdio_registry(registry, None)
}
