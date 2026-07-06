//! Authoring-scope tool definitions — CLI↔MCP parity for `product scope` (§14).
//!
//! Mirrors the `product scope` family. Authoring scopes are an intake / What
//! concept, so these gate to the What session phase (the `product_scope_`
//! prefix in `workflow::phase_of`). `add` vendors a scope, `validate` re-checks
//! a stored one, `enforce` runs the §14.3 oracle, `join` the §14.4 join.

use super::ToolDef;

fn read(name: &str, description: &str, props: serde_json::Value, required: serde_json::Value) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: false,
        input_schema: serde_json::json!({"type": "object", "properties": props, "required": required}),
    }
}

fn write(name: &str, description: &str, props: serde_json::Value, required: serde_json::Value) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: true,
        input_schema: serde_json::json!({"type": "object", "properties": props, "required": required}),
    }
}

pub(super) fn all() -> Vec<ToolDef> {
    let tooled = serde_json::json!({"tool": {"type": "string"}, "product": {"type": "string"}});
    vec![
        read("product_scope_list", "List the authoring scopes stored under .product/authoring-scopes/.",
            serde_json::json!({"product": {"type": "string"}}), serde_json::json!([])),
        read("product_scope_show", "Show a stored authoring scope (tool, adapter, authors, excluded, process-slice).",
            tooled.clone(), serde_json::json!(["tool"])),
        read("product_scope_validate", "Validate a stored authoring scope (§14.2): wholeness, kind-vocabulary membership, the derived-kind rule.",
            tooled.clone(), serde_json::json!(["tool"])),
        read("product_scope_enforce", "Run the §14.3 enforcement oracle over a tool submission: accept in-scope authorship, reject out-of-scope content regardless of quality, split the gap into unauthored-within-scope vs outside-scope.",
            serde_json::json!({"tool": {"type": "string"}, "submission_path": {"type": "string", "description": "Path to the submission JSON (authored + unauthored-candidates), relative to the repo root"}, "product": {"type": "string"}}),
            serde_json::json!(["tool", "submission_path"])),
        read("product_scope_join", "Run the §14.4 completeness join across every stored scope: report per required kind covered (by whom) / coverable-but-unauthored / uncovered.",
            serde_json::json!({
                "required": {"type": "array", "items": {"type": "string"}, "description": "Required What-element kinds"},
                "authored": {"type": "object", "description": "Optional map of tool -> array of kinds it has authored"},
                "product": {"type": "string"}
            }),
            serde_json::json!(["required"])),
        write("product_scope_add", "Validate an authoring-scope file (§14.2) and vendor it under .product/authoring-scopes/<tool>.yaml. An unwhole scope is rejected; nothing is saved.",
            serde_json::json!({"file": {"type": "string", "description": "Path to the authoring-scope file (YAML or JSON), relative to the repo root"}, "product": {"type": "string"}}),
            serde_json::json!(["file"])),
    ]
}
