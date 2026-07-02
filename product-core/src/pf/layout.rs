//! Repository layout model — §4.3 glob rules for what is legal where.
//!
//! Allowlist semantics with two normative guards: every rule cites the
//! principle it `enforces` (Guard 1), and prohibitions (`must_not_exist`) are
//! reserved for actively-dangerous cases and must carry rationale (Guard 2).
//! Mirrors `schema/json/layout-model.schema.json`.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};
use super::validate::Violation;

/// A `must_co_exist` completeness rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CoExist {
    pub when: String,
    pub require: Vec<String>,
}

/// One layout rule. Exactly one rule-kind key must be present.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LayoutRule {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enforces: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_exist: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub for_each: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub may_exist_here: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_co_exist: Option<CoExist>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_not_exist: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_orphans: Option<String>,
}

impl LayoutRule {
    /// Count how many rule-kind keys are set (must be exactly 1).
    fn kind_count(&self) -> usize {
        [
            self.must_exist.is_some(),
            self.may_exist_here.is_some(),
            self.must_co_exist.is_some(),
            self.must_not_exist.is_some(),
            self.no_orphans.is_some(),
        ]
        .iter()
        .filter(|b| **b)
        .count()
    }
}

/// A repository layout model (§4.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LayoutModel {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "archetype")]
    pub blueprint: Option<String>,
    #[serde(default = "default_true")]
    pub allowlist: bool,
    pub layout: Vec<LayoutRule>,
}

fn default_true() -> bool {
    true
}

impl LayoutModel {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid layout-model YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize layout-model: {}", e)))
    }

    pub fn scaffold(blueprint: &str) -> Self {
        Self {
            version: "0.1".to_string(),
            blueprint: Some(blueprint.to_string()),
            allowlist: true,
            layout: vec![LayoutRule {
                id: "no-orphans".to_string(),
                rationale: Some("every source file must have a declared, legal home".to_string()),
                enforces: vec!["provable-layout".to_string()],
                no_orphans: Some("src/**".to_string()),
                ..Default::default()
            }],
        }
    }
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

/// Validate a layout model against the §4.3 rules.
pub fn validate_layout(m: &LayoutModel) -> Vec<Violation> {
    let mut out = Vec::new();
    if m.layout.is_empty() {
        out.push(v("layout", "layout", "§4.3 A layout model must declare at least one rule."));
    }
    for r in &m.layout {
        let kinds = r.kind_count();
        if kinds != 1 {
            out.push(v(&r.id, "rule-kind",
                "§4.3 Exactly one rule-kind (must_exist | may_exist_here | must_co_exist | must_not_exist | no_orphans) must be present."));
        }
        if r.enforces.is_empty() {
            out.push(v(&r.id, "enforces",
                "§4.3 Guard 1: every layout rule must cite the principle(s) it enforces (a rule with no principle is a superstition)."));
        }
        if r.must_exist.is_some() && r.cardinality.is_none() {
            out.push(v(&r.id, "cardinality", "§4.3 A must_exist rule must declare its cardinality."));
        }
        if r.must_not_exist.is_some() && r.rationale.is_none() {
            out.push(v(&r.id, "rationale",
                "§4.3 Guard 2: a prohibition (must_not_exist) must carry rationale."));
        }
    }
    out
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
