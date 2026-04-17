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
    let mut tools = read_product_tools();
    tools.extend(read_feature_tools());
    tools.extend(read_adr_and_test_tools());
    tools.extend(read_graph_tools());
    tools.extend(read_agent_context_tools());
    tools.extend(read_prompts_tools());
    tools.extend(write_create_tools());
    tools.extend(write_field_domain_tools());
    tools.extend(write_field_adr_tools());
    tools.extend(write_field_test_tools());
    tools.extend(write_update_tools());
    tools.extend(request_tools());
    tools
}

// ---------------------------------------------------------------------------
// Request tools (FT-041, ADR-038)
// ---------------------------------------------------------------------------

fn request_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_request_validate".to_string(),
            description: "Validate a request YAML (type: create | change | create-and-change) without writing. Returns every finding in one pass.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "request_yaml": {"type": "string", "description": "Full YAML source of the request"}
                },
                "required": ["request_yaml"]
            }),
        },
        ToolDef {
            name: "product_request_apply".to_string(),
            description: "Validate a request YAML and apply it atomically. Returns created and changed arrays with assigned IDs.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "request_yaml": {"type": "string", "description": "Full YAML source of the request"}
                },
                "required": ["request_yaml"]
            }),
        },
    ]
}

// ---------------------------------------------------------------------------
// Read tools: product identity (FT-039)
// ---------------------------------------------------------------------------

fn read_product_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_responsibility".to_string(),
            description: "Get the product name and responsibility statement. This is the first call an agent should make in any session.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
    ]
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
// Read tools: agent context and schema (ADR-031)
// ---------------------------------------------------------------------------

fn read_agent_context_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_schema".to_string(),
            description: "Get the front-matter schema for an artifact type (feature, adr, test, dep) or all types".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"artifact_type": {"type": "string", "description": "Artifact type: feature, adr, test, dep. Omit for all schemas."}}}),
        },
        ToolDef {
            name: "product_agent_context".to_string(),
            description: "Get the full AGENTS.md content — working protocol, schemas, repo state, domains, and tool guide".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
    ]
}

// ---------------------------------------------------------------------------
// Read tools: authoring prompts (ADR-022)
// ---------------------------------------------------------------------------

fn read_prompts_tools() -> Vec<ToolDef> {
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

fn write_field_domain_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_feature_domain".to_string(),
            description: "Add or remove concern domains on a feature. Domains validated against product.toml.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}, "add": {"type": "array", "items": {"type": "string"}},
                    "remove": {"type": "array", "items": {"type": "string"}}
                }, "required": ["id"]
            }),
        },
        ToolDef {
            name: "product_feature_acknowledge".to_string(),
            description: "Acknowledge a domain gap on a feature with mandatory reasoning.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}, "domain": {"type": "string"},
                    "reason": {"type": "string"}, "remove": {"type": "boolean", "default": false}
                }, "required": ["id", "domain"]
            }),
        },
        ToolDef {
            name: "product_adr_domain".to_string(),
            description: "Add or remove concern domains on an ADR. Domains validated against product.toml.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}, "add": {"type": "array", "items": {"type": "string"}},
                    "remove": {"type": "array", "items": {"type": "string"}}
                }, "required": ["id"]
            }),
        },
    ]
}

fn write_field_adr_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_adr_scope".to_string(),
            description: "Set ADR scope: cross-cutting, domain, or feature-specific.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "scope": {"type": "string", "enum": ["cross-cutting", "domain", "feature-specific"]}
                }, "required": ["id", "scope"]
            }),
        },
        ToolDef {
            name: "product_adr_supersede".to_string(),
            description: "Declare or remove ADR supersession (bidirectional, with cycle detection).".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}, "supersedes": {"type": "string"},
                    "remove": {"type": "string"}
                }, "required": ["id"]
            }),
        },
        ToolDef {
            name: "product_adr_source_files".to_string(),
            description: "Add or remove governed source files on an ADR for drift detection.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}, "add": {"type": "array", "items": {"type": "string"}},
                    "remove": {"type": "array", "items": {"type": "string"}}
                }, "required": ["id"]
            }),
        },
    ]
}

fn write_field_test_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_test_runner".to_string(),
            description: "Configure test runner: runner type, arguments, timeout, and prerequisites.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "runner": {"type": "string", "enum": ["cargo-test", "bash", "pytest", "custom"]},
                    "args": {"type": "string"}, "timeout": {"type": "string"},
                    "requires": {"type": "array", "items": {"type": "string"}}
                }, "required": ["id"]
            }),
        },
    ]
}

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
