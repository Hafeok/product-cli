//! The specification flow's act-time record substrate.
//!
//! `implement` opens a record; only a principal closes one. The two acts are
//! separated by a process boundary, so the record is a file on disk rather
//! than a held continuation — see `docs/spec-flow-act-record-v1.md`, which is
//! normative for everything in this crate.

pub mod check;
pub mod closure;
pub mod digest;
pub mod record;
pub mod store;

pub use closure::{Closure, ClosureKind};
pub use record::ActRecord;
