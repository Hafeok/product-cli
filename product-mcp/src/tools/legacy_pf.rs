//! Tool definitions for the remaining framework families — archetype, cell,
//! how, work-unit, worker (inspection plus scaffolding; CLI↔MCP parity).

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
    let none = serde_json::json!({});
    vec![
        // archetype
        read("product_archetype_list", "List the archetypes under .product/archetypes/.", none.clone(), serde_json::json!([])),
        read("product_archetype_show", "Show an assembled archetype's parts.", named.clone(), serde_json::json!(["name"])),
        read("product_archetype_validate", "Validate an archetype (How + layout + cells + coherence).", named.clone(), serde_json::json!(["name"])),
        read("product_archetype_check", "Check an archetype's layout against the repository tree (§4.3).", named.clone(), serde_json::json!(["name"])),
        write("product_archetype_init", "Scaffold a new archetype under .product/archetypes/<name>/ (How contract + layout + an example cell). Returns { ok, name, written }.",
            serde_json::json!({"name": {"type": "string"}, "force": {"type": "boolean"}}), serde_json::json!(["name"])),
        // cell
        read("product_cell_show", "Show the task-type cell (.product/cell.yaml).", none.clone(), serde_json::json!([])),
        read("product_cell_validate", "Validate the cell against the What graph + How contract.", serde_json::json!({"product": {"type": "string"}}), serde_json::json!([])),
        write("product_cell_init", "Scaffold a starter task-type (cell) at .product/cell.yaml. Returns { ok, id, archetype, written }.",
            serde_json::json!({"id": {"type": "string"}, "archetype": {"type": "string"}, "file": {"type": "string"}, "force": {"type": "boolean"}}), serde_json::json!(["id"])),
        write("product_cell_dispatch", "Dispatch the cell at .product/cell.yaml into concrete §5 work units (under .product/work-units/), bound to the captured What graph. `binds` is an object {slot: value}. Returns { ok, workUnits, written, violations }.",
            serde_json::json!({"binds": {"type": "object"}, "product": {"type": "string"}}), serde_json::json!([])),
        // how
        read("product_how_show", "Show the How contract summary (.product/how-contract.yaml).", none.clone(), serde_json::json!([])),
        read("product_how_validate", "Validate the How contract against the framework shapes.", none.clone(), serde_json::json!([])),
        read("product_how_export", "Export the How contract as Turtle.", none.clone(), serde_json::json!([])),
        // work-unit
        read("product_work_unit_show", "Show the work unit (.product/work-unit.yaml).", none.clone(), serde_json::json!([])),
        read("product_work_unit_validate", "Validate the work unit against the What graph + How contract.", serde_json::json!({"product": {"type": "string"}}), serde_json::json!([])),
        write("product_work_unit_init", "Scaffold a starter §5 work unit at .product/work-unit.yaml. Returns { ok, id, written }.",
            serde_json::json!({"id": {"type": "string"}, "file": {"type": "string"}, "force": {"type": "boolean"}}), serde_json::json!(["id"])),
        // worker (capability catalog)
        read("product_worker_list", "List the worker capabilities + role bindings.", none, serde_json::json!([])),
        read("product_worker_resolve", "Resolve a role to its capability, applying escalation triggers.",
            serde_json::json!({"role": {"type": "string"}, "triggers": {"type": "array", "items": {"type": "string"}}}),
            serde_json::json!(["role"])),
    ]
}
