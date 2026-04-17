//! Product Request — Unified atomic write interface (FT-041, ADR-038).
//!
//! A request is a typed, versioned, validated, atomically-applied description of
//! changes to the knowledge graph. Three operation types: `create`, `change`,
//! and `create-and-change`. The full schema is documented in
//! `docs/product-request-spec.md`.

pub mod apply;
pub mod log;
pub mod parse;
pub mod types;
pub mod validate;

pub use apply::{apply_request, ApplyOptions, ApplyResult};
pub use log::append_log;
pub use parse::{parse_request, parse_request_str};
pub use types::*;
pub use validate::{validate_request, validate_against_graph, ValidationContext};
