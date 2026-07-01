//! Release — a coherent set of delivery features (§7.1).
//!
//! A release groups the deliverables that ship together. It is a partition of
//! the What (via its deliverables' features), not a free-floating milestone. The
//! `done` predicate and the "cut is closed" check (§7.2) are a separate
//! increment; this module validates membership resolves.

use serde::{Deserialize, Serialize};

use std::collections::BTreeSet;

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// A release: the set of deliverable ids that ship together.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
}

impl Release {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid release YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize release: {}", e)))
    }
}

/// Validate a release: it must contain at least one feature, and every member
/// must resolve to a known deliverable.
pub fn validate_release(r: &Release, known_deliverables: &BTreeSet<String>) -> Vec<Violation> {
    let mut out = Vec::new();
    if r.features.is_empty() {
        out.push(v(&r.id, "features", "§7.1 A release must contain at least one delivery feature."));
    }
    for f in &r.features {
        if !known_deliverables.contains(f) {
            out.push(v(&r.id, "features",
                &format!("§7.1 feature '{f}' is not a deliverable — create it with `product deliverable new`.")));
        }
    }
    out
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
#[path = "release_tests.rs"]
mod tests;
