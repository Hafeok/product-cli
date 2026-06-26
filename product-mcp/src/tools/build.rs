//! Build (§5) tool definition — run a deliverable build over MCP.

use super::ToolDef;

/// The `product_build_run` tool: the Build-phase entry point that runs the
/// deliverable build to completion and returns its gate/session report.
pub(super) fn all() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "product_build_run".to_string(),
        description: "Run a deliverable build (§5): assemble the SPMC context, dispatch the worker, run the LSP + verify gates, and return the gate/session report. Long-running; blocks until the build completes.".to_string(),
        requires_write: true,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "deliverable": {"type": "string", "description": "The deliverable id to build"},
                "role": {"type": "string", "description": "Worker role to resolve to a capability (default: implementer)"},
                "jobs": {"type": "integer", "description": "Max work units to dispatch concurrently (default: 1)"},
                "dry_run": {"type": "boolean", "description": "Assemble + show the context and gate plan without dispatching"},
                "lsp": {"type": "boolean", "description": "Diagnose + fix the worker's Rust output with rust-analyzer before gating"},
                "no_verify": {"type": "boolean", "description": "Skip the §6 verify step"},
                "max_rounds": {"type": "integer", "description": "Max diagnose→fix rounds per gate (default: 3)"},
                "budget": {"type": "integer", "description": "Token budget; escalation stops once total tokens reach it"},
                "product": {"type": "string"}
            },
            "required": ["deliverable"]
        }),
    }]
}
