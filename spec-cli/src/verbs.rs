//! The verbs this binary owns — the half a model may not call.
//!
//! Each submodule is a thin adapter: parse the arguments, call `spec_core`,
//! wrap the result in a report. No verdict logic lives here. The gate is
//! `spec_core::gate`'s and the refusals are the store's, so nothing in the
//! CLI can be softer at the door than the gate is.

pub mod gate;
pub mod git;
pub mod policy;
pub mod ratify;
pub mod record;
pub mod trust;

pub use gate::{check, map, CheckArgs, MapArgs};
pub use policy::{set as policy_set, show as policy_show, PolicySetArgs, PolicyShowArgs};
pub use ratify::{accept, candidates, reject, AcceptArgs, CandidatesArgs, RejectArgs};
pub use record::{close, implement, records, CloseArgs, ImplementArgs, RecordsArgs};
pub use trust::{
    generate as trust_generate, list as trust_list, TrustGenerateArgs, TrustListArgs,
};

use ledger_core::identity::Identity;
use product_core::error::{ProductError, Result};
use std::path::Path;

/// Whose identity a verb acts under.
///
/// An explicit flag wins; otherwise git config answers. There is no third
/// source, and in particular no way to assert an identity the machine does not
/// already have — `--as` is the flag this deliberately does not offer.
pub fn resolve_identity(root: &Path, supplied: Option<&str>) -> Result<Identity> {
    let raw = match supplied {
        Some(s) => s.to_string(),
        None => ledger_core::whoami::git_identity(root)
            .map_err(ProductError::ConfigError)?
            .as_str()
            .to_string(),
    };
    raw.parse().map_err(ProductError::ConfigError)
}

#[path = "verbs_tests.rs"]
#[cfg(test)]
mod tests;
