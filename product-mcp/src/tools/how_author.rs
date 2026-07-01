//! Tool definitions for authoring the How contract — the Why cascade plus the
//! application/infrastructure contracts (CLI↔MCP parity for `product how`).

use product_core::pf::how::{
    ApplicationContract, ContractStatement, InfrastructureContract, InterfaceContract, Pattern,
    Principle, Resource, TopDecision,
};
use serde_json::Value;

use super::{schema_props, ToolDef};

fn write(name: &str, description: &str, props: Value, required: Value) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: true,
        input_schema: serde_json::json!({"type": "object", "properties": props, "required": required}),
    }
}

/// The union of every Why-cascade + contract element's fields, generated from the
/// `how.rs` structs so a schema-typed client encodes each field with its real
/// JSON type. The structs are the single source of truth (schema-single-source);
/// nothing here is hand-listed, so the schema cannot drift from the handler.
fn how_field_props() -> serde_json::Map<String, Value> {
    let mut props = serde_json::Map::new();
    let sources = [
        schema_props::<TopDecision>(),
        schema_props::<Principle>(),
        schema_props::<Pattern>(),
        schema_props::<InterfaceContract>(),
        schema_props::<ContractStatement>(),
        schema_props::<ApplicationContract>(),
        schema_props::<InfrastructureContract>(),
        schema_props::<Resource>(),
    ];
    for src in sources {
        for (k, v) in src {
            props.entry(k).or_insert(v);
        }
    }
    props
}

/// The how-element field union plus a `<selector>` enum key and `id`.
fn selector_props(selector: &str, choices: &str) -> Value {
    let mut props = how_field_props();
    props.insert(selector.to_string(), serde_json::json!({"type": "string", "description": choices}));
    props.insert("id".to_string(), serde_json::json!({"type": "string"}));
    Value::Object(props)
}

pub(super) fn all() -> Vec<ToolDef> {
    let add_props = selector_props("element", "decision | principle | pattern | interface | app-statement | resource");
    let set_props = selector_props("target", "app-contract | infra-contract | version | realises-version");
    let edit_props = selector_props("element", "decision | principle | pattern | interface");

    vec![
        write(
            "product_how_init",
            "Scaffold a fresh How contract for an archetype at .product/how-contract.yaml. Returns { ok, created }.",
            serde_json::json!({"archetype": {"type": "string"}, "product": {"type": "string"}}),
            serde_json::json!([]),
        ),
        write(
            "product_how_add",
            "Add a Why-cascade element or contract part — `element` (decision | principle | pattern | interface | app-statement | resource) + `id` plus the element's fields. Validated in-loop; returns { ok, id, element, violations }.",
            add_props,
            serde_json::json!(["element", "id"]),
        ),
        write(
            "product_how_set",
            "Set a singleton — `target` (app-contract | infra-contract) + `id` plus its fields, or a §7.3 version (version | realises-version) where `id` carries the version string. Validated in-loop; returns { ok, id, element, violations }.",
            set_props,
            serde_json::json!(["target", "id"]),
        ),
        write(
            "product_how_edit",
            "Patch a Why-cascade element by `element` (decision | principle | pattern | interface) + `id`, keeping unmentioned fields. Validated in-loop; returns { ok, id, element, violations }.",
            edit_props,
            serde_json::json!(["element", "id"]),
        ),
        write(
            "product_how_rm",
            "Remove a Why-cascade element or contract part by `id`. Re-validated in-loop; returns { ok, id, removed, violations }.",
            serde_json::json!({"id": {"type": "string"}}),
            serde_json::json!(["id"]),
        ),
    ]
}
