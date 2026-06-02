//! In-memory knowledge graph — construction, traversal, validation (ADR-003, ADR-012)

mod algorithms;
mod dep_validation;
pub mod full_check;
pub mod functional_spec_validation;
mod lifecycle_validation;
pub mod inference;
mod model;
pub mod observability_validation;
mod ordering;
pub mod pattern_topo;
pub mod pattern_validation;
pub mod planning_validation;
mod removal_validation;
pub mod responsibility;
mod runner_required_validation;
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
