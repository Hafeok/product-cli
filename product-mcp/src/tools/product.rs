//! Product tool definitions — CLI↔MCP parity for `product product`.

use super::ToolDef;

fn def(name: &str, description: &str, write: bool, props: serde_json::Value, required: serde_json::Value) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: write,
        input_schema: serde_json::json!({"type": "object", "properties": props, "required": required}),
    }
}

pub(super) fn all() -> Vec<ToolDef> {
    let id = serde_json::json!({"id": {"type": "string", "description": "The product id"}});
    let new_props = serde_json::json!({
        "id": {"type": "string", "description": "The product id (^[A-Za-z][A-Za-z0-9_-]*$)"},
        "title": {"type": "string", "description": "A human title for the product"}
    });
    vec![
        def("product_product_list", "List every product home under .product/products/ (plus legacy homes).",
            false, serde_json::json!({}), serde_json::json!([])),
        def("product_product_show", "Show a product's home and its What/How/Delivery state.",
            false, id, serde_json::json!(["id"])),
        def("product_product_new", "Add a product — creates its home .product/products/<id>/ with an empty What graph.",
            true, new_props, serde_json::json!(["id"])),
    ]
}
