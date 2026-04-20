//! Status slice — structured project/feature summaries with text renderers.
//!
//! `summary.rs` holds pure builders that inspect a `KnowledgeGraph` and
//! produce deterministic data structures (`ProjectSummary`, `FeatureList`).
//! `render.rs` holds pure text renderers that turn those structures into
//! human-readable strings. JSON rendering is derived directly from serde
//! derives on the data structures.

pub mod render;
pub mod summary;

pub use render::{render_feature_list_text, render_project_summary_text};
pub use summary::{
    build_failing_list, build_project_summary, build_untested_list, ExitCriterionSummary,
    FeatureList, FeatureRow, GateSummary, PhaseSummary, ProjectSummary,
};

#[cfg(test)]
mod tests;
