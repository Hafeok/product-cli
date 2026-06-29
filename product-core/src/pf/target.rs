//! Target version — a declared future partition of feature-slices (§7.3).
//!
//! A target names a What-version goal as a set of deliverables (feature-slices),
//! some not yet realised. Direction is the computed gap: the unrealised members.
//! It is a query over the graph against a declared target, never roadmap prose —
//! a goal that cannot be written as a named set of slices is not yet specified.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// A target version: the set of deliverable ids (feature-slices) that constitute
/// a future What-version, some of which may not be realised yet.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    /// §7.3 — the What-version this target constitutes (e.g. `2.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// §7.3 — the feature-slices (deliverable ids) in this target's partition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub in_target: Vec<String>,
}

impl Target {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid target YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize target: {}", e)))
    }
}

/// The computed gap toward a target: which members are not yet realised.
#[derive(Debug, Clone, PartialEq)]
pub struct Direction {
    pub version: Option<String>,
    pub total: usize,
    /// The members of the partition that are not `feature_done` (§7.2).
    pub unrealised: Vec<String>,
}

impl Direction {
    /// `1 − |distance| / |partition|`, computed continuously (1.0 when empty).
    pub fn progress(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        1.0 - self.unrealised.len() as f64 / self.total as f64
    }
}

/// `distance(target) = { slice ∈ target.partition : not feature_done(slice) }`.
/// `done` maps each member to its computed `feature_done` verdict; a member
/// absent from the map (no deliverable, never computed) counts as unrealised.
pub fn direction(t: &Target, done: &BTreeMap<String, bool>) -> Direction {
    let unrealised = t
        .in_target
        .iter()
        .filter(|m| !done.get(*m).copied().unwrap_or(false))
        .cloned()
        .collect();
    Direction { version: t.version.clone(), total: t.in_target.len(), unrealised }
}

/// Validate a target: it must name at least one feature-slice, and every member
/// must resolve to a known deliverable.
pub fn validate_target(t: &Target, known_deliverables: &BTreeSet<String>) -> Vec<Violation> {
    let mut out = Vec::new();
    if t.in_target.is_empty() {
        out.push(v(&t.id, "in_target",
            "§7.3 A target version must name at least one feature-slice — a goal that cannot be written as a named set of slices is not yet specified."));
    }
    for m in &t.in_target {
        if !known_deliverables.contains(m) {
            out.push(v(&t.id, "in_target",
                &format!("§7.3 member '{m}' is not a deliverable — create it with `product deliverable new`.")));
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
mod tests {
    use super::*;

    fn t() -> Target {
        Target { id: "v2".into(), version: Some("2.0".into()),
            in_target: vec!["a".into(), "b".into(), "c".into()] }
    }

    #[test]
    fn direction_is_the_unrealised_members() {
        let done = BTreeMap::from([("a".to_string(), true), ("b".to_string(), false)]);
        let d = direction(&t(), &done);
        // b is not done; c is absent → both unrealised.
        assert_eq!(d.unrealised, vec!["b".to_string(), "c".to_string()]);
        assert_eq!(d.total, 3);
        assert!((d.progress() - (1.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn full_partition_done_is_complete() {
        let done = BTreeMap::from([("a".into(), true), ("b".into(), true), ("c".into(), true)]);
        let d = direction(&t(), &done);
        assert!(d.unrealised.is_empty());
        assert_eq!(d.progress(), 1.0);
    }

    #[test]
    fn an_empty_target_must_name_a_slice() {
        let empty = Target { id: "x".into(), version: None, in_target: vec![] };
        assert!(!validate_target(&empty, &BTreeSet::new()).is_empty());
    }
}
