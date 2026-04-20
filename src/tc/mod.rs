//! Test-criterion domain slice — pure planning with I/O appliers.
//!
//! Mirrors the feature/adr slice conventions: plan_* functions are pure,
//! apply_* functions are thin I/O wrappers. Unit-tested without tempdirs.

pub mod create;
pub mod runner_config;
pub mod status_change;

pub use create::{apply_create, plan_create, CreatePlan};
pub use runner_config::{apply_runner_config, plan_runner_config, RunnerConfigPlan};
pub use status_change::{apply_status_change, plan_status_change, StatusChangePlan};

#[cfg(test)]
mod tests;
