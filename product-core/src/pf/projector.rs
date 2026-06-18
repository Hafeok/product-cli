//! Projector — the executable form of a read model (§3.4).
//!
//! The read-model peer of the Decider: where a Decider formalizes `decide`, a
//! Projector formalizes `project` (the `evolve` half) — how events fold into a
//! read model. Its signature is *derived from* the event model: it folds exactly
//! the events the read model projects (directly, or by changing a projected
//! entity), over exactly the entities those events change; only the fold logic is
//! authored. A read model's `projects` link names *which* events feed the view;
//! the Projector says *how*. This module derives that signature and validates an
//! authored Projector against it via the three §3.4 drift rules.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::model::DomainGraph;
use super::projector_logic::{ProjectorLogic, ProjectorScenario};
use super::validate::Violation;

/// A §3.4 Projector for one read model. The signature fields are derived from the
/// event model; only the fold logic is authored.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Projector {
    pub id: String,
    pub projects_for: String,
    /// The events this projector folds — derived: the events the read model
    /// projects, plus the events that change an entity it projects (§3.4 `folds`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folds: Vec<String>,
    /// The entities the folded events change — the read model's state footprint.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub over: Vec<String>,
    /// The one authored part (§3.4): the fold. Optional — a freshly derived
    /// Projector has only its signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic: Option<ProjectorLogic>,
    /// Projection scenarios — the oracle simulated before realisation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenarios: Vec<ProjectorScenario>,
}

impl Projector {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid projector YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize projector: {}", e)))
    }
}

/// The set of events a read model is fed by: the events it projects directly,
/// plus every event that changes an entity it projects (§3.4 — projecting an
/// entity implicitly draws in the events that change it). This is the single
/// set the no-foreign and coverage rules are both stated against, which is what
/// reconciles "folds exactly the events the read model projects" with "must fold
/// every event that changes an entity the read model projects".
fn projected_event_set(graph: &DomainGraph, read_model: &str) -> Vec<String> {
    let Some(rm) = graph.read_models.iter().find(|r| r.id == read_model) else {
        return Vec::new();
    };
    let is_event = |id: &str| graph.events.iter().any(|e| e.id == id);
    let projected_entities: Vec<&String> = rm.projects.iter().filter(|p| graph.entities.iter().any(|e| &e.id == *p)).collect();
    let mut out: Vec<String> = rm.projects.iter().filter(|p| is_event(p)).cloned().collect();
    for e in &graph.events {
        if projected_entities.iter().any(|pe| **pe == e.changes) && !out.contains(&e.id) {
            out.push(e.id.clone());
        }
    }
    out
}

/// Derive a Projector's full signature for `read_model` from the What graph: the
/// events that feed it (directly projected or changing a projected entity), and
/// the entities those events change.
pub fn derive_projector(graph: &DomainGraph, read_model: &str) -> Result<Projector> {
    if !graph.read_models.iter().any(|r| r.id == read_model) {
        return Err(ProductError::NotFound(format!(
            "no read model '{read_model}' in the What graph to project"
        )));
    }
    let folds = projected_event_set(graph, read_model);
    let mut over: Vec<String> = Vec::new();
    for fid in &folds {
        if let Some(ev) = graph.events.iter().find(|e| &e.id == fid) {
            if !over.contains(&ev.changes) {
                over.push(ev.changes.clone());
            }
        }
    }
    Ok(Projector {
        id: format!("{read_model}-projector"),
        projects_for: read_model.to_string(),
        folds,
        over,
        logic: None,
        scenarios: Vec::new(),
    })
}

/// Validate an authored Projector against the What graph (§3.4 drift rules):
/// it must project for a real read model, fold only events that feed it (no
/// foreign events), and fold every event that feeds it (coverage).
pub fn validate_projector(projector: &Projector, graph: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    if projector.id.trim().is_empty() {
        out.push(v(&projector.id, "id", "§3.4 A Projector must be named."));
    }
    if projector.projects_for.trim().is_empty() {
        out.push(v(&projector.id, "projects_for", "§3.4 A Projector must project for a read model."));
        return out;
    }
    if !graph.read_models.iter().any(|r| r.id == projector.projects_for) {
        out.push(v(&projector.id, "projects_for",
            &format!("§3.4 projects_for '{}' is not a read model in the What graph", projector.projects_for)));
        return out;
    }
    let fed = projected_event_set(graph, &projector.projects_for);
    // No foreign events — it may fold only events the read model is fed by.
    for f in &projector.folds {
        if !fed.contains(f) {
            out.push(v(&projector.id, "folds",
                &format!("§3.4 folds event '{f}', which the read model '{}' is not fed by", projector.projects_for)));
        }
    }
    // Event coverage — it must fold every event the read model is fed by.
    for e in &fed {
        if !projector.folds.contains(e) {
            out.push(v(&projector.id, "folds",
                &format!("§3.4 read model '{}' is fed by event '{e}' but the projector does not fold it", projector.projects_for)));
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
#[path = "projector_tests.rs"]
mod tests;
