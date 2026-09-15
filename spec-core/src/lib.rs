//! The specification flow's act-time record substrate.
//!
//! `implement` opens a record; only a principal closes one. The two acts are
//! separated by a process boundary, so the record is a file on disk rather
//! than a held continuation — see `docs/spec-flow-store-v1.md`, which is
//! normative for everything in this crate.

pub mod act;
pub mod check;
pub mod closure;
pub mod digest;
pub mod gate;
pub mod inventory;
pub mod map;
pub mod metrics;
pub mod policy;
pub mod policy_check;
pub mod ratify;
pub mod record;
pub mod signing;
pub mod store;

pub use act::{Act, Rejection};
pub use closure::{Closure, ClosureKind};
pub use gate::SpecStore;
pub use inventory::Inventory;
pub use policy::Policy;
pub use record::ActRecord;
