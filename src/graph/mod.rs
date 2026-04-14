//! In-memory knowledge graph — construction, traversal, validation (ADR-003, ADR-012)

mod algorithms;
mod model;
mod ordering;
mod stats;
#[cfg(test)]
mod tests;
mod types;
mod validation;

pub use model::{Edge, EdgeType, KnowledgeGraph};
pub use types::{
    FeatureNextResult, GraphStats, ImpactResult, PhaseGateStatus, PhaseGateTC,
};
