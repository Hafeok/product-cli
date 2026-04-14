//! MCP tool definitions — read/write tool schemas (ADR-020)

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub requires_write: bool,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Build the complete list of tool definitions
pub fn build_tool_list() -> Vec<ToolDef> {
    let mut tools = read_feature_tools();
    tools.extend(read_adr_and_test_tools());
    tools.extend(read_graph_tools());
    tools.extend(write_create_tools());
    tools.extend(write_update_tools());
    tools
}

// ---------------------------------------------------------------------------
// Read tools: context and features
// ---------------------------------------------------------------------------

fn read_feature_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_context".to_string(),
            description: "Assemble a context bundle for a feature or ADR".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "depth": {"type": "integer", "default": 1}}, "required": ["id"]}),
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

// ---------------------------------------------------------------------------
// Read tools: ADRs and test criteria
// ---------------------------------------------------------------------------

fn read_adr_and_test_tools() -> Vec<ToolDef> {
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

// ---------------------------------------------------------------------------
// Read tools: graph operations, impact, gap analysis
// ---------------------------------------------------------------------------

fn read_graph_tools() -> Vec<ToolDef> {
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

// ---------------------------------------------------------------------------
// Write tools: creating new artifacts
// ---------------------------------------------------------------------------

fn write_create_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_feature_new".to_string(),
            description: "Create a new feature file".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}, "phase": {"type": "integer", "default": 1}}, "required": ["title"]}),
        },
        ToolDef {
            name: "product_adr_new".to_string(),
            description: "Create a new ADR file".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}}, "required": ["title"]}),
        },
        ToolDef {
            name: "product_test_new".to_string(),
            description: "Create a new test criterion file".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}, "test_type": {"type": "string", "default": "scenario"}}, "required": ["title"]}),
        },
        ToolDef {
            name: "product_feature_link".to_string(),
            description: "Link a feature to an ADR, test, or dependency".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "adr": {"type": "string"}, "test": {"type": "string"}, "dep": {"type": "string"}}, "required": ["id"]}),
        },
    ]
}

// ---------------------------------------------------------------------------
// Write tools: status updates, body edits, amendments
// ---------------------------------------------------------------------------

fn write_update_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_feature_status".to_string(),
            description: "Set feature status".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
        },
        ToolDef {
            name: "product_adr_status".to_string(),
            description: "Set ADR status".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}, "by": {"type": "string"}}, "required": ["id", "status"]}),
        },
        ToolDef {
            name: "product_test_status".to_string(),
            description: "Set test criterion status".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
        },
        ToolDef {
            name: "product_body_update".to_string(),
            description: "Update the markdown body of a feature, ADR, or test criterion (preserves front-matter). Cannot modify accepted ADR bodies — use product_adr_amend instead.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "body": {"type": "string"}}, "required": ["id", "body"]}),
        },
        ToolDef {
            name: "product_adr_amend".to_string(),
            description: "Record a legitimate amendment to an accepted ADR with mandatory reason and audit trail (ADR-032)".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "reason": {"type": "string"}}, "required": ["id", "reason"]}),
        },
    ]
}
