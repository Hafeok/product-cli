//! Reconciling divergent version chains (L3, PRD §7).
//!
//! Merge is **two mechanisms, never one**. A git merge driver registered
//! for `.decisions/**` ([`driver`]) settles the mechanical majority —
//! disjoint decisions, disjoint sets, append-only log interleaving — and
//! refuses anything requiring judgment, exiting non-zero with the conflict
//! preserved. `ledger merge --resolve` ([`conflicts`] + [`resolve`]) then
//! presents each conflict — both chains, the common parent, the
//! consequences — and records the human's arbitration as an ordinary
//! ledger act: a new version, or an explicit withdrawal, attributed to the
//! arbitrating identity from git config. Never a silent resolution; never
//! a merge that succeeds by picking.
//!
//! What merge reconciles is *version chains*, not files: the log files are
//! append-only and never in conflict. Acceptances never survive
//! reconciliation — a reconciled version is a new version, its hash is
//! fresh, and every prior acceptance stays in the log as history about the
//! version it signed. The same law as `revise`: content moves, acceptance
//! does not follow it.

pub mod conflicts;
pub mod driver;
pub mod plan;
pub mod resolve;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use std::fmt;

use serde::Serialize;

/// The closed set of named conflict classes a merge can surface. None is
/// ever resolved automatically — not by last-write, not by newest ULID,
/// not by longest chain, not by "identical content".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictClass {
    /// Two writers revised one decision from a common parent.
    DivergentRevision,
    /// The divergence moved the allocation itself.
    DivergentAllocation,
    /// Two live claimants supersede one decision (`G005`).
    CompetingSupersession,
    /// One set file changed differently on both sides.
    DivergentFloor,
    /// A written log file was edited on both sides — append-only violated.
    EditedLogFile,
    /// Two different change-sets under one ULID.
    UlidCollision,
}

impl ConflictClass {
    pub fn code(self) -> &'static str {
        match self {
            Self::DivergentRevision => "divergent-revision",
            Self::DivergentAllocation => "divergent-allocation",
            Self::CompetingSupersession => "competing-supersession",
            Self::DivergentFloor => "divergent-floor",
            Self::EditedLogFile => "edited-log-file",
            Self::UlidCollision => "ulid-collision",
        }
    }
}

impl fmt::Display for ConflictClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
