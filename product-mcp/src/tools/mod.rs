//! MCP tool definitions — the What/How framework-graph tool schemas.

mod decider;
mod projector;
mod primitive;
mod delivery;
mod domain;
mod legacy_pf;

use serde::Serialize;
use serde_json::Value;

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
    tools.extend(delivery::all());
    tools.extend(legacy_pf::all());
    tools
}
