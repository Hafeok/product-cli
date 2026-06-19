//! Primitive tool definitions — CLI↔MCP parity for `product primitive` (§3.5).

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
    let name = serde_json::json!({"name": {"type": "string"}});
    vec![
        read("product_primitive_list", "List the named-algorithm primitives under .product/primitives/.",
            serde_json::json!({}), serde_json::json!([])),
        read("product_primitive_show", "Show a named-algorithm primitive's declaration.", name.clone(), serde_json::json!(["name"])),
        read("product_primitive_validate", "Validate a primitive declares a reference + I/O contract + oracle (§3.5).",
            name, serde_json::json!(["name"])),
    ]
}
