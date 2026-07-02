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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ContractStatement {
    pub id: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforced_by: Option<String>,
}

/// §4.2 — invariant code-shaping decisions, stable across instances.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct InfrastructureContract {
    pub id: String,
    pub satisfies: String,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default)]
    pub resources: Vec<Resource>,
}

/// §4.4 — a published interface generated from the domain model.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct InterfaceContract {
    pub id: String,
    pub surface: String,
    pub standard: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<String>,
}

/// An archetype's complete How (§4).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct HowContract {
    pub archetype: String,
    /// §7.3 — the How's own semantic version (the realisation's version).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// §7.3 — the What-version this How realises (`realises What 2.1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realises_version: Option<String>,
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

/// A `how-contract.yaml` that is a pointer to another one (`ref: <relative
/// path>`), instead of an inline contract. Lets an archetype *reference* a
/// shared How — e.g. the self-hosted archetype points at the repo's canonical
/// `.product/how-contract.yaml` — so the contract has a single source of truth.
/// `deny_unknown_fields` keeps a full inline contract (which carries `archetype`
/// / `application_contract`) from parsing as a ref.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct HowRef {
    #[serde(rename = "ref")]
    target: String,
}

impl HowContract {
    /// Parse a How contract from YAML text.
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid how-contract YAML: {}", e)))
    }

    /// Load a How contract from `path`, following one `ref:` hop. A ref stub
    /// (`ref: <relative path>`) resolves against `path`'s own directory, so an
    /// archetype can point at a shared contract rather than duplicate it.
    /// Returns `Ok(None)` when `path` does not exist.
    pub fn load_opt(path: &std::path::Path) -> Result<Option<Self>> {
        Self::load_depth(path, 0).map(Some).or_else(|e| match e {
            ProductError::NotFound(_) => Ok(None),
            other => Err(other),
        })
    }

    fn load_depth(path: &std::path::Path, depth: u8) -> Result<Self> {
        if depth > 4 {
            return Err(ProductError::ConfigError(format!(
                "how-contract ref chain too deep at {}", path.display())));
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound =>
                return Err(ProductError::NotFound(format!("{}", path.display()))),
            Err(e) => return Err(ProductError::IoError(format!("{}: {}", path.display(), e))),
        };
        if let Ok(r) = serde_yaml::from_str::<HowRef>(&text) {
            let base = path.parent().unwrap_or_else(|| std::path::Path::new("."));
            return Self::load_depth(&base.join(&r.target), depth + 1);
        }
        Self::from_yaml(&text)
    }

    /// True if `id` names a Why-cascade element (decision/principle/pattern/interface).
    /// The resolution target for an architectural quality demand's `constrains` (§3.6).
    pub fn has_element(&self, id: &str) -> bool {
        self.top_decisions.iter().any(|d| d.id == id)
            || self.principles.iter().any(|p| p.id == id)
            || self.patterns.iter().any(|p| p.id == id)
            || self.interface_contracts.iter().any(|i| i.id == id)
            || self.application_contract.id == id
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

    #[test]
    fn round_trips_yaml_significant_characters() {
        // Regression for the ` #` truncation: a scalar carrying a comment
        // indicator (and `:`/leading `-`/brackets) must survive to_yaml →
        // from_yaml. A plain-style serializer would lose everything past ` #`.
        let nasty = "an assert_cmd-driven #[test] fn tc_XXX in tests: a - b [c] {d}";
        let mut c = HowContract::scaffold("special");
        c.patterns = vec![Pattern { id: "p0".to_string(), shape: nasty.to_string(), ..Default::default() }];
        let back = HowContract::from_yaml(&c.to_yaml().expect("yaml")).expect("reparse");
        assert_eq!(back.patterns[0].shape, nasty, "a ` #` scalar must not truncate to a comment");
        assert_eq!(c, back);
    }

    use proptest::prelude::*;

    proptest! {
        /// Universal serialization invariant: for ANY contract text the writer
        /// must quote whatever it needs so `parse(serialize(x)) == x`. This
        /// replaces hand-picked example strings with a property over the class —
        /// every YAML-significant character is covered, not the ones we guessed.
        #[test]
        fn any_contract_text_round_trips(
            text in proptest::collection::vec(
                proptest::sample::select("abZ09 #:-[]{}*&!|>%@\"'\\,./()".chars().collect::<Vec<char>>()),
                0..48,
            ).prop_map(|cs| cs.into_iter().collect::<String>().trim().to_string()),
        ) {
            let mut c = HowContract::scaffold("prop");
            c.patterns = vec![Pattern { id: "p0".to_string(), shape: text.clone(), ..Default::default() }];
            c.top_decisions = vec![TopDecision {
                id: "d0".to_string(),
                decision: text.clone(),
                rationale: text.clone(),
                ..Default::default()
            }];
            let back = HowContract::from_yaml(&c.to_yaml().expect("yaml")).expect("reparse");
            prop_assert_eq!(c, back);
        }
    }
}
