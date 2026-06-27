//! Domain (What) graph tool definitions — CLI↔MCP parity for `domain` (FT-119).

use super::{schema_props, ToolDef};

/// Every `product_domain_*` tool (read + write; gating is per-`ToolDef`).
pub(super) fn all() -> Vec<ToolDef> {
    let mut tools = read_query_tools();
    tools.extend(read_inspect_tools());
    tools.extend(write_tools());
    tools
}

fn read_query_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_domain_list".to_string(),
            description: "List nodes in the captured What graph, optionally filtered by kind (entity, context, event, …).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"kind": {"type": "string"}, "product": {"type": "string"}}
            }),
        },
        ToolDef {
            name: "product_domain_show".to_string(),
            description: "Show a What-graph node's fields and its links (what changes/targets/projects it).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

fn read_inspect_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_domain_validate".to_string(),
            description: "Validate the What graph against the framework SHACL shapes; returns conformance + violations.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"product": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_domain_export".to_string(),
            description: "Export the What graph as Turtle.".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({"type": "object", "properties": {"product": {"type": "string"}}}),
        },
        ToolDef {
            name: "product_domain_context".to_string(),
            description: "Assemble an LLM context bundle around a node (focus + neighbourhood to a depth).".to_string(),
            requires_write: false,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "depth": {"type": "integer"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

/// Map a node kind to its struct's generated field schema. The `match` is
/// exhaustive over `NodeKind`, so adding a kind without wiring its schema is a
/// compile error — the schema can never silently fall behind the structs.
fn props_for(kind: product_core::pf::ids::NodeKind) -> serde_json::Map<String, serde_json::Value> {
    use product_core::pf::ids::NodeKind as K;
    use product_core::pf::model::*;
    match kind {
        K::BoundedContext => schema_props::<BoundedContext>(),
        K::Entity => schema_props::<Entity>(),
        K::ValueObject => schema_props::<ValueObject>(),
        K::Relation => schema_props::<Relation>(),
        K::Invariant => schema_props::<Invariant>(),
        K::ContextMapping => schema_props::<ContextMapping>(),
        K::Command => schema_props::<Command>(),
        K::Event => schema_props::<Event>(),
        K::ReadModel => schema_props::<ReadModel>(),
        K::WireframeStep => schema_props::<WireframeStep>(),
        K::Flow => schema_props::<Flow>(),
        K::Aio => schema_props::<Aio>(),
        K::ContextOfUse => schema_props::<ContextOfUse>(),
        K::ApplicationRoot => schema_props::<ApplicationRoot>(),
        K::WcagCriterion => schema_props::<WcagCriterion>(),
        K::Attestation => schema_props::<Attestation>(),
        K::ContentStore => schema_props::<ContentStore>(),
        K::DesignSystem => schema_props::<DesignSystem>(),
        K::Cio => schema_props::<Cio>(),
        K::Token => schema_props::<Token>(),
        K::ReificationRule => schema_props::<ReificationRule>(),
        K::ReferenceSet => schema_props::<ReferenceSet>(),
        K::DataShape => schema_props::<DataShape>(),
        K::ProductionDataset => schema_props::<ProductionDataset>(),
        K::System => schema_props::<System>(),
        K::Trigger => schema_props::<Trigger>(),
        K::UnreifiableRule => schema_props::<UnreifiableRule>(),
    }
}

/// Generated union of every writable node kind's fields. The structs are the
/// single source of truth; a `NodeKind` added to `all()` flows through
/// `props_for`'s exhaustive match. Nothing here is hand-maintained, so the
/// schema cannot drift from what the handler accepts.
fn node_field_props() -> serde_json::Map<String, serde_json::Value> {
    let mut props = serde_json::Map::new();
    for kind in product_core::pf::ids::NodeKind::all() {
        for (k, v) in props_for(kind) {
            props.entry(k).or_insert(v);
        }
    }
    // `kind` is the kind selector (not a struct field); `product` selects the repo.
    props.insert("kind".to_string(), serde_json::json!({"type": "string", "description": "entity | context | event | command | relation | …"}));
    props.insert("product".to_string(), serde_json::json!({"type": "string"}));
    props
}

fn write_tools() -> Vec<ToolDef> {
    let mut new_props = node_field_props();
    new_props.insert("kind".to_string(), serde_json::json!({"type": "string", "description": "entity | context | event | command | relation | …"}));
    new_props.insert("id".to_string(), serde_json::json!({"type": "string"}));
    let mut edit_props = node_field_props();
    edit_props.insert("id".to_string(), serde_json::json!({"type": "string"}));
    vec![
        ToolDef {
            name: "product_domain_new".to_string(),
            description: "Create a What-graph node: `kind` + `id` plus the node's fields (label, context, definition, changes, targets, emits, …). Validated in-loop; returns { ok, node, violations }.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": new_props,
                "required": ["kind", "id"]
            }),
        },
        ToolDef {
            name: "product_domain_edit".to_string(),
            description: "Patch a What-graph node's fields by id; re-validated in-loop. Returns { ok, node, violations }.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": edit_props,
                "required": ["id"]
            }),
        },
        ToolDef {
            name: "product_domain_rm".to_string(),
            description: "Delete a What-graph node by id; reports any references the deletion leaves dangling.".to_string(),
            requires_write: true,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "product": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_schema_props() -> serde_json::Map<String, serde_json::Value> {
        let tools = write_tools();
        let new = tools
            .iter()
            .find(|t| t.name == "product_domain_new")
            .expect("product_domain_new tool");
        new.input_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .cloned()
            .expect("properties object")
    }

    #[test]
    fn array_and_bool_fields_keep_their_real_type() {
        // The `domain-schema-parity` gate: array/bool struct fields must surface
        // as array/boolean in the generated schema so a schema-typed MCP client
        // encodes them correctly (the regression that started this).
        let props = new_schema_props();
        for field in ["emits", "projects", "steps", "glossary"] {
            let ty = props.get(field).and_then(|f| f.get("type")).and_then(|t| t.as_str());
            assert_eq!(ty, Some("array"), "{field} must be an array");
        }
        let agg = props
            .get("is_aggregate_root")
            .and_then(|f| f.get("type"))
            .and_then(|t| t.as_str());
        assert_eq!(agg, Some("boolean"), "is_aggregate_root must be a boolean");
        assert!(props.len() > 25, "expected the union of all node fields, got {}", props.len());
    }

    #[test]
    fn every_node_kind_generates_a_schema() {
        // `props_for`'s match is exhaustive (compile-enforced over NodeKind);
        // this guards that schemars yields a non-empty schema for every kind, so
        // a newly added kind cannot ship with an empty/broken field set.
        for kind in product_core::pf::ids::NodeKind::all() {
            assert!(!props_for(kind).is_empty(), "{kind:?} produced no schema properties");
        }
    }
}
