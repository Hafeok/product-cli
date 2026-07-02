//! Task-type definition model — the §5 dual-read cell container.
//!
//! A task type (e.g. `add-crud-resource`) is one schema read three ways:
//! dispatch input, capture form, acceptance criteria. It declares `slots`
//! (each dual-read), the `cells` (SPMC work units) it dispatches, and the
//! `audits` that back every slot. Cells draw their frozen input from the
//! captured What graph (`domain:…`) and the How contract's patterns
//! (`applies`). Mirrors `schema/json/task-type-definition.schema.json`.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

/// A dual-read slot: simultaneously a dispatch input, a capture question, and
/// an acceptance check.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Slot {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub dispatch: String,
    pub capture: String,
    pub audit: String,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}
fn is_true(b: &bool) -> bool {
    *b
}

/// A work unit (SPMC) the task type dispatches.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Cell {
    pub id: String,
    pub artifact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub derived_from: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies: Vec<String>,
    /// The repository path this cell's artifact lands at — a *template* that may
    /// reference the task-type's slots as `<slot>` (e.g. `src/pf/<concept>.rs`),
    /// resolved to a concrete path against the dispatch bindings. This is what
    /// keeps a cell reusable across features: the pattern names a path shape, not
    /// a literal file. Required unless `edits` names the target instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// When set, this cell *edits* an existing file at this (templated) path
    /// rather than producing a new artifact. The resolved path flows to the work
    /// unit's `produces.path`, and `build` injects the file's current content so
    /// the worker returns a precise edit (§5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edits: Option<String>,
}

/// A verification that makes the definition true; names what it protects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Audit {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub checks: String,
    pub protects: String,
}

/// The shape of the discovery-record diff a session of this type produces.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Deltas {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behaviour: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infra: Option<String>,
}

/// A task-type definition (§5).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TaskType {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "archetype")]
    pub blueprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    pub applies_when: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub does_not_apply_when: Option<String>,
    #[serde(default)]
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub cells: Vec<Cell>,
    #[serde(default)]
    pub audits: Vec<Audit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deltas: Option<Deltas>,
}

impl TaskType {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid task-type YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize task-type: {}", e)))
    }

    /// A scaffold task type for `product cell init`.
    pub fn scaffold(id: &str, blueprint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            blueprint: Some(blueprint.to_string()),
            applies_when: "TODO: when this task type applies.".to_string(),
            slots: vec![Slot {
                name: "entity".to_string(),
                kind: Some("domain".to_string()),
                dispatch: "name the aggregate".to_string(),
                capture: "which entity?".to_string(),
                audit: "entity exists in the domain model".to_string(),
                required: true,
            }],
            cells: vec![Cell {
                id: "handler".to_string(),
                artifact: "TODO artifact".to_string(),
                model: Some("code".to_string()),
                derived_from: vec!["domain:entity".to_string()],
                applies: vec![],
                path: Some("path/to/<entity>.ext".to_string()),
                edits: None,
            }],
            audits: vec![Audit {
                id: "entity-exists".to_string(),
                kind: Some("domain-conformance".to_string()),
                checks: "the entity slot resolves to a real domain entity".to_string(),
                protects: "entity slot".to_string(),
            }],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../schema/examples/task-type-definition.example.yaml");

    #[test]
    fn parses_the_bundled_example() {
        let t = TaskType::from_yaml(EXAMPLE).expect("parse");
        assert_eq!(t.id, "add-crud-resource");
        assert_eq!(t.blueprint.as_deref(), Some("example-rest-api"));
        assert_eq!(t.slots.len(), 5);
        assert_eq!(t.cells.len(), 4);
        assert_eq!(t.audits.len(), 3);
        // a cell draws from the domain + a contract
        assert!(t.cells.iter().any(|c| c.derived_from.iter().any(|d| d.starts_with("domain:"))));
    }

    #[test]
    fn yaml_round_trips() {
        let t = TaskType::from_yaml(EXAMPLE).expect("parse");
        let back = TaskType::from_yaml(&t.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(t, back);
    }

    #[test]
    fn scaffold_is_parseable() {
        let t = TaskType::scaffold("add-crud-resource", "rest-api");
        let back = TaskType::from_yaml(&t.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(t, back);
    }
}
