//! Conformance checker mirroring the framework SHACL shapes.
//!
//! Splits along the §6 line: presence/cardinality checks (non-empty definition,
//! cardinality, rationale, …) stay native here, while the load-bearing
//! cross-references (§3.1/§3.2: an entity in a real context, an event changing a
//! real entity, a command targeting an entity + emitting an event) are SPARQL
//! rules in `rules_what`, run over the Turtle projection by `sparql_rules`. The
//! exported Turtle remains verifiable against `shapes.shacl.ttl` with `pyshacl`.

use serde::Serialize;

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::rules_what::what_rules;
use super::sparql_rules::run_rules;
use super::turtle::to_turtle;

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
        check_entity(e, &mut v);
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
    for rm in &graph.read_models {
        check_read_model(rm, &mut v);
    }
    for a in &graph.attestations {
        check_attestation(a, &mut v);
    }
    v.extend(run_rules(&ui_projection(graph), what_rules()));
    v.extend(run_rules(&ui_projection(graph), super::rules_ui::ui_rules()));
    v.extend(super::rules_ui::check_state_coverage(graph));
    v
}

/// The Turtle projection used for graph rules, augmented with the closed-core
/// AIO vocabulary so the §3.2.1 AIO-only rule recognises the built-in set.
fn ui_projection(graph: &DomainGraph) -> String {
    let mut ttl = to_turtle(graph, "validate");
    ttl.push_str(&super::rules_ui::core_aio_triples());
    ttl
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
                check_entity(e, &mut v);
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
        Some(NodeKind::ReadModel) => {
            if let Some(rm) = graph.read_models.iter().find(|n| n.id == id) {
                check_read_model(rm, &mut v);
            }
        }
        Some(NodeKind::Attestation) => {
            if let Some(a) = graph.attestations.iter().find(|n| n.id == id) {
                check_attestation(a, &mut v);
            }
        }
        // Event/Command cross-references are graph rules (below); ValueObject,
        // WireframeStep, Flow have no blocking shape.
        _ => {}
    }
    // §3.1/§3.2/§3.2.1 cross-references run as SPARQL over the projection,
    // scoped to the node just built (the ADR-053 in-loop path).
    let projection = ui_projection(graph);
    let mut graph_v = run_rules(&projection, what_rules());
    graph_v.extend(run_rules(&projection, super::rules_ui::ui_rules()));
    graph_v.extend(super::rules_ui::check_state_coverage(graph));
    graph_v.retain(|x| x.focus == id);
    v.extend(graph_v);
    v
}

// --- §3.1 structure -------------------------------------------------------

fn check_context(c: &super::model::BoundedContext, v: &mut Vec<Violation>) {
    if c.label.trim().is_empty() {
        v.push(Violation::new(&c.id, "label",
            "A bounded context must be named (its ubiquitous language is authoritative)."));
    }
}

fn check_entity(e: &super::model::Entity, v: &mut Vec<Violation>) {
    if e.definition.trim().is_empty() {
        v.push(Violation::new(&e.id, "definition",
            "§3.1 An entity must carry a business-language definition."));
    }
}

fn check_attestation(a: &super::model::Attestation, v: &mut Vec<Violation>) {
    if a.date.trim().is_empty() {
        v.push(Violation::new(&a.id, "date",
            "§3.2.3 An attestation must carry a date (the frozen-boundary record)."));
    }
    if a.by.trim().is_empty() {
        v.push(Violation::new(&a.id, "by",
            "§3.2.3 An attestation must be attributed (who evaluated the criterion)."));
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

fn check_read_model(rm: &super::model::ReadModel, v: &mut Vec<Violation>) {
    if rm.projects.iter().all(|s| s.trim().is_empty()) {
        v.push(Violation::new(&rm.id, "projects",
            "§3.2 A read model must project at least one entity/event (read-model provenance: no ghost views)."));
    }
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
