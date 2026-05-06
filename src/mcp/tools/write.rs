//! Mutating tool definitions for the MCP surface (gated by `mcp.write`).

use super::ToolDef;

/// Aggregate every write tool registered with the MCP server.
pub(super) fn all() -> Vec<ToolDef> {
    let mut tools = create_tools();
    tools.extend(field_domain_tools());
    tools.extend(field_adr_tools());
    tools.extend(field_test_tools());
    tools.extend(status_tools());
    tools.extend(adr_lifecycle_tools());
    tools.extend(request_tools());
    tools
}

// Write tools: creating new artifacts
fn create_tools() -> Vec<ToolDef> {
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

// Write tools: domain edits on features and ADRs (FT-038)
fn field_domain_tools() -> Vec<ToolDef> {
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

// Write tools: ADR scoping, supersession, source-files
fn field_adr_tools() -> Vec<ToolDef> {
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

// Write tools: TC runner configuration
fn field_test_tools() -> Vec<ToolDef> {
    vec![ToolDef {
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
    }]
}

// Write tools: status updates and body edits
fn status_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_feature_status".to_string(),
            description: "Set feature status".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
        },
        ToolDef {
            name: "product_test_status".to_string(),
            description: "Set test criterion status".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
        },
        ToolDef {
            name: "product_body_update".to_string(),
            description: "Update the markdown body of a feature, ADR, test criterion, or dependency (preserves front-matter). Cannot modify accepted ADR bodies — use product_adr_amend instead.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string", "description": "Artifact ID — FT-NNN, ADR-NNN, TC-NNN, or DEP-NNN."}, "body": {"type": "string"}}, "required": ["id", "body"]}),
        },
    ]
}

// Write tools: ADR lifecycle (status transitions and amendments) — FT-046
fn adr_lifecycle_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_adr_status".to_string(),
            description: "Write an ADR status transition (FT-046). Handles proposed<->abandoned, *->superseded (with bidirectional 'by' link), and accepted->abandoned. Rejects 'accepted' with E020 (CLI-only, ADR-032). Rejects accepted->proposed with E021.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "status": {"type": "string", "enum": ["proposed", "superseded", "abandoned"]},
                    "by": {"type": "string", "description": "When status is 'superseded', the ADR that supersedes this one."}
                },
                "required": ["id", "status"]
            }),
        },
        ToolDef {
            name: "product_adr_amend".to_string(),
            description: "Record an amendment to an accepted ADR with mandatory reason and audit trail (ADR-032). Optional 'body' atomically replaces the markdown body and recomputes the content-hash (FT-046). Payloads carrying 'status' are rejected with E019.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "reason": {"type": "string"},
                    "body": {"type": "string", "description": "Optional new markdown body."}
                },
                "required": ["id", "reason"]
            }),
        },
    ]
}

// Request tools (FT-041, ADR-038)
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
