//! LSP-backed language intelligence for the ddd governance surface.
//!
//! Implements PRD M3 (`docs/ddd-cli-prd.md` §5.1): the tool is an LSP
//! *client* hosting per-language servers as stdio children — lazy start,
//! health check, restart on crash. The LSP protocol is the only seam to
//! language intelligence; nothing here links Roslyn or Bicep APIs. All
//! language knowledge (host command lines, handshake quirks, contract-
//! surface policy tables §9) lives under `adapter/`; every other module is
//! language-neutral. `mock` is the fixture-grade test host CI runs against
//! (the workspace CI has no .NET SDK — `dec/ddd/fixtures-not-sdk`).

pub mod adapter;
pub mod classify;
pub mod client;
pub mod error;
pub mod host;
pub mod jsonrpc;
pub mod manager;
pub mod mock;
pub mod protocol;
/// The contract-surface vocabulary, owned by `ddd-core` because the policy
/// table mechanism is language-neutral; re-exported here so adapters keep
/// reaching it as `crate::surface`.
pub use ddd_core::surface;

pub use error::{LspError, Result};
pub use manager::HostManager;
