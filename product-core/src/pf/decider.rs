//! Decider — the executable form of behaviour for one aggregate (§3.3).
//!
//! A Decider's signature is *derived from* the event model: it handles exactly
//! the commands targeting its aggregate, emits only events those commands
//! sanction, evolves from the events that change the aggregate, rejects via the
//! aggregate's invariants. This module derives that signature from the What
//! graph and validates an authored Decider against it — the three §3.3 drift
//! rules run as graph rules (`rules_decider`) over the combined projection.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::decider_turtle::decider_to_turtle;
use super::ids::NodeKind;
use super::model::DomainGraph;
use super::rules_decider::decider_rules;
use super::sparql_rules::run_rules;
use super::validate::Violation;

/// A §3.3 Decider for one aggregate. The signature fields are derived from the
/// event model; only decision logic (future) is authored.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Decider {
    pub id: String,
    pub decides_for: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emits: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evolves_from: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejects: Vec<String>,
}

impl Decider {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid decider YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize decider: {}", e)))
    }
}

/// Derive a Decider's full signature for `aggregate` from the What graph: the
/// commands targeting it, the events those commands emit, the events that
/// change it, and the invariants that apply to it.
pub fn derive_decider(graph: &DomainGraph, aggregate: &str) -> Result<Decider> {
    if !graph.is_kind(aggregate, NodeKind::Entity) {
        return Err(ProductError::NotFound(format!(
            "no entity '{aggregate}' in the What graph to decide for"
        )));
    }
    let handled: Vec<_> = graph.commands.iter().filter(|c| c.targets == aggregate).collect();
    let handles: Vec<String> = handled.iter().map(|c| c.id.clone()).collect();
    let mut emits: Vec<String> = Vec::new();
    for c in &handled {
        for e in &c.emits {
            if !emits.contains(e) {
                emits.push(e.clone());
            }
        }
    }
    let evolves_from: Vec<String> = graph.events.iter()
        .filter(|e| e.changes == aggregate)
        .map(|e| e.id.clone())
        .collect();
    let rejects: Vec<String> = graph.invariants.iter()
        .filter(|i| i.applies_to.as_deref() == Some(aggregate))
        .map(|i| i.id.clone())
        .collect();
    Ok(Decider {
        id: format!("{aggregate}-decider"),
        decides_for: aggregate.to_string(),
        handles,
        emits,
        evolves_from,
        rejects,
    })
}

/// Validate an authored Decider against the What graph (§3.3): presence checks
/// (native) plus the drift rules (graph SPARQL over the combined What + Decider
/// projection — no foreign commands, command coverage, output-alphabet).
pub fn validate_decider(decider: &Decider, graph: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    if decider.id.trim().is_empty() {
        out.push(v(&decider.id, "id", "§3.3 A Decider must be named."));
    }
    if decider.decides_for.trim().is_empty() {
        out.push(v(&decider.id, "decides_for", "§3.3 A Decider must decide for an aggregate entity."));
        return out;
    }
    out.extend(run_rules(&decider_to_turtle(graph, decider, "validate"), decider_rules()));
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
#[path = "decider_tests.rs"]
mod tests;
