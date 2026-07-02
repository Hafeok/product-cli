//! Build (§5) tool definition — run a deliverable build over MCP.

use super::ToolDef;

/// The `product_build_run` tool: the Build-phase entry point that runs the
/// deliverable build to completion and returns its gate/session report.
pub(super) fn all() -> Vec<ToolDef> {
    vec![run_tool(), emit_tool(), verdict_tool()]
}

fn run_tool() -> ToolDef {
        ToolDef {
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
        }
}

fn emit_tool() -> ToolDef {
        ToolDef {
            name: "product_build_emit".to_string(),
            description: "Emit a self-contained SPMC prompt for a Claude Code -p session: the frozen What/How/Behaviour/Acceptance context plus the work-unit build plan (in dependency order) and the verify commands the agent must pass. Returns { ok, deliverable, spmc }. With `seam: true`, instead emits the §5.1 build-seam envelopes (work units by value with a content-hash identity) as { ok, deliverable, units }.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "deliverable": {"type": "string", "description": "The deliverable id to emit an SPMC for"},
                    "seam": {"type": "boolean", "description": "Emit §5.1 build-seam envelopes instead of an SPMC prompt"},
                    "product": {"type": "string"}
                },
                "required": ["deliverable"]
            }),
        }
}

fn verdict_tool() -> ToolDef {
        ToolDef {
            name: "product_build_verdict".to_string(),
            description: "Validate an inbound §5.1 build-seam verdict event against the canonical contract (required fields event-id/emitted-at/unit-ref/parent-deliverable/bundle-hash/verdict/tier-ran/cell-results/next-consequence, the pinned accepted/rejected/escalate vocabulary, a closed top-level envelope). Pass the event object as `event`. Returns { ok, … } or { ok: false, error }.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event": {"type": "object", "description": "The verdict event JSON to validate"}
                },
                "required": ["event"]
            }),
        }
}
