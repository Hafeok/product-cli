//! Mutating operations on a domain session.
//!
//! One function per `add_*` tool. Each builds the typed fragment, runs the
//! relevant conformance shape on it in-loop, and either commits it (returning
//! `ok: true`) or reverts it (returning `ok: false` with the framework-section
//! violations) so the model self-corrects before moving on.

use serde::Serialize;

use super::ids::{validate_id, Cardinality};
use super::model::*;
use super::session::DomainSession;
use super::validate::{validate_node, Violation};

/// The `{ ok, node, violations[] }` contract every mutating tool returns.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OpResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    pub violations: Vec<Violation>,
}

fn reject(id: &str, path: &str, message: impl Into<String>) -> OpResult {
    OpResult {
        ok: false,
        node: Some(id.to_string()),
        violations: vec![Violation {
            focus: id.to_string(),
            path: path.to_string(),
            message: message.into(),
            severity: "violation".to_string(),
        }],
    }
}

/// Reject up-front for a malformed or duplicate id (before insertion).
fn precheck(session: &DomainSession, id: &str) -> Option<OpResult> {
    if validate_id(id).is_err() {
        return Some(reject(id, "id", "An id must match ^[A-Za-z][A-Za-z0-9_-]*$ (letter first)."));
    }
    if session.graph.contains(id) {
        return Some(reject(id, "id", format!("id {:?} already exists in the graph.", id)));
    }
    None
}

/// Validate the just-inserted node; commit on success, revert on violation.
fn finish(session: &mut DomainSession, id: &str, undo: impl FnOnce(&mut DomainGraph)) -> OpResult {
    let violations = validate_node(&session.graph, id);
    if violations.is_empty() {
        session.tool_calls += 1;
        OpResult { ok: true, node: Some(id.to_string()), violations: vec![] }
    } else {
        undo(&mut session.graph);
        OpResult { ok: false, node: Some(id.to_string()), violations }
    }
}

pub fn add_bounded_context(session: &mut DomainSession, c: BoundedContext) -> OpResult {
    if let Some(r) = precheck(session, &c.id) { return r; }
    let id = c.id.clone();
    session.graph.contexts.push(c);
    finish(session, &id, |g| { g.contexts.pop(); })
}

pub fn add_entity(session: &mut DomainSession, e: Entity) -> OpResult {
    if let Some(r) = precheck(session, &e.id) { return r; }
    let id = e.id.clone();
    session.graph.entities.push(e);
    finish(session, &id, |g| { g.entities.pop(); })
}

pub fn add_value_object(session: &mut DomainSession, v: ValueObject) -> OpResult {
    if let Some(r) = precheck(session, &v.id) { return r; }
    let id = v.id.clone();
    session.graph.value_objects.push(v);
    finish(session, &id, |g| { g.value_objects.pop(); })
}

pub fn add_relation(session: &mut DomainSession, r: Relation) -> OpResult {
    if let Some(rej) = precheck(session, &r.id) { return rej; }
    if let Err(e) = Cardinality::parse(&r.cardinality) {
        return reject(&r.id, "cardinality", format!("{}", e));
    }
    let id = r.id.clone();
    session.graph.relations.push(r);
    finish(session, &id, |g| { g.relations.pop(); })
}

pub fn add_invariant(session: &mut DomainSession, i: Invariant) -> OpResult {
    if let Some(rej) = precheck(session, &i.id) { return rej; }
    let id = i.id.clone();
    session.graph.invariants.push(i);
    finish(session, &id, |g| { g.invariants.pop(); })
}

pub fn add_context_mapping(session: &mut DomainSession, m: ContextMapping) -> OpResult {
    if let Some(rej) = precheck(session, &m.id) { return rej; }
    let id = m.id.clone();
    session.graph.context_mappings.push(m);
    finish(session, &id, |g| { g.context_mappings.pop(); })
}

pub fn add_command(session: &mut DomainSession, c: Command) -> OpResult {
    if let Some(rej) = precheck(session, &c.id) { return rej; }
    let id = c.id.clone();
    session.graph.commands.push(c);
    finish(session, &id, |g| { g.commands.pop(); })
}

pub fn add_event(session: &mut DomainSession, e: Event) -> OpResult {
    if let Some(rej) = precheck(session, &e.id) { return rej; }
    let id = e.id.clone();
    session.graph.events.push(e);
    finish(session, &id, |g| { g.events.pop(); })
}

pub fn add_read_model(session: &mut DomainSession, r: ReadModel) -> OpResult {
    if let Some(rej) = precheck(session, &r.id) { return rej; }
    let id = r.id.clone();
    session.graph.read_models.push(r);
    finish(session, &id, |g| { g.read_models.pop(); })
}

pub fn add_wireframe_step(session: &mut DomainSession, w: WireframeStep) -> OpResult {
    if let Some(rej) = precheck(session, &w.id) { return rej; }
    let id = w.id.clone();
    session.graph.wireframe_steps.push(w);
    finish(session, &id, |g| { g.wireframe_steps.pop(); })
}

pub fn add_flow(session: &mut DomainSession, f: Flow) -> OpResult {
    if let Some(rej) = precheck(session, &f.id) { return rej; }
    let id = f.id.clone();
    session.graph.flows.push(f);
    finish(session, &id, |g| { g.flows.pop(); })
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
