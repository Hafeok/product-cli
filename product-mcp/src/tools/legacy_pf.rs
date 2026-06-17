//! Tool definitions for the remaining framework families — archetype, cell,
//! how, work-unit, dep (read-only inspection; CLI↔MCP parity).

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
    let named = serde_json::json!({"name": {"type": "string"}, "product": {"type": "string"}});
    let id = serde_json::json!({"id": {"type": "string"}});
    let none = serde_json::json!({});
    vec![
        // archetype
        read("product_archetype_list", "List the archetypes under .product/archetypes/.", none.clone(), serde_json::json!([])),
        read("product_archetype_show", "Show an assembled archetype's parts.", named.clone(), serde_json::json!(["name"])),
        read("product_archetype_validate", "Validate an archetype (How + layout + cells + coherence).", named.clone(), serde_json::json!(["name"])),
        read("product_archetype_check", "Check an archetype's layout against the repository tree (§4.3).", named.clone(), serde_json::json!(["name"])),
        // cell
        read("product_cell_show", "Show the task-type cell (.product/cell.yaml).", none.clone(), serde_json::json!([])),
        read("product_cell_validate", "Validate the cell against the What graph + How contract.", serde_json::json!({"product": {"type": "string"}}), serde_json::json!([])),
        // how
        read("product_how_show", "Show the How contract summary (.product/how-contract.yaml).", none.clone(), serde_json::json!([])),
        read("product_how_validate", "Validate the How contract against the framework shapes.", none.clone(), serde_json::json!([])),
        read("product_how_export", "Export the How contract as Turtle.", none.clone(), serde_json::json!([])),
        // work-unit
        read("product_work_unit_show", "Show the work unit (.product/work-unit.yaml).", none.clone(), serde_json::json!([])),
        read("product_work_unit_validate", "Validate the work unit against the What graph + How contract.", serde_json::json!({"product": {"type": "string"}}), serde_json::json!([])),
        // dep
        read("product_dep_list", "List dependencies in the knowledge graph.", none, serde_json::json!([])),
        read("product_dep_show", "Show a dependency's front matter.", id.clone(), serde_json::json!(["id"])),
        read("product_dep_features", "List the features that use a dependency.", id, serde_json::json!(["id"])),
    ]
}
