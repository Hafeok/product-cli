//! Agent orchestration — implementation pipeline with verification (ADR-021)

pub mod pipeline;
mod runner;
pub mod verify;

// Re-export public API
pub use pipeline::run_implement;
pub use verify::run_verify;
pub use verify::run_verify_platform;

#[cfg(test)]
mod tests;
