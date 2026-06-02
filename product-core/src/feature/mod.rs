//! Feature domain slice — pure planning functions paired with thin I/O appliers.
//!
//! Each submodule exposes a `plan_*` function that is deterministic, takes
//! current state plus user input, returns a plan struct; plus an `apply_*`
//! function that is a narrow I/O wrapper with no business logic. This keeps
//! domain behaviour unit-testable without tempdirs or process::exit.

pub mod body_sections;
pub mod create;
pub mod depends_on;
pub mod domain_edit;
pub mod link;
pub mod status_change;

pub use body_sections::{parse_body_sections, BodySections};
pub use create::{plan_create, apply_create, CreatePlan};
pub use depends_on::{plan_depends_on_edit, apply_depends_on_edit, DependsOnPlan};
pub use domain_edit::{plan_domain_edit, apply_domain_edit, DomainEditPlan};
pub use link::{
    apply_link, plan_link, plan_link_with_pattern, LinkPlan, LinkReciprocation, LinkWarning,
    LinkWrite, LinkWriteKind,
};
pub use status_change::{plan_status_change, apply_status_change, StatusChangePlan};

#[cfg(test)]
mod link_tests;
#[cfg(test)]
mod tests;
