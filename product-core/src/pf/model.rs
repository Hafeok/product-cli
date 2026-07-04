//! Typed node model for the What graph.

use serde::{Deserialize, Serialize};

use super::ids::NodeKind;

pub use super::model_data::*;
pub use super::model_product::*;
pub use super::model_ui::*;

/// A named attribute of an entity (e.g. `email: string`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
}

/// §3.1 — a region within which every term has exactly one meaning.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct BoundedContext {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glossary: Vec<String>,
}

/// §3.1 — a domain concept with identity, placed in one bounded context.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ValueObject {
    pub id: String,
    pub label: String,
    pub context: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
}

/// §3.1 — a typed link between two entities, carrying cardinality + rationale.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Invariant {
    pub id: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applies_to: Option<String>,
}

/// §3.1 — an explicit correspondence between concepts in two contexts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ContextMapping {
    pub id: String,
    pub concept_a: String,
    pub concept_b: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub rationale: String,
}

/// §3.2 — an intent that causes events; targets an aggregate.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub context: String,
    pub targets: String,
    pub emits: Vec<String>,
    /// §3.2 — the declared payload schema (name + datatype per field). The
    /// wire contract between systems; reification prefers these over
    /// scenario inference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Attribute>,
}

/// §3.2 — a past-tense fact; changes a real entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Event {
    pub id: String,
    pub label: String,
    pub context: String,
    pub changes: String,
    /// §3.2 — the declared payload schema (see [`Command::fields`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Attribute>,
}

/// §3.2 — a view; projects entities/events. §3.2 state space — `present` plus
/// any of `loading`/`empty`/`failed` it can exhibit (declared in `states`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
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
    #[serde(default)]
    pub design_systems: Vec<DesignSystem>,
    #[serde(default)]
    pub cios: Vec<Cio>,
    #[serde(default)]
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub reification_rules: Vec<ReificationRule>,
    #[serde(default)]
    pub reference_sets: Vec<ReferenceSet>,
    #[serde(default)]
    pub data_shapes: Vec<DataShape>,
    #[serde(default)]
    pub production_datasets: Vec<ProductionDataset>,
    #[serde(default)]
    pub systems: Vec<System>,
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    #[serde(default)]
    pub unreifiable_rules: Vec<UnreifiableRule>,
    #[serde(default)]
    pub products: Vec<Product>,
    #[serde(default)]
    pub journeys: Vec<Journey>,
    #[serde(default)]
    pub quality_demands: Vec<QualityDemand>,
}

impl DomainGraph {
    /// True if any node with this id already exists.
    pub fn contains(&self, id: &str) -> bool {
        self.kind_of(id).is_some()
    }

    /// The class of the node with this id, if it exists. Derived from the
    /// canonical `ids()` table so there is one place that enumerates kinds.
    pub fn kind_of(&self, id: &str) -> Option<NodeKind> {
        self.ids().into_iter().find(|(nid, _)| nid == id).map(|(_, k)| k)
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
            ("DesignSystem", self.design_systems.len()),
            ("Cio", self.cios.len()),
            ("Token", self.tokens.len()),
            ("ReificationRule", self.reification_rules.len()),
            ("ReferenceSet", self.reference_sets.len()),
            ("DataShape", self.data_shapes.len()),
            ("ProductionDataset", self.production_datasets.len()),
            ("System", self.systems.len()),
            ("Trigger", self.triggers.len()),
            ("UnreifiableRule", self.unreifiable_rules.len()),
            ("Product", self.products.len()),
            ("Journey", self.journeys.len()),
            ("QualityDemand", self.quality_demands.len()),
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
        self.design_systems.iter().for_each(|n| out.push((n.id.clone(), NodeKind::DesignSystem)));
        self.cios.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Cio)));
        self.tokens.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Token)));
        self.reification_rules.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ReificationRule)));
        self.reference_sets.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ReferenceSet)));
        self.data_shapes.iter().for_each(|n| out.push((n.id.clone(), NodeKind::DataShape)));
        self.production_datasets.iter().for_each(|n| out.push((n.id.clone(), NodeKind::ProductionDataset)));
        self.systems.iter().for_each(|n| out.push((n.id.clone(), NodeKind::System)));
        self.triggers.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Trigger)));
        self.unreifiable_rules.iter().for_each(|n| out.push((n.id.clone(), NodeKind::UnreifiableRule)));
        self.products.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Product)));
        self.journeys.iter().for_each(|n| out.push((n.id.clone(), NodeKind::Journey)));
        self.quality_demands.iter().for_each(|n| out.push((n.id.clone(), NodeKind::QualityDemand)));
        out
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
