//! Pattern domain slice — pure planning paired with thin I/O appliers (FT-070, ADR-050).
//!
//! Patterns capture reusable implementation knowledge, peer to FT/ADR/TC/DEP.
//! The slice owns scaffolding new patterns, status transitions
//! (`live ↔ deprecated`), and bidirectional `examples:` ↔ `feature.patterns:`
//! linking. Adapters in `commands/pattern.rs` and `mcp/` are thin wrappers.

pub mod create;
pub mod link;
pub mod render;
pub mod status_change;
pub mod suggest;

pub use create::{apply_create, plan_create, CreatePlan};
pub use link::{apply_link, plan_link, LinkPlan, LinkReciprocation, LinkWrite, LinkWriteKind};
pub use render::{render_list_text, render_show_text};
pub use status_change::{apply_status_change, plan_status_change, StatusChangePlan};
pub use suggest::{render_suggestions_block, suggest_patterns, Suggestion};

#[cfg(test)]
mod tests;
