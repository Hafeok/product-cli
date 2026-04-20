//! Feature domain slice — pure planning functions paired with thin I/O appliers.
//!
//! Each submodule exposes a `plan_*` function that is deterministic, takes
//! current state plus user input, returns a plan struct; plus an `apply_*`
//! function that is a narrow I/O wrapper with no business logic. This keeps
//! domain behaviour unit-testable without tempdirs or process::exit.

pub mod create;
pub mod domain_edit;
pub mod status_change;

pub use create::{plan_create, apply_create, CreatePlan};
pub use domain_edit::{plan_domain_edit, apply_domain_edit, DomainEditPlan};
pub use status_change::{plan_status_change, apply_status_change, StatusChangePlan};

#[cfg(test)]
mod tests;
