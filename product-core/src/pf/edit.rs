//! Create, update, delete operations driven by JSON field maps.
//!
//! The CLI (`product domain new/edit/rm`) needs CRUD over every node kind
//! without 11 typed code paths. These functions build or patch a node by
//! merging a field map into a typed default (create) or the existing node
//! (edit), then run the same in-loop conformance checker as the MCP `add_*`
//! tools — so a CLI edit can never leave a non-conformant fragment committed.

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};

use super::ids::{validate_id, NodeKind};
use super::model::*;
use super::ops::OpResult;
use super::session::DomainSession;
use super::validate::{validate_node, Violation};

fn ok(id: &str) -> OpResult {
    OpResult { ok: true, node: Some(id.to_string()), violations: vec![] }
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

/// Create a node of `kind` with `id`, taking remaining fields from `fields`.
pub fn create(session: &mut DomainSession, kind: NodeKind, id: &str, fields: &Map<String, Value>) -> OpResult {
    if validate_id(id).is_err() {
        return reject(id, "id", "An id must match ^[A-Za-z][A-Za-z0-9_-]*$ (letter first).");
    }
    if session.graph.contains(id) {
        return reject(id, "id", format!("id {:?} already exists in the graph.", id));
    }
    let snapshot = session.graph.clone();
    if let Err(e) = insert(&mut session.graph, kind, id, fields) {
        return reject(id, "field", e);
    }
    commit_or_revert(session, id, snapshot)
}

/// Patch an existing node's fields. `id` selects the node; its kind is fixed.
pub fn edit(session: &mut DomainSession, id: &str, fields: &Map<String, Value>) -> OpResult {
    let Some(kind) = session.graph.kind_of(id) else {
        return reject(id, "id", format!("no node with id {:?} in the graph.", id));
    };
    let snapshot = session.graph.clone();
    if let Err(e) = patch(&mut session.graph, kind, id, fields) {
        session.graph = snapshot;
        return reject(id, "field", e);
    }
    commit_or_revert(session, id, snapshot)
}

/// Delete a node by id. Returns `ok: false` if it does not exist.
pub fn remove(session: &mut DomainSession, id: &str) -> OpResult {
    let g = &mut session.graph;
    let before = g.node_count();
    g.contexts.retain(|n| n.id != id);
    g.entities.retain(|n| n.id != id);
    g.value_objects.retain(|n| n.id != id);
    g.relations.retain(|n| n.id != id);
    g.invariants.retain(|n| n.id != id);
    g.context_mappings.retain(|n| n.id != id);
    g.commands.retain(|n| n.id != id);
    g.events.retain(|n| n.id != id);
    g.read_models.retain(|n| n.id != id);
    g.wireframe_steps.retain(|n| n.id != id);
    g.flows.retain(|n| n.id != id);
    if g.node_count() < before {
        session.tool_calls += 1;
        ok(id)
    } else {
        reject(id, "id", format!("no node with id {:?} in the graph.", id))
    }
}

/// Validate the just-changed node; keep on success, restore snapshot on failure.
fn commit_or_revert(session: &mut DomainSession, id: &str, snapshot: DomainGraph) -> OpResult {
    let violations = validate_node(&session.graph, id);
    if violations.is_empty() {
        session.tool_calls += 1;
        ok(id)
    } else {
        session.graph = snapshot;
        OpResult { ok: false, node: Some(id.to_string()), violations }
    }
}

/// Build a fresh node from `id` + `fields` and push it onto its vec.
fn insert(g: &mut DomainGraph, kind: NodeKind, id: &str, fields: &Map<String, Value>) -> Result<(), String> {
    match kind {
        NodeKind::BoundedContext => g.contexts.push(build(id, fields)?),
        NodeKind::Entity => g.entities.push(build(id, fields)?),
        NodeKind::ValueObject => g.value_objects.push(build(id, fields)?),
        NodeKind::Relation => g.relations.push(build(id, fields)?),
        NodeKind::Invariant => g.invariants.push(build(id, fields)?),
        NodeKind::ContextMapping => g.context_mappings.push(build(id, fields)?),
        NodeKind::Command => g.commands.push(build(id, fields)?),
        NodeKind::Event => g.events.push(build(id, fields)?),
        NodeKind::ReadModel => g.read_models.push(build(id, fields)?),
        NodeKind::WireframeStep => g.wireframe_steps.push(build(id, fields)?),
        NodeKind::Flow => g.flows.push(build(id, fields)?),
    }
    Ok(())
}

/// Patch the node with `id` in place by merging `fields` into it.
fn patch(g: &mut DomainGraph, kind: NodeKind, id: &str, fields: &Map<String, Value>) -> Result<(), String> {
    match kind {
        NodeKind::BoundedContext => patch_at(&mut g.contexts, id, fields),
        NodeKind::Entity => patch_at(&mut g.entities, id, fields),
        NodeKind::ValueObject => patch_at(&mut g.value_objects, id, fields),
        NodeKind::Relation => patch_at(&mut g.relations, id, fields),
        NodeKind::Invariant => patch_at(&mut g.invariants, id, fields),
        NodeKind::ContextMapping => patch_at(&mut g.context_mappings, id, fields),
        NodeKind::Command => patch_at(&mut g.commands, id, fields),
        NodeKind::Event => patch_at(&mut g.events, id, fields),
        NodeKind::ReadModel => patch_at(&mut g.read_models, id, fields),
        NodeKind::WireframeStep => patch_at(&mut g.wireframe_steps, id, fields),
        NodeKind::Flow => patch_at(&mut g.flows, id, fields),
    }
}

/// Default-construct `T`, force its id, then merge `fields`.
fn build<T: Default + Serialize + DeserializeOwned>(id: &str, fields: &Map<String, Value>) -> Result<T, String> {
    let mut value = serde_json::to_value(T::default()).map_err(|e| e.to_string())?;
    set_field(&mut value, "id", Value::String(id.to_string()));
    apply_fields(&mut value, fields);
    serde_json::from_value(value).map_err(|e| format!("invalid field value: {}", e))
}

/// Find the node with `id` in `vec`, merge `fields`, and write it back.
fn patch_at<T: Serialize + DeserializeOwned + HasId>(vec: &mut [T], id: &str, fields: &Map<String, Value>) -> Result<(), String> {
    let node = vec.iter_mut().find(|n| n.node_id() == id).ok_or("node vanished")?;
    let mut value = serde_json::to_value(&*node).map_err(|e| e.to_string())?;
    apply_fields(&mut value, fields);
    *node = serde_json::from_value(value).map_err(|e| format!("invalid field value: {}", e))?;
    Ok(())
}

/// Merge `fields` into an object value, ignoring any attempt to change `id`.
fn apply_fields(value: &mut Value, fields: &Map<String, Value>) {
    for (key, val) in fields {
        if key == "id" {
            continue;
        }
        set_field(value, key, val.clone());
    }
}

fn set_field(value: &mut Value, key: &str, val: Value) {
    if let Value::Object(map) = value {
        map.insert(key.to_string(), val);
    }
}

/// Minimal id accessor so `patch_at` can find a node generically.
trait HasId {
    fn node_id(&self) -> &str;
}
macro_rules! has_id {
    ($($t:ty),+ $(,)?) => { $(impl HasId for $t { fn node_id(&self) -> &str { &self.id } })+ };
}
has_id!(BoundedContext, Entity, ValueObject, Relation, Invariant, ContextMapping, Command, Event, ReadModel, WireframeStep, Flow);

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
