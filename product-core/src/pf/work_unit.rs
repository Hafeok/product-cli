//! Work-unit model — the §5 SPMC unit of realisation.
//!
//! The smallest reproducible unit: one bounded transformation with a frozen,
//! declared input (Schema, Prompt, Model, Context) producing exactly one
//! artifact, carrying the rationale trace it emits. Mirrors
//! `schema/json/work-unit.schema.json`.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

/// C of SPMC — the frozen, declared input.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Context {
    pub derived_from: Vec<String>,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// The single artifact this unit produces.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Produces {
    pub artifact: String,
    /// The exact repository path the artifact lands at. Authoritative — the
    /// harness writes the worker's content here and ignores any path the worker
    /// emits, so a hallucinated location cannot misplace the file. Concrete: the
    /// cell's templated path resolved against the dispatch bindings.
    pub path: String,
}

/// The rationale trace the unit's output carries.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Trace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behaviour: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub why: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structure: Option<String>,
}

/// A work unit (SPMC, §5).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct WorkUnit {
    pub id: String,
    pub schema: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub context: Context,
    pub produces: Produces,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Trace>,
}

impl WorkUnit {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid work-unit YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize work-unit: {}", e)))
    }

    pub fn scaffold(id: &str) -> Self {
        Self {
            id: id.to_string(),
            schema: "schemas/artifact.schema.json".to_string(),
            prompt: "TODO: the single-purpose instruction (one prompt, one artifact).".to_string(),
            model: Some("small/code".to_string()),
            context: Context {
                derived_from: vec!["domain:TODO".to_string()],
                frozen: true,
                hash: None,
            },
            produces: Produces { artifact: "TODO".to_string(), path: "path/to/TODO.ext".to_string() },
            applies: vec![],
            trace: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../schema/examples/work-unit.example.yaml");

    #[test]
    fn parses_the_bundled_example() {
        let w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
        assert_eq!(w.id, "complete-task-handler");
        assert!(w.context.frozen);
        assert_eq!(w.context.derived_from.len(), 3);
        assert_eq!(w.produces.artifact, "CompleteTaskHandler.cs");
        assert_eq!(w.trace.as_ref().expect("trace").what.as_deref(), Some("Task"));
    }

    #[test]
    fn yaml_round_trips() {
        let w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
        let back = WorkUnit::from_yaml(&w.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(w, back);
    }

    #[test]
    fn scaffold_is_parseable() {
        let w = WorkUnit::scaffold("my-unit");
        let back = WorkUnit::from_yaml(&w.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(w, back);
    }
}
