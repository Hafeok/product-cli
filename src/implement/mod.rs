//! Agent orchestration — implementation pipeline with verification (ADR-021)

pub mod pipeline;
pub mod verify;

// Re-export public API
pub use pipeline::run_implement;
pub use verify::run_verify;

#[cfg(test)]
mod tests;
