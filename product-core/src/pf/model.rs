//! Typed node model for the What graph.

use serde::{Deserialize, Serialize};

use super::ids::NodeKind;

pub use super::model_ui::*;

/// A named attribute of an entity (e.g. `email: string`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
}

/// §3.1 — a region within which every term has exactly one meaning.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BoundedContext {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glossary: Vec<String>,
}

/// §3.1 — a domain concept with identity, placed in one bounded context.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Entity {
    pub id: String,
    pub label: String,
    pub context: String,
    pub definition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(default)]
    pub is_aggregate_root: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
}

/// §3.1 — a domain concept without identity (e.g. Money, Address).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValueObject {
    pub id: String,
    pub label: String,
    pub context: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
}

/// §3.1 — a typed link between two entities, carrying cardinality + rationale.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Relation {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub from: String,
    pub to: String,
    pub cardinality: String,
    pub rationale: String,
}

/// §3.1 — a rule that must always hold, stated as a checkable constraint.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Invariant {
    pub id: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applies_to: Option<String>,
}

/// §3.1 — an explicit correspondence between concepts in two contexts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContextMapping {
    pub id: String,
    pub concept_a: String,
    pub concept_b: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub rationale: String,
}

/// §3.2 — an intent that causes events; targets an aggregate.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub context: String,
    pub targets: String,
    pub emits: Vec<String>,
}

/// §3.2 — a past-tense fact; changes a real entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Event {
    pub id: String,
    pub label: String,
    pub context: String,
    pub changes: String,
}

/// §3.2 — a view; projects entities/events. §3.2 state space — `present` plus
/// any of `loading`/`empty`/`failed` it can exhibit (declared in `states`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ReadModel {
    pub id: String,
    pub label: String,
    pub projects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
}


/// The whole What graph: the typed nodes captured in a session. Ordered
/// `Vec`s preserve insertion order for stable Turtle output. Graphs are
/// workshop-sized, so linear lookup is fine.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DomainGraph {
    #[serde(default)]
    pub contexts: Vec<BoundedContext>,
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub value_objects: Vec<ValueObject>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub invariants: Vec<Invariant>,
    #[serde(default)]
    pub context_mappings: Vec<ContextMapping>,
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub read_models: Vec<ReadModel>,
    #[serde(default)]
    pub wireframe_steps: Vec<WireframeStep>,
    #[serde(default)]
    pub flows: Vec<Flow>,
    #[serde(default)]
    pub aios: Vec<Aio>,
    #[serde(default)]
    pub contexts_of_use: Vec<ContextOfUse>,
    #[serde(default)]
    pub application_roots: Vec<ApplicationRoot>,
    #[serde(default)]
    pub wcag_criteria: Vec<WcagCriterion>,
    #[serde(default)]
    pub attestations: Vec<Attestation>,
    #[serde(default)]
    pub content_stores: Vec<ContentStore>,
}

impl DomainGraph {
    /// True if any node with this id already exists.
    pub fn contains(&self, id: &str) -> bool {
        self.kind_of(id).is_some()
    }

    /// The class of the node with this id, if it exists.
    pub fn kind_of(&self, id: &str) -> Option<NodeKind> {
        if self.contexts.iter().any(|n| n.id == id) {
            Some(NodeKind::BoundedContext)
        } else if self.entities.iter().any(|n| n.id == id) {
            Some(NodeKind::Entity)
        } else if self.value_objects.iter().any(|n| n.id == id) {
            Some(NodeKind::ValueObject)
        } else if self.relations.iter().any(|n| n.id == id) {
            Some(NodeKind::Relation)
        } else if self.invariants.iter().any(|n| n.id == id) {
            Some(NodeKind::Invariant)
        } else if self.context_mappings.iter().any(|n| n.id == id) {
            Some(NodeKind::ContextMapping)
        } else if self.commands.iter().any(|n| n.id == id) {
            Some(NodeKind::Command)
        } else if self.events.iter().any(|n| n.id == id) {
            Some(NodeKind::Event)
        } else if self.read_models.iter().any(|n| n.id == id) {
            Some(NodeKind::ReadModel)
        } else if self.wireframe_steps.iter().any(|n| n.id == id) {
            Some(NodeKind::WireframeStep)
        } else if self.flows.iter().any(|n| n.id == id) {
            Some(NodeKind::Flow)
        } else if self.aios.iter().any(|n| n.id == id) {
            Some(NodeKind::Aio)
        } else if self.contexts_of_use.iter().any(|n| n.id == id) {
            Some(NodeKind::ContextOfUse)
        } else if self.application_roots.iter().any(|n| n.id == id) {
            Some(NodeKind::ApplicationRoot)
        } else if self.wcag_criteria.iter().any(|n| n.id == id) {
            Some(NodeKind::WcagCriterion)
        } else if self.attestations.iter().any(|n| n.id == id) {
            Some(NodeKind::Attestation)
        } else if self.content_stores.iter().any(|n| n.id == id) {
            Some(NodeKind::ContentStore)
        } else {
            None
        }
    }

    /// True if the node with this id exists and has the given kind.
    pub fn is_kind(&self, id: &str, kind: NodeKind) -> bool {
        self.kind_of(id) == Some(kind)
    }

    /// Counts per class, for `session_state` / `query`.
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("BoundedContext", self.contexts.len()),
            ("Entity", self.entities.len()),
            ("ValueObject", self.value_objects.len()),
            ("Relation", self.relations.len()),
            ("Invariant", self.invariants.len()),
            ("ContextMapping", self.context_mappings.len()),
            ("Command", self.commands.len()),
            ("Event", self.events.len()),
            ("ReadModel", self.read_models.len()),
            ("WireframeStep", self.wireframe_steps.len()),
            ("Flow", self.flows.len()),
            ("Aio", self.aios.len()),
            ("ContextOfUse", self.contexts_of_use.len()),
            ("ApplicationRoot", self.application_roots.len()),
            ("WcagCriterion", self.wcag_criteria.len()),
            ("Attestation", self.attestations.len()),
            ("ContentStore", self.content_stores.len()),
        ]
    }

    /// Total node count across every class.
    pub fn node_count(&self) -> usize {
        self.counts().iter().map(|(_, n)| n).sum()
    }

    /// Every node id paired with its kind, in canonical class order.
    pub fn ids(&self) -> Vec<(String, NodeKind)> {
        let mut out = Vec::new();
        self.contexts.iter().for_each(|n| out.push((n.id.clone(), NodeKind::BoundedContext)));
        self.entities.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Entity)));
        self.value_objects.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ValueObject)));
        self.relations.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Relation)));
        self.invariants.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Invariant)));
        self.context_mappings.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ContextMapping)));
        self.commands.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Command)));
        self.events.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Event)));
        self.read_models.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ReadModel)));
        self.wireframe_steps.iter().for_each(|n| out.push((n.id.clone(), NodeKind::WireframeStep)));
        self.flows.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Flow)));
        self.aios.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Aio)));
        self.contexts_of_use.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ContextOfUse)));
        self.application_roots.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ApplicationRoot)));
        self.wcag_criteria.iter().for_each(|n| out.push((n.id.clone(), NodeKind::WcagCriterion)));
        self.attestations.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Attestation)));
        self.content_stores.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ContentStore)));
        out
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
