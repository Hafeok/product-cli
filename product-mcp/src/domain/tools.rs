//! Domain MCP tool definitions sourced from the canonical schema.
//!
//! The tool surface is the single source of truth in
//! `product-author-domain.tools.json` (vendored next to this file). Parsing it
//! at startup keeps the `tools/list` response byte-faithful to the published
//! contract instead of hand-restating every `inputSchema` in Rust.

use crate::tools::ToolDef;
use serde_json::{json, Value};

const TOOLS_JSON: &str = include_str!("tools.json");

/// Build the domain tool list from the vendored schema. All tools run inside
/// the dedicated authoring server, so none are gated by `requires_write`.
pub fn tool_defs() -> Vec<ToolDef> {
    let parsed: Value = serde_json::from_str(TOOLS_JSON).unwrap_or(Value::Null);
    let mut out = Vec::new();
    let Some(arr) = parsed.get("tools").and_then(|t| t.as_array()) else {
        return out;
    };
    for t in arr {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        out.push(ToolDef {
            name: name.to_string(),
            description: t.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            requires_write: false,
            input_schema: t.get("inputSchema").cloned().unwrap_or_else(|| json!({ "type": "object" })),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_full_tool_surface() {
        let tools = tool_defs();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        // §4 of the spec lists 17 tools.
        assert_eq!(tools.len(), 17, "got {:?}", names);
        for required in [
            "session_start", "session_state", "session_finalize",
            "add_bounded_context", "add_entity", "add_value_object", "add_relation",
            "add_invariant", "add_context_mapping", "add_command", "add_event",
            "add_read_model", "add_wireframe_step", "add_flow",
            "open_questions", "query", "validate",
        ] {
            assert!(names.contains(&required), "missing tool {required}");
        }
        // Every tool carries an object inputSchema.
        for t in &tools {
            assert_eq!(t.input_schema.get("type").and_then(|v| v.as_str()), Some("object"), "{}", t.name);
        }
    }
}
