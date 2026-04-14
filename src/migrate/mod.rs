//! Migration commands — parse monolithic PRD/ADR files into structured artifacts (ADR-017)

pub mod types;
pub mod extract;
pub mod execute;
pub(crate) mod helpers;

// Re-export public API
pub use execute::execute_plan;
pub use extract::{migrate_from_adrs, migrate_from_prd};
pub use types::*;

#[cfg(test)]
mod tests;
