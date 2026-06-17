//! Decider tool definitions — CLI↔MCP parity for `product decider` (§3.3).

use super::ToolDef;

fn read(name: &str, description: &str, props: serde_json::Value, required: serde_json::Value) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: false,
        input_schema: serde_json::json!({"type": "object", "properties": props, "required": required}),
    }
}

pub(super) fn all() -> Vec<ToolDef> {
    let name = serde_json::json!({"name": {"type": "string"}, "product": {"type": "string"}});
    vec![
        read("product_decider_list", "List the deciders under .product/deciders/.",
            serde_json::json!({}), serde_json::json!([])),
        read("product_decider_show", "Show a Decider's derived signature.", name.clone(), serde_json::json!(["name"])),
        read("product_decider_validate", "Validate a Decider's signature against the event model (§3.3 drift rules).",
            name.clone(), serde_json::json!(["name"])),
        read("product_decider_simulate", "Simulate a Decider's scenarios — sound + complete before realisation.",
            name, serde_json::json!(["name"])),
        ToolDef {
            name: "product_decider_derive".to_string(),
            description: "Derive a Decider's signature for an aggregate from the What graph; writes the decider file.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"aggregate": {"type": "string"}, "product": {"type": "string"}, "force": {"type": "boolean"}},
                "required": ["aggregate"]
            }),
        },
    ]
}
