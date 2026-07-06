//! MCP tool definitions — the What/How framework-graph tool schemas.

mod build;
mod decider;
mod projector;
mod primitive;
mod product;
mod delivery;
mod deployable_unit;
mod design_system;
mod domain;
mod how_author;
mod legacy_pf;
mod codegen;
mod scope;

use serde::Serialize;
use serde_json::Value;

/// Generate a struct's field schema (draft-07, subschemas inlined) for use in a
/// tool's input schema. The struct is the single source of truth, so the tool
/// schema cannot drift from what the handler accepts (schema-single-source).
pub(super) fn schema_props<T: schemars::JsonSchema>() -> serde_json::Map<String, Value> {
    let generator = schemars::gen::SchemaSettings::draft07()
        .with(|s| s.inline_subschemas = true)
        .into_generator();
    let root = generator.into_root_schema_for::<T>();
    serde_json::to_value(root)
        .ok()
        .and_then(|v| v.get("properties").and_then(|p| p.as_object()).cloned())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub requires_write: bool,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Build the complete list of tool definitions
pub fn build_tool_list() -> Vec<ToolDef> {
    let mut tools = domain::all();
    tools.extend(decider::all());
    tools.extend(projector::all());
    tools.extend(primitive::all());
    tools.extend(product::all());
    tools.extend(delivery::all());
    tools.extend(deployable_unit::all());
    tools.extend(design_system::all());
    tools.extend(how_author::all());
    tools.extend(legacy_pf::all());
    tools.extend(build::all());
    tools.extend(codegen::all());
    tools.extend(scope::all());
    tools
}
