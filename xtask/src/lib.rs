//! Workspace convention enforcement library.
//!
//! This crate is invoked by `cargo xtask check` (see `src/main.rs`) and also
//! exposes its types as a library so doctests can lock in compile-time
//! invariants — see [`CtxId`] for an example.

mod check_id;

pub mod checks;
pub mod conventions;
pub mod diagnostic;
pub mod drift;

pub use check_id::CtxId;
pub use checks::{Check, Registry};
pub use diagnostic::{Diagnostic, Format, Severity};
