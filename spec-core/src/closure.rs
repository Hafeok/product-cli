//! How an act-time record is closed.
//!
//! Two kinds, exhaustively. Silence is neither of them: a record with no
//! closure is open, never `nothing-arose`. Absence and decision must not look
//! alike, so the positive declaration carries its own discriminant.

use chrono::{DateTime, Utc};
use ledger_core::identity::Identity;
use serde::{Deserialize, Serialize};

/// What the closer declares acting produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClosureKind {
    /// Determinations were produced while acting, filed at the address they
    /// were read from.
    Determinations,
    /// A positive declaration that acting produced no determination — the
    /// same shape as `asserted-none` for an uncovered set.
    NothingArose,
}

impl ClosureKind {
    /// The on-disk spelling, for messages that quote it back.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Determinations => "determinations",
            Self::NothingArose => "nothing-arose",
        }
    }
}

/// The closure half of an act-time record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    pub kind: ClosureKind,
    /// Who answers for it. A machine identity is refused at `S002`.
    pub principal: Identity,
    pub at: DateTime<Utc>,
    /// The determination addresses this closure files. Empty for
    /// `nothing-arose`; non-empty is what `determinations` means.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub determinations: Vec<String>,
    /// Digest binding this closure to the exact opening it discharges.
    pub binds: String,
}

impl Closure {
    /// Whether the kind agrees with the determination list it carries.
    ///
    /// The two disagreeing is `S004`: a `determinations` closure with nothing
    /// filed claims a write that did not happen, and a `nothing-arose`
    /// closure with a list contradicts its own declaration.
    pub fn kind_matches_payload(&self) -> bool {
        match self.kind {
            ClosureKind::Determinations => !self.determinations.is_empty(),
            ClosureKind::NothingArose => self.determinations.is_empty(),
        }
    }
}

#[path = "closure_tests.rs"]
#[cfg(test)]
mod tests;
