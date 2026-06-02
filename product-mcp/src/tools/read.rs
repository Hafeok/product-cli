//! Read-only tool definitions for the MCP surface.

use super::ToolDef;

/// Aggregate every read-only tool registered with the MCP server.
pub(super) fn all() -> Vec<ToolDef> {
    let mut tools = product_tools();
    tools.extend(feature_tools());
    tools.extend(adr_and_test_tools());
    tools.extend(pattern_read_tools());
    tools.extend(graph_tools());
    tools.extend(health_tools());
    tools.extend(agent_context_tools());
    tools.extend(prompts_tools());
    tools
}

// Read tools: patterns (FT-070, ADR-050)
fn pattern_read_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_pattern_list".to_string(),
            description: "List patterns, optionally filtered by status (live | deprecated).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"status": {"type": "string"}}
            }),
        },
        ToolDef {
            name: "product_pattern_show".to_string(),
            description: "Show a pattern's front-matter, links, and body.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

// Read tools: product identity (FT-039)
fn product_tools() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "product_responsibility".to_string(),
        description:
            "Get the product name and responsibility statement. This is the first call an agent should make in any session."
                .to_string(),
        requires_write: false,
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
    }]
}

// Read tools: context and features
fn feature_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_context".to_string(),
            description: "Assemble a context bundle for a feature or ADR. Pass `target` to render through a per-model template (FT-063).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "depth": {"type": "integer", "default": 1},
                    "target": {"type": "string", "description": "Per-model template name (e.g. claude-opus, gpt-4-markdown). Falls back to [context].default-target."}
                },
                "required": ["id"]
            }),
        },
        ToolDef {
            name: "product_feature_list".to_string(),
            description: "List all features with phase, status, and title".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"phase": {"type": "integer"}, "status": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_feature_show".to_string(),
            description: "Show a feature's full details".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
        ToolDef {
            name: "product_feature_deps".to_string(),
            description: "Show the dependency tree for a feature".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
    ]
}

// Read tools: ADRs and test criteria
fn adr_and_test_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_adr_list".to_string(),
            description: "List all ADRs".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_adr_show".to_string(),
            description: "Show an ADR's full details".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
        ToolDef {
            name: "product_test_show".to_string(),
            description: "Show a test criterion's details".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
    ]
}

// Read tools: graph operations, impact, gap analysis
fn graph_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_graph_check".to_string(),
            description: "Validate graph links and report errors/warnings".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "product_graph_central".to_string(),
            description: "Show top ADRs by betweenness centrality".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"top": {"type": "integer", "default": 10}}}),
        },
        ToolDef {
            name: "product_impact".to_string(),
            description: "Show what depends on an artifact".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
        ToolDef {
            name: "product_gap_check".to_string(),
            description: "Run gap analysis on an ADR".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"adr_id": {"type": "string"}}}),
        },
    ]
}

// Read tools: health checks (FT-059) — drift check and preflight
fn health_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_drift_check".to_string(),
            description:
                "Structural spec-vs-code drift check (read-only). Mirrors `product drift check`. With no arguments, aggregates across every ADR in the graph. With `id: ADR-NNN` checks one ADR; with `id: FT-NNN` uses the completion tag (D003 if changed). With `all_complete: true` iterates every complete-tagged feature. Returns a unified envelope with `status`, `findings`, `summary`. Errors via E022 / E023."
                    .to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Optional ADR or Feature ID. Mutually exclusive with all_complete."},
                    "files": {"type": "array", "items": {"type": "string"}, "description": "Optional explicit source files to scope the check to."},
                    "all_complete": {"type": "boolean", "description": "When true, iterate every complete feature with a completion tag."}
                }
            }),
        },
        ToolDef {
            name: "product_preflight".to_string(),
            description:
                "Pre-flight analysis for a feature (read-only). Mirrors `product preflight FT-XXX`. Returns cross-cutting ADR coverage, domain coverage, and dependency availability in a unified envelope. Errors via E022 (unknown id) and E024 (TC runner config missing on an active feature)."
                    .to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Feature ID — FT-NNN."}
                },
                "required": ["id"]
            }),
        },
    ]
}

// Read tools: agent context and schema (ADR-031)
fn agent_context_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_schema".to_string(),
            description: "Get the front-matter schema for an artifact type (feature, adr, test, dep, formal) or all types".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"artifact_type": {"type": "string", "description": "Artifact type: feature, adr, test, dep, or formal (AISP formal-block grammar reference). Omit for all schemas."}}}),
        },
        ToolDef {
            name: "product_agent_context".to_string(),
            description: "Get the full AGENTS.md content — working protocol, schemas, repo state, domains, and tool guide".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
    ]
}

// Read tools: authoring prompts (ADR-022)
fn prompts_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_prompts_list".to_string(),
            description: "List available authoring session prompts with version numbers".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "product_prompts_get".to_string(),
            description: "Get the content of an authoring session prompt by name".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"name": {"type": "string", "description": "Prompt name: author-feature, author-adr, author-review, implement"}}, "required": ["name"]}),
        },
    ]
}
