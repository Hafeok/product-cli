//! How-contract model — an archetype's §4 architecture (the Why plus contracts).
//!
//! The file-based authoring surface for the How layer: the Why cascade (top
//! decisions → principles → patterns), the application + infrastructure
//! contracts, the repository layout reference, and interface contracts.
//! Mirrors `schema/json/how-contract.schema.json`; authored as YAML, projected
//! into the graph, and validated against `schema/shapes/how.shacl.ttl`.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

/// §4.1 — a foundational choice carrying rationale.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TopDecision {
    pub id: String,
    pub decision: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applies_when: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub does_not_apply_when: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enforced_by: Vec<String>,
}

/// §4.1 — a rule a top decision licenses, stated checkably.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Principle {
    pub id: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub licensed_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub realized_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enforced_by: Vec<String>,
}

/// §4.1 — a concrete shape that realises principles; what a work unit emits.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Pattern {
    pub id: String,
    pub shape: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub realizes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enforced_by: Vec<String>,
}

/// §4.2 — one checkable statement of the application contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContractStatement {
    pub id: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforced_by: Option<String>,
}

/// §4.2 — invariant code-shaping decisions, stable across instances.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ApplicationContract {
    pub id: String,
    pub language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layering: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_organization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistence_model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_cutting: Vec<String>,
    #[serde(default)]
    pub statements: Vec<ContractStatement>,
}

/// §4.2 — one concrete infrastructure resource choice.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Resource {
    pub id: String,
    pub kind: String,
    pub choice: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satisfies_statement: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

/// §4.2 — concrete runtime choices; frozen; must satisfy the app contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InfrastructureContract {
    pub id: String,
    pub satisfies: String,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default)]
    pub resources: Vec<Resource>,
}

/// §4.4 — a published interface generated from the domain model.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InterfaceContract {
    pub id: String,
    pub surface: String,
    pub standard: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<String>,
}

/// An archetype's complete How (§4).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HowContract {
    pub archetype: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub top_decisions: Vec<TopDecision>,
    #[serde(default)]
    pub principles: Vec<Principle>,
    #[serde(default)]
    pub patterns: Vec<Pattern>,
    pub application_contract: ApplicationContract,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infrastructure_contract: Option<InfrastructureContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_model: Option<String>,
    #[serde(default)]
    pub interface_contracts: Vec<InterfaceContract>,
}

impl HowContract {
    /// Parse a How contract from YAML text.
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid how-contract YAML: {}", e)))
    }

    /// Serialize the contract back to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize how-contract: {}", e)))
    }

    /// A scaffold contract for `product how init`.
    pub fn scaffold(archetype: &str) -> Self {
        Self {
            archetype: archetype.to_string(),
            version: Some("0.1".to_string()),
            application_contract: ApplicationContract {
                id: format!("{archetype}-app-contract"),
                language: "TODO".to_string(),
                statements: vec![ContractStatement {
                    id: "example-statement".to_string(),
                    statement: "TODO: a checkable invariant a verification can confirm.".to_string(),
                    enforced_by: None,
                }],
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../schema/examples/how-contract.example.yaml");

    #[test]
    fn parses_the_bundled_example() {
        let c = HowContract::from_yaml(EXAMPLE).expect("parse");
        assert_eq!(c.archetype, "example-rest-api");
        assert_eq!(c.top_decisions.len(), 2);
        assert_eq!(c.principles.len(), 3);
        assert_eq!(c.patterns.len(), 3);
        assert_eq!(c.application_contract.language, "C#");
        assert_eq!(c.application_contract.statements.len(), 2);
        let infra = c.infrastructure_contract.as_ref().expect("infra");
        assert!(infra.frozen);
        assert_eq!(infra.satisfies, "rest-api-app-contract");
        assert_eq!(c.interface_contracts[0].standard, "OpenAPI");
    }

    #[test]
    fn yaml_round_trips() {
        let c = HowContract::from_yaml(EXAMPLE).expect("parse");
        let back = HowContract::from_yaml(&c.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(c, back);
    }

    #[test]
    fn scaffold_is_parseable() {
        let c = HowContract::scaffold("rest-api");
        assert_eq!(c.archetype, "rest-api");
        let back = HowContract::from_yaml(&c.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(c, back);
    }
}
