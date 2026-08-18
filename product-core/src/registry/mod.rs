//! Ground-registry instance generation from the versioned registry template.
//!
//! The generator mints a registry instance: it renders the template's files
//! with typed parameters, gates the result, writes the tree, records the birth
//! provenance in the first commit. Generation is local — publishing a tree to a
//! remote is a separate act that lives outside this module (see `apply`).
//!
//! The slice is pure apart from [`apply`], which performs the writes.

pub mod apply;
pub mod check;
pub mod params;
pub mod plan;
pub mod substitute;
pub mod template;
pub mod verify;

pub use apply::{apply_generation, GenerationReport};
pub use check::{check_instance, evaluate, CheckReport, Finding, FindingKind};
pub use params::RegistryParams;
pub use plan::{plan_generation, GenerationPlan, PlannedFile};
pub use substitute::{render, Site, Token};
pub use template::{BASE_IRI_TOKEN, HOST_SENTINEL, TEMPLATE, TEMPLATE_VERSION};
