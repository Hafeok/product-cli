//! Decision Ledger record substrate — the L0 file format with its gate.
//!
//! A version-control system for governing decisions (PRD:
//! `docs/decision-ledger-prd.md`). L0 is deliberately the format plus
//! `verify`: no graph, no RDF index, no merge, no coverage query beyond the
//! gate. `.decisions/` files in git are the append-only log and the sole
//! source of truth; everything derived is out of scope at this milestone.
//!
//! The load-bearing pieces are [`canon`] (the canonical form an acceptance
//! signs, specified in `docs/ledger-format-v1.md`) and [`verify`] (the ten
//! failure classes the CI gate polices). Both are stable surfaces other
//! implementations import from the format document, not from this crate.

#![deny(clippy::unwrap_used)]

pub mod acceptance;
pub mod allocation;
pub mod author;
pub mod blame;
pub mod canon;
pub mod changeset;
pub mod coverage;
pub mod diff;
pub mod discharge;
pub mod finding;
pub mod format;
pub mod graph;
pub mod hash;
pub mod id;
pub mod identity;
pub mod init;
pub mod mint;
pub mod render;
pub mod revision;
pub mod set;
pub mod store;
pub mod tier;
pub mod verify;
pub mod version;
pub mod whoami;

#[cfg(test)]
pub(crate) mod testkit;

/// The store directory name, sibling to `.product/` and `.ddd/` where those
/// are present (PRD §5).
pub const STORE_DIR: &str = ".decisions";
