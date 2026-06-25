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
    for rs in &graph.reference_sets {
        super::rules_data::check_reference_set(rs, &mut v);
    }
    for s in &graph.data_shapes {
        super::rules_data::check_data_shape(s, &mut v);
    }
    for d in &graph.production_datasets {
        super::rules_data::check_dataset(d, &mut v);
    }
    for s in &graph.systems {
        check_system(s, graph, &mut v);
    }
    for f in &graph.flows {
        check_flow(f, graph, &mut v);
    }
    for t in &graph.triggers {
        check_trigger(t, graph, &mut v);
    }
    for c in &graph.contexts_of_use {
        check_context_of_use(c, &mut v);
    }
    v.extend(super::rules_data::data_cross_refs(graph));
    v.extend(run_rules(&ui_projection(graph), what_rules()));
    v.extend(run_rules(&ui_projection(graph), super::rules_ui::ui_rules()));
    v.extend(super::rules_ui::check_state_coverage(graph));
    v.extend(super::rules_ui::check_content_refs(graph));
    v.extend(super::rules_ui::check_content_coverage(graph));
    v
}

/// The Turtle projection used for graph rules, augmented with the closed-core
/// AIO vocabulary so the §3.2.1 AIO-only rule recognises the built-in set.
fn ui_projection(graph: &DomainGraph) -> String {
    let mut ttl = to_turtle(graph, "validate");
    ttl.push_str(&super::rules_ui::core_aio_triples());
    ttl
}

/// Run the native (non-cross-reference) shape that targets this node's kind.
fn check_local_shape(graph: &DomainGraph, id: &str, v: &mut Vec<Violation>) {
    match graph.kind_of(id) {
        Some(NodeKind::BoundedContext) => {
            if let Some(c) = graph.contexts.iter().find(|n| n.id == id) { check_context(c, v); }
        }
        Some(NodeKind::Entity) => {
            if let Some(e) = graph.entities.iter().find(|n| n.id == id) { check_entity(e, v); }
        }
        Some(NodeKind::Relation) => {
            if let Some(r) = graph.relations.iter().find(|n| n.id == id) { check_relation(r, v); }
        }
        Some(NodeKind::ContextMapping) => {
            if let Some(m) = graph.context_mappings.iter().find(|n| n.id == id) { check_mapping(m, v); }
        }
        Some(NodeKind::Invariant) => {
            if let Some(i) = graph.invariants.iter().find(|n| n.id == id) { check_invariant(i, v); }
        }
        Some(NodeKind::ReadModel) => {
            if let Some(rm) = graph.read_models.iter().find(|n| n.id == id) { check_read_model(rm, v); }
        }
        Some(NodeKind::Attestation) => {
            if let Some(a) = graph.attestations.iter().find(|n| n.id == id) { check_attestation(a, v); }
        }
        Some(NodeKind::ReferenceSet) => {
            if let Some(rs) = graph.reference_sets.iter().find(|n| n.id == id) { super::rules_data::check_reference_set(rs, v); }
        }
        Some(NodeKind::DataShape) => {
            if let Some(s) = graph.data_shapes.iter().find(|n| n.id == id) { super::rules_data::check_data_shape(s, v); }
        }
        Some(NodeKind::ProductionDataset) => {
            if let Some(d) = graph.production_datasets.iter().find(|n| n.id == id) { super::rules_data::check_dataset(d, v); }
        }
        Some(NodeKind::System) => {
            if let Some(s) = graph.systems.iter().find(|n| n.id == id) { check_system(s, graph, v); }
        }
        Some(NodeKind::Flow) => {
            if let Some(f) = graph.flows.iter().find(|n| n.id == id) { check_flow(f, graph, v); }
        }
        Some(NodeKind::Trigger) => {
            if let Some(t) = graph.triggers.iter().find(|n| n.id == id) { check_trigger(t, graph, v); }
        }
        Some(NodeKind::ContextOfUse) => {
            if let Some(c) = graph.contexts_of_use.iter().find(|n| n.id == id) { check_context_of_use(c, v); }
        }
        // Event/Command cross-references are graph rules (below); ValueObject,
        // WireframeStep have no blocking shape.
        _ => {}
    }
}

// --- §3.2.0 the Trigger block and its patterns ----------------------------

fn check_trigger(t: &super::model::Trigger, graph: &DomainGraph, v: &mut Vec<Violation>) {
    const SOURCES: [&str; 3] = ["user", "external", "automated"];
    if t.source.trim().is_empty() {
        v.push(Violation::new(&t.id, "source",
            "§3.2.0 A trigger must declare its source (user / external / automated)."));
    } else if !SOURCES.contains(&t.source.as_str()) {
        v.push(Violation::new(&t.id, "source",
            "§3.2.0 A trigger's source must be one of user, external, or automated."));
    }
    if t.issues.trim().is_empty() {
        v.push(Violation::new(&t.id, "issues", "§3.2.0 A trigger must issue a command."));
    } else if !graph.is_kind(&t.issues, NodeKind::Command) {
        v.push(Violation::new(&t.id, "issues",
            "§3.2.0 A trigger's issued command must resolve to a declared Command."));
    }
    // Automation pattern: an automated trigger observes a View, then acts.
    if t.source == "automated" && t.watches.is_none() {
        v.push(Violation::new(&t.id, "watches",
            "§3.2.0 An automated trigger must watch a View (read model) — the Automation pattern observes, then issues a command."));
    }
    if let Some(w) = &t.watches {
        if !graph.is_kind(w, NodeKind::ReadModel) {
            v.push(Violation::new(&t.id, "watches",
                "§3.2.0 A trigger's watched View must resolve to a declared read model."));
        }
    }
    // Translation pattern: the read side reads from exactly one source system.
    if let Some(sys) = &t.translates_from {
        if !graph.is_kind(sys, NodeKind::System) {
            v.push(Violation::new(&t.id, "translates_from",
                "§3.2.0 A Translation trigger's source system must resolve to a declared System."));
        }
    }
}

// --- §3.2.5 the system ----------------------------------------------------

fn check_system(s: &super::model::System, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if s.kind.trim().is_empty() {
        v.push(Violation::new(&s.id, "kind",
            "§3.2.5 A system must declare its kind (application/website/service/cli/…)."));
    }
    if s.purpose.trim().is_empty() {
        v.push(Violation::new(&s.id, "purpose",
            "§3.2.5 A system must state its purpose in one sentence (the ubiquitous language)."));
    }
    if let Some(root) = &s.root {
        if !graph.is_kind(root, NodeKind::ApplicationRoot) {
            v.push(Violation::new(&s.id, "root",
                "§3.2.5 A system's root must resolve to a declared ApplicationRoot."));
        }
    }
    for c in &s.target_classes {
        if !super::ids::CORE_INTERACTION_CLASSES.contains(&c.as_str()) {
            v.push(Violation::new(&s.id, "target_classes",
                "§3.2.2 A system's interaction class must be a recognised class (gui or tui — the gating context dimension)."));
        }
    }
}

// §3.2.2 — interaction class is the senior context dimension; a context of use
// declaring it must name a recognised class. Platform is an open dimension.
fn check_context_of_use(c: &super::model::ContextOfUse, v: &mut Vec<Violation>) {
    if c.dimension.as_deref() == Some("interaction-class") {
        let ok = c.value.as_deref().map(|x| super::ids::CORE_INTERACTION_CLASSES.contains(&x)).unwrap_or(false);
        if !ok {
            v.push(Violation::new(&c.id, "value",
                "§3.2.2 An interaction-class context must name a recognised class (gui or tui)."));
        }
    }
}

fn check_flow(f: &super::model::Flow, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if let Some(sys) = &f.system {
        if !graph.is_kind(sys, NodeKind::System) {
            v.push(Violation::new(&f.id, "system",
                "§3.2.5 A flow's system must resolve to a declared System (a flow belongs to exactly one)."));
        }
    }
}

/// Run only the shape(s) that target the node with this id (the in-loop path
/// every mutating tool runs against the fragment it just built).
pub fn validate_node(graph: &DomainGraph, id: &str) -> Vec<Violation> {
    let mut v = Vec::new();
    check_local_shape(graph, id, &mut v);
    // §3.1/§3.2/§3.2.1 cross-references run as SPARQL over the projection,
    // scoped to the node just built (the ADR-053 in-loop path).
    let projection = ui_projection(graph);
    let mut graph_v = run_rules(&projection, what_rules());
    graph_v.extend(run_rules(&projection, super::rules_ui::ui_rules()));
    graph_v.extend(super::rules_ui::check_state_coverage(graph));
    graph_v.extend(super::rules_ui::check_content_refs(graph));
    graph_v.extend(super::rules_ui::check_content_coverage(graph));
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
