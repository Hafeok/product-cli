//! Delivery feature — one slice plus its acceptance (§7.1).
//!
//! The framework's delivery "feature" (named `deliverable` here, since
//! `product feature` owns the legacy FT-XXX graph): it references exactly one
//! slice (the event-model section it ships) and carries the acceptance criteria
//! agreed for it. It restates no behaviour — the behaviour lives in the slice.

use serde::{Deserialize, Serialize};

use std::collections::BTreeSet;

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// One agreed acceptance criterion for a deliverable. `status` is the recorded
/// verdict — `pending` until explicitly marked `passing`/`failing` (§7.2: done
/// is a predicate over recorded verifications, not a judgement).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub statement: String,
    #[serde(default = "pending")]
    pub status: String,
    /// How to check this criterion: `cargo-test` (runs `cargo test <args>`) or
    /// `shell` (runs `sh -c <args>`). Absent → the criterion is judged manually
    /// and `build --verify` leaves its status untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<String>,
    /// Arguments for the runner — a test filter for `cargo-test`, a command line
    /// for `shell`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_args: Option<String>,
}

fn pending() -> String {
    "pending".to_string()
}

/// A delivery feature: a pointer to one slice plus its acceptance criteria.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Deliverable {
    pub id: String,
    /// The single slice this deliverable ships.
    pub slice: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance: Vec<AcceptanceCriterion>,
}

impl Deliverable {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid deliverable YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize deliverable: {}", e)))
    }
}

/// Validate a deliverable: it must name a slice that resolves (against the set
/// of known slice ids), and each acceptance criterion must be a checkable
/// statement.
pub fn validate_deliverable(d: &Deliverable, known_slices: &BTreeSet<String>) -> Vec<Violation> {
    let mut out = Vec::new();
    if d.slice.trim().is_empty() {
        out.push(v(&d.id, "slice", "§7.1 A deliverable must point at exactly one slice."));
    } else if !known_slices.contains(&d.slice) {
        out.push(v(&d.id, "slice",
            &format!("§7.1 slice '{}' is not a saved slice — create it with `product slice new`.", d.slice)));
    }
    for a in &d.acceptance {
        if a.statement.trim().is_empty() {
            out.push(v(&a.id, "acceptance",
                "§7.2 An acceptance criterion must be a checkable statement (done is a predicate, not a judgement)."));
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
#[path = "deliverable_tests.rs"]
mod tests;
