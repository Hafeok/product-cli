//! Interactive draft sessions (FT-052, ADR-044).
//!
//! The builder is a thin UX layer on top of the request YAML. Every draft
//! lives at `.product/requests/draft.yaml` and parses to the exact same
//! `Request` that `product request apply` accepts. There is no builder-only
//! representation — opening the draft in `$EDITOR` or piping it directly to
//! `product request apply` yields identical behaviour.

pub mod add;
pub mod add_dep;
pub mod add_helpers;
pub mod draft;
pub mod render;
pub mod status;
pub mod submit;

pub use add::{add_feature, AddAckArgs, AddAdrArgs, AddDepArgs, AddDocArgs, AddFeatureArgs, AddTargetArgs, AddTcArgs, AddedArtifact};
pub use draft::{Draft, DraftKind};
pub use render::render_status;
pub use status::{status_report, StatusReport};
pub use submit::{submit, SubmitOptions, SubmitOutcome};
