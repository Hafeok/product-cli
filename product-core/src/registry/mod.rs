//! Ground-registry instance generation from the versioned registry template.
//!
//! The generator mints a registry instance: it renders the template's files
//! with typed parameters, gates the result, writes the tree, records the birth
//! provenance in the first commit. Generation is local — publishing a tree to a
//! remote is a separate act that lives outside this module (see `apply`).
//!
//! The slice is pure apart from the apply step, which performs the writes.
//!
//! The public surface is the re-export list below, deliberately: the modules
//! are crate-internal, so a caller depends on the acts (plan, apply, check) and
//! never on how the template is rendered or how the gate is spelled.

pub(crate) mod apply;
pub(crate) mod check;
pub(crate) mod params;
pub(crate) mod plan;
pub(crate) mod substitute;
pub(crate) mod template;
pub(crate) mod verify;

pub use apply::{apply_generation, GenerationReport};
pub use check::{check_instance, evaluate, CheckReport, Finding, FindingKind};
pub use params::RegistryParams;
pub use plan::{plan_generation, GenerationPlan, PlannedFile};
pub use substitute::Site;
pub use template::TEMPLATE_VERSION;
