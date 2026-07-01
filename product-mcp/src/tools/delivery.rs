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
        // feature (§7.1 — a reference to a subgraph of one or more flows)
        read("product_feature_list", "List delivery features.", serde_json::json!({}), serde_json::json!([])),
        read("product_feature_show", "Show a feature's pointer.", named.clone(), serde_json::json!(["name"])),
        read("product_feature_context", "Assemble the LLM build-context for a feature (the reachable What subgraph).",
            serde_json::json!({"name": {"type": "string"}, "depth": {"type": "integer"}, "product": {"type": "string"}}), serde_json::json!(["name"])),
        write("product_feature_new", "Create a feature pointing at subgraph(s) of the event model.",
            serde_json::json!({"id": {"type": "string"}, "anchors": {"type": "array", "items": {"type": "string"}}, "depth": {"type": "integer"}, "product": {"type": "string"}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "anchors"])),
        // deliverable — the shippable unit wrapping one feature
        read("product_deliverable_list", "List deliverables.", serde_json::json!({}), serde_json::json!([])),
        read("product_deliverable_show", "Show a deliverable.", named.clone(), serde_json::json!(["name"])),
        read("product_deliverable_done", "Compute whether a deliverable is done (§7.2).", named.clone(), serde_json::json!(["name"])),
        write("product_deliverable_new", "Create a deliverable pointing at one feature (§7.1) with acceptance criteria.",
            serde_json::json!({"id": {"type": "string"}, "feature": {"type": "string"}, "acceptance": {"type": "array", "items": {"type": "string"}}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "feature"])),
        write("product_deliverable_accept", "Record an acceptance criterion's verdict (status: passing | failing).",
            serde_json::json!({"id": {"type": "string"}, "criterion": {"type": "string"}, "status": {"type": "string"}}),
            serde_json::json!(["id", "criterion", "status"])),
        write("product_deliverable_runner", "Bind a §6 verification runner to an acceptance criterion so the build auto-verifies it. `runner`: cargo-test (args = test filter) | shell (args = command).",
            serde_json::json!({"id": {"type": "string"}, "criterion": {"type": "string"}, "runner": {"type": "string"}, "args": {"type": "string"}}),
            serde_json::json!(["id", "criterion", "runner"])),
        // release
        read("product_release_list", "List releases.", serde_json::json!({}), serde_json::json!([])),
        read("product_release_show", "Show a release.", named.clone(), serde_json::json!(["name"])),
        read("product_release_done", "Compute whether a release is done — members done + cut closed (§7.2).", named.clone(), serde_json::json!(["name"])),
        write("product_release_new", "Create a release grouping delivery features.",
            serde_json::json!({"id": {"type": "string"}, "features": {"type": "array", "items": {"type": "string"}}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "features"])),
        // target version (§7.3)
        read("product_target_list", "List target versions.", serde_json::json!({}), serde_json::json!([])),
        read("product_target_show", "Show a target version.", named.clone(), serde_json::json!(["name"])),
        read("product_target_direction", "Compute the gap to a target — the unrealised features (§7.3).", named, serde_json::json!(["name"])),
        write("product_target_new", "Declare a target version as a set of features (deliverables), some not yet realised.",
            serde_json::json!({"id": {"type": "string"}, "version": {"type": "string"}, "features": {"type": "array", "items": {"type": "string"}}, "force": {"type": "boolean"}}),
            serde_json::json!(["id", "features"])),
    ]
}
