//! MCP health-check handlers (FT-059).
//!
//! Read-only tools that mirror the CLI's `product drift check` and
//! `product preflight` commands. Errors flow through the JSON-RPC error
//! response with the FT-059 health-check codes — E022 (id not found),
//! E023 (conflicting args), E024 (TC runner config missing).
//!
//! Submodules:
//! - [`drift_check`] — `product_drift_check` handler.
//! - [`preflight`] — `product_preflight` handler.
//! - [`shared`] — common envelope helpers (status, summary, error encoding).

pub(crate) mod drift_check;
pub(crate) mod preflight;
pub(crate) mod shared;

pub(crate) use drift_check::handle_drift_check;
pub(crate) use preflight::handle_preflight;
