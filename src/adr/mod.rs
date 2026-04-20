//! ADR domain slice — pure planning functions with thin I/O appliers.
//!
//! Mirrors the feature slice convention: each submodule has `plan_*`
//! functions returning plan structs, plus `apply_*` functions that persist
//! them via atomic batch writes.

pub mod conflicts;
pub mod create;
pub mod field_edits;
pub mod seal;
pub mod status_change;
pub mod supersede;

pub use conflicts::{check_conflicts, ConflictFinding, FindingCode};
pub use create::{apply_create, plan_create, CreatePlan};
pub use field_edits::{
    apply_domain_edit, apply_scope_change, apply_source_files_edit, plan_domain_edit,
    plan_scope_change, plan_source_files_edit, DomainEditPlan, ScopeChangePlan,
    SourceFilesEditPlan,
};
pub use seal::{
    apply_amend, apply_seal, plan_amend, plan_seal, unsealed_accepted_ids, AmendPlan, SealPlan,
};
pub use status_change::{apply_status_change, plan_status_change, StatusChangePlan};
pub use supersede::{apply_supersede, plan_supersede_add, plan_supersede_remove, SupersedePlan};

#[cfg(test)]
mod tests;
