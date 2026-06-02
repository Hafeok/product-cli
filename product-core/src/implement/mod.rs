//! Agent orchestration — implementation pipeline with verification (ADR-021)

pub mod observes_table;
pub mod pipeline;
mod runner;
pub mod runner_autofill;
pub mod verify;

// Re-export public API
pub use observes_table::{build_observes_table, inject_observes_inline, ObservesRow};
pub use pipeline::run_implement;
pub use verify::run_verify;
pub use verify::run_verify_platform;

#[cfg(test)]
mod tests;
