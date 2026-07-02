//! DeployableUnit tool definitions — CLI↔MCP parity for `deployable-unit`
//! (§4/§4.2: the concrete artifact a blueprint produces for a system).

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
        read("product_deployable_unit_list", "List deployable units (§4).", serde_json::json!({}), serde_json::json!([])),
        read("product_deployable_unit_show", "Show a deployable unit's declaration.", named.clone(), serde_json::json!(["name"])),
        read("product_deployable_unit_validate", "Validate a deployable unit (§4/§4.2 — blueprint resolves, systems resolve, identity present).", named, serde_json::json!(["name"])),
        write(
            "product_deployable_unit_new",
            "Instantiate a blueprint as a concrete deployable unit: built_from a blueprint, deploys_system (§3.2.5) one or more systems, carrying a §4.2 deployment identity (domain_name / bundle_id / runtime) per environment.",
            serde_json::json!({
                "id": {"type": "string"},
                "built_from": {"type": "string", "description": "the blueprint (reusable How) this unit is built from"},
                "deploys_system": {"type": "array", "items": {"type": "string"}, "description": "system id(s) this unit deploys (monolith fan-out allowed)"},
                "environment": {"type": "string", "description": "e.g. production, staging"},
                "domain_name": {"type": "string"},
                "bundle_id": {"type": "string"},
                "runtime": {"type": "string"},
                "product": {"type": "string"},
                "force": {"type": "boolean"}
            }),
            serde_json::json!(["id", "built_from", "deploys_system"]),
        ),
    ]
}
