//! Design-system tool definitions — the §11 provider as an addressable artifact.
//!
//! Mirrors the `product design-system` CLI family, gated to the How phase by
//! the `product_design_system_` prefix: the design system is a §4.5 How-side
//! choice. `add` vendors a manifest, `bind` records it on the How contract,
//! `validate`/`couple` are the wholeness + coupling gates, `list`/`show` read.

use super::ToolDef;

pub(super) fn all() -> Vec<ToolDef> {
    vec![list_tool(), show_tool(), validate_tool(), couple_tool(), add_tool(), bind_tool()]
}

fn common_props(extra: serde_json::Value) -> serde_json::Value {
    let mut props = serde_json::json!({ "product": {"type": "string"} });
    if let (Some(base), Some(add)) = (props.as_object_mut(), extra.as_object()) {
        for (k, v) in add {
            base.insert(k.clone(), v.clone());
        }
    }
    props
}

fn list_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_list".to_string(),
        description: "List the design systems stored under .product/design-systems/, marking the How-bound one.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({ "type": "object", "properties": common_props(serde_json::json!({})) }),
    }
}

fn show_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_show".to_string(),
        description: "Show a stored design system: identity + hash, catalog, reification coverage, token surface, targets and themes. Defaults to the How-bound system.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": common_props(serde_json::json!({"id": {"type": "string", "description": "A stored design-system id (default: the How-bound one)"}}))
        }),
    }
}

fn validate_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_validate".to_string(),
        description: "Validate a stored design system: declaration wholeness (§11.3 — catalog, tokens, WCAG entities) plus the bundle check (an implementation per declared target, a token value per declared theme, templates on-catalog).".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": common_props(serde_json::json!({"id": {"type": "string", "description": "A stored design-system id (default: the How-bound one)"}}))
        }),
    }
}

fn couple_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_couple".to_string(),
        description: "Coupling check (§11.2): every AIO the What's UI steps reference has a reifying CIO for each declared context of use. The same gate `product reify` applies at plan time.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": common_props(serde_json::json!({"id": {"type": "string", "description": "A stored design-system id (default: the How-bound one)"}}))
        }),
    }
}

fn add_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_add".to_string(),
        description: "Validate a design-system manifest's declaration half (§11.3) and vendor it — plus every implementation source it references — into .product/design-systems/<id>/. An unwhole manifest is rejected; nothing is saved.".to_string(),
        requires_write: true,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": common_props(serde_json::json!({"manifest_path": {"type": "string", "description": "Path to the YAML design-system manifest, relative to the repo root"}})),
            "required": ["manifest_path"]
        }),
    }
}

fn bind_tool() -> ToolDef {
    ToolDef {
        name: "product_design_system_bind".to_string(),
        description: "Bind a stored design system to the How contract's screen-composition contract (§4.5) by id + version — the system every `product reify` backend resolves.".to_string(),
        requires_write: true,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": common_props(serde_json::json!({"id": {"type": "string", "description": "A stored design-system id"}})),
            "required": ["id"]
        }),
    }
}
