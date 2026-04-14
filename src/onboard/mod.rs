//! Codebase onboarding — decision discovery from existing code (ADR-027)
//!
//! Three-phase pipeline:
//! 1. **Scan** — detect decision candidates from code patterns (heuristic + evidence validation)
//! 2. **Triage** — structured team review: confirm, reject, merge, skip
//! 3. **Seed** — convert confirmed candidates into ADR files + feature stubs

pub mod types;
pub mod scan;
mod scan_builders;
pub mod seed;
pub mod triage;
pub(crate) mod evidence;

// Re-export public API at module level for backward compatibility
pub use evidence::validate_all_evidence;
pub use scan::scan;
pub use seed::{execute_seed, plan_seed};
pub use triage::{triage_batch_confirm, triage_interactive};
pub use types::*;

#[cfg(test)]
mod tests;
