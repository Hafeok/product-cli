//! Verification — TC runner execution, status updates, prerequisite checking,
//! and the unified six-stage pipeline (FT-044, ADR-040).

pub mod pipeline;

pub use crate::implement::run_verify;
pub use pipeline::{run_all, render_json, render_pretty, PipelineResult, PipelineScope};
