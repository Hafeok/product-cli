//! Projector tool definitions — CLI↔MCP parity for `product projector` (§3.4).

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
        read("product_projector_list", "List the projectors under .product/projectors/.",
            serde_json::json!({}), serde_json::json!([])),
        read("product_projector_show", "Show a Projector's derived fold signature.", name.clone(), serde_json::json!(["name"])),
        read("product_projector_validate", "Validate a Projector against the event model (§3.4 drift rules).",
            name.clone(), serde_json::json!(["name"])),
        read("product_projector_simulate", "Simulate a Projector's scenarios — sound + complete before realisation.",
            name, serde_json::json!(["name"])),
        ToolDef {
            name: "product_projector_derive".to_string(),
            description: "Derive a Projector's fold signature for a read model from the What graph; writes the projector file.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"read_model": {"type": "string"}, "product": {"type": "string"}, "force": {"type": "boolean"}},
                "required": ["read_model"]
            }),
        },
    ]
}
