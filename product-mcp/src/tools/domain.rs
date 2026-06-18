//! Domain (What) graph tool definitions — CLI↔MCP parity for `domain` (FT-119).

use super::ToolDef;

/// Every `product_domain_*` tool (read + write; gating is per-`ToolDef`).
pub(super) fn all() -> Vec<ToolDef> {
    let mut tools = read_query_tools();
    tools.extend(read_inspect_tools());
    tools.extend(write_tools());
    tools
}

fn read_query_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_domain_list".to_string(),
            description: "List nodes in the captured What graph, optionally filtered by kind (entity, context, event, …).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"kind": {"type": "string"}, "product": {"type": "string"}}
            }),
        },
        ToolDef {
            name: "product_domain_show".to_string(),
            description: "Show a What-graph node's fields and its links (what changes/targets/projects it).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

fn read_inspect_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_domain_validate".to_string(),
            description: "Validate the What graph against the framework SHACL shapes; returns conformance + violations.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"product": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_domain_export".to_string(),
            description: "Export the What graph as Turtle.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"product": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_domain_context".to_string(),
            description: "Assemble an LLM context bundle around a node (focus + neighbourhood to a depth).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "depth": {"type": "integer"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

fn write_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_domain_new".to_string(),
            description: "Create a What-graph node: `kind` + `id` plus the node's fields (label, context, definition, changes, targets, emits, …). Validated in-loop; returns { ok, node, violations }.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kind": {"type": "string", "description": "entity | context | event | command | relation | …"},
                    "id": {"type": "string"},
                    "product": {"type": "string"}
                },
                "required": ["kind", "id"]
            }),
        },
        ToolDef {
            name: "product_domain_edit".to_string(),
            description: "Patch a What-graph node's fields by id; re-validated in-loop. Returns { ok, node, violations }.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
        ToolDef {
            name: "product_domain_rm".to_string(),
            description: "Delete a What-graph node by id; reports any references the deletion leaves dangling.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}
