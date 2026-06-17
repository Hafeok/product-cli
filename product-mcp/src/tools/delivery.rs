//! Delivery tool definitions — CLI↔MCP parity for `slice`, `deliverable`,
//! `release` (§7).

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
    let named = serde_json::json!({"name": {"type": "string"}, "product": {"type": "string"}});
    vec![
        // slice
        read("product_slice_list", "List delivery slices.", serde_json::json!({}), serde_json::json!([])),
        read("product_slice_show", "Show a slice's pointer.", named.clone(), serde_json::json!(["name"])),
        read("product_slice_context", "Assemble the LLM build-context for a slice (the reachable What subgraph).",
            serde_json::json!({"name": {"type": "string"}, "depth": {"type": "integer"}, "product": {"type": "string"}}), serde_json::json!(["name"])),
        write("product_slice_new", "Create a slice pointing at section(s) of the event model.",
            serde_json::json!({"id": {"type": "string"}, "anchors": {"type": "array", "items": {"type": "string"}}, "depth": {"type": "integer"}, "product": {"type": "string"}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "anchors"])),
        // deliverable
        read("product_deliverable_list", "List delivery features (deliverables).", serde_json::json!({}), serde_json::json!([])),
        read("product_deliverable_show", "Show a deliverable.", named.clone(), serde_json::json!(["name"])),
        read("product_deliverable_done", "Compute whether a deliverable is done (§7.2).", named.clone(), serde_json::json!(["name"])),
        write("product_deliverable_new", "Create a deliverable pointing at one slice with acceptance criteria.",
            serde_json::json!({"id": {"type": "string"}, "slice": {"type": "string"}, "acceptance": {"type": "array", "items": {"type": "string"}}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "slice"])),
        write("product_deliverable_accept", "Record an acceptance criterion's verdict (status: passing | failing).",
            serde_json::json!({"id": {"type": "string"}, "criterion": {"type": "string"}, "status": {"type": "string"}}),
            serde_json::json!(["id", "criterion", "status"])),
        // release
        read("product_release_list", "List releases.", serde_json::json!({}), serde_json::json!([])),
        read("product_release_show", "Show a release.", named.clone(), serde_json::json!(["name"])),
        read("product_release_done", "Compute whether a release is done — members done + cut closed (§7.2).", named, serde_json::json!(["name"])),
        write("product_release_new", "Create a release grouping delivery features.",
            serde_json::json!({"id": {"type": "string"}, "features": {"type": "array", "items": {"type": "string"}}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "features"])),
    ]
}
