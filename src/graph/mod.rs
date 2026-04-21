//! In-memory knowledge graph — construction, traversal, validation (ADR-003, ADR-012)

mod algorithms;
mod dep_validation;
mod lifecycle_validation;
pub mod inference;
mod model;
mod ordering;
pub mod planning_validation;
mod removal_validation;
pub mod responsibility;
mod stats;
#[cfg(test)]
mod tests;
mod types;
pub(crate) mod validation;
mod validation_helpers;

pub use model::{Edge, EdgeType, KnowledgeGraph};
pub use types::{
    FeatureNextResult, GraphStats, ImpactResult, PhaseGateStatus, PhaseGateTC,
};
