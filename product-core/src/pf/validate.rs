//! Conformance checker mirroring the framework SHACL shapes.
//!
//! Each rule here corresponds one-to-one to a shape in
//! `schema/shapes/shapes.shacl.ttl` (the "What" half: §3.1 structure, §3.2
//! behaviour), carrying the same framework-section message. "Passes this
//! checker" therefore means "passes `shapes.shacl.ttl`" — the exported
//! Turtle is verifiable against the real shapes with `pyshacl`.

use serde::Serialize;

use super::ids::NodeKind;
use super::model::DomainGraph;

/// A single conformance violation, shaped like the SHACL report rows the MCP
/// contract returns in `{ ok, node, violations[] }`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Violation {
    /// The id of the node that failed (SHACL focus node).
    pub focus: String,
    /// The property path that failed (local name, e.g. `changes`).
    pub path: String,
    /// The framework-section message, e.g. "§3.2 every event must change …".
    pub message: String,
    /// Always `violation` for these shapes (all are blocking).
    pub severity: String,
}

impl Violation {
    fn new(focus: &str, path: &str, message: &str) -> Self {
        Self {
            focus: focus.to_string(),
            path: path.to_string(),
            message: message.to_string(),
            severity: "violation".to_string(),
        }
    }
}

/// Run every shape over the whole graph (the `validate` / `session_finalize`
/// path). Returns all blocking violations.
pub fn validate_graph(graph: &DomainGraph) -> Vec<Violation> {
    let mut v = Vec::new();
    for c in &graph.contexts {
        check_context(c, &mut v);
    }
    for e in &graph.entities {
        check_entity(graph, e, &mut v);
    }
    for r in &graph.relations {
        check_relation(r, &mut v);
    }
    for m in &graph.context_mappings {
        check_mapping(m, &mut v);
    }
    for i in &graph.invariants {
        check_invariant(i, &mut v);
    }
    for ev in &graph.events {
        check_event(graph, ev, &mut v);
    }
    for cmd in &graph.commands {
        check_command(graph, cmd, &mut v);
    }
    for rm in &graph.read_models {
        check_read_model(rm, &mut v);
    }
    v
}

/// Run only the shape(s) that target the node with this id (the in-loop path
/// every mutating tool runs against the fragment it just built).
pub fn validate_node(graph: &DomainGraph, id: &str) -> Vec<Violation> {
    let mut v = Vec::new();
    match graph.kind_of(id) {
        Some(NodeKind::BoundedContext) => {
            if let Some(c) = graph.contexts.iter().find(|n| n.id == id) {
                check_context(c, &mut v);
            }
        }
        Some(NodeKind::Entity) => {
            if let Some(e) = graph.entities.iter().find(|n| n.id == id) {
                check_entity(graph, e, &mut v);
            }
        }
        Some(NodeKind::Relation) => {
            if let Some(r) = graph.relations.iter().find(|n| n.id == id) {
                check_relation(r, &mut v);
            }
        }
        Some(NodeKind::ContextMapping) => {
            if let Some(m) = graph.context_mappings.iter().find(|n| n.id == id) {
                check_mapping(m, &mut v);
            }
        }
        Some(NodeKind::Invariant) => {
            if let Some(i) = graph.invariants.iter().find(|n| n.id == id) {
                check_invariant(i, &mut v);
            }
        }
        Some(NodeKind::Event) => {
            if let Some(ev) = graph.events.iter().find(|n| n.id == id) {
                check_event(graph, ev, &mut v);
            }
        }
        Some(NodeKind::Command) => {
            if let Some(cmd) = graph.commands.iter().find(|n| n.id == id) {
                check_command(graph, cmd, &mut v);
            }
        }
        Some(NodeKind::ReadModel) => {
            if let Some(rm) = graph.read_models.iter().find(|n| n.id == id) {
                check_read_model(rm, &mut v);
            }
        }
        // ValueObject, WireframeStep, Flow have no blocking shape.
        _ => {}
    }
    v
}

// --- §3.1 structure -------------------------------------------------------

fn check_context(c: &super::model::BoundedContext, v: &mut Vec<Violation>) {
    if c.label.trim().is_empty() {
        v.push(Violation::new(&c.id, "label",
            "A bounded context must be named (its ubiquitous language is authoritative)."));
    }
}

fn check_entity(graph: &DomainGraph, e: &super::model::Entity, v: &mut Vec<Violation>) {
    if e.definition.trim().is_empty() {
        v.push(Violation::new(&e.id, "definition",
            "§3.1 An entity must carry a business-language definition."));
    }
    if !graph.is_kind(&e.context, NodeKind::BoundedContext) {
        v.push(Violation::new(&e.id, "inContext",
            "§3.1 An entity must belong to exactly one bounded context (never a flat model)."));
    }
}

fn check_relation(r: &super::model::Relation, v: &mut Vec<Violation>) {
    if r.cardinality.trim().is_empty() {
        v.push(Violation::new(&r.id, "cardinality", "§3.1 A relation must declare cardinality."));
    }
    if r.rationale.trim().is_empty() {
        v.push(Violation::new(&r.id, "rationale",
            "§3.1 A relation must carry rationale (why this link, this cardinality)."));
    }
}

fn check_mapping(m: &super::model::ContextMapping, v: &mut Vec<Violation>) {
    let sides = [&m.concept_a, &m.concept_b].iter().filter(|s| !s.trim().is_empty()).count();
    if sides < 2 {
        v.push(Violation::new(&m.id, "mapsTo",
            "§3.1 A context mapping must connect two concepts (an explicit declared correspondence, never assumed)."));
    }
    if m.rationale.trim().is_empty() {
        v.push(Violation::new(&m.id, "rationale",
            "§3.1 A context mapping must state the correspondence (e.g. linked by PersonId)."));
    }
}

fn check_invariant(i: &super::model::Invariant, v: &mut Vec<Violation>) {
    if i.statement.trim().is_empty() {
        v.push(Violation::new(&i.id, "statement",
            "§3.1 An invariant must be stated as a checkable constraint."));
    }
}

// --- §3.2 behaviour -------------------------------------------------------

fn check_event(graph: &DomainGraph, ev: &super::model::Event, v: &mut Vec<Violation>) {
    if !graph.is_kind(&ev.changes, NodeKind::Entity) {
        v.push(Violation::new(&ev.id, "changes",
            "§3.2 Every event must change a real domain entity (the load-bearing relation; behaviour may not reference structure that does not exist)."));
    }
    if !graph.is_kind(&ev.context, NodeKind::BoundedContext) {
        v.push(Violation::new(&ev.id, "inContext", "§3.2 An event must live in a bounded context."));
    }
}

fn check_command(graph: &DomainGraph, cmd: &super::model::Command, v: &mut Vec<Violation>) {
    if !graph.is_kind(&cmd.targets, NodeKind::Entity) {
        v.push(Violation::new(&cmd.id, "targets",
            "§3.2 A command must target a real aggregate (entity)."));
    }
    let emits_event = cmd.emits.iter().any(|id| graph.is_kind(id, NodeKind::Event));
    if !emits_event {
        v.push(Violation::new(&cmd.id, "emits",
            "§3.2 A command must emit at least one event (command coverage)."));
    }
}

fn check_read_model(rm: &super::model::ReadModel, v: &mut Vec<Violation>) {
    if rm.projects.iter().all(|s| s.trim().is_empty()) {
        v.push(Violation::new(&rm.id, "projects",
            "§3.2 A read model must project at least one entity/event (read-model provenance: no ghost views)."));
    }
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
