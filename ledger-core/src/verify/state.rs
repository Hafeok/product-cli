//! The disposition state of a decision — the vocabulary coverage reports in.
//!
//! Seven states, closed: `undecided`, `awaiting-acceptance`, `decided`,
//! `escaped-priced`, `escape-review-due`, `expired`, `superseded`. They are
//! derived from the same [`View`] the gate rules read, so `status`,
//! `coverage` and `verify` can never disagree about what a decision's
//! situation is. Judged on latest versions only, like the gate; supersession
//! is terminal and overrides every other reading, because a superseded
//! decision's disposition is a resolved fact, not a pending one.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::allocation::Allocation;
use crate::version::DecisionVersion;

use super::view::View;

/// Where a decision stands, as one of the seven closed states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DispositionState {
    /// Enumerated, no allocation — the state `L001` fails at the gate.
    Undecided,
    /// Allocated, no live acceptance of the latest version. Pendency.
    AwaitingAcceptance,
    /// Allocated with a live, unexpired acceptance of the latest version.
    Decided,
    /// Escaped with its price stated and the review date still ahead.
    EscapedPriced,
    /// Escaped, and the stated review date has passed.
    EscapeReviewDue,
    /// Every live acceptance of the latest version has expired (`L003`).
    Expired,
    /// Another decision's latest version supersedes this one. Terminal.
    Superseded,
}

impl DispositionState {
    /// The kebab-case name used in reports and JSON alike.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Undecided => "undecided",
            Self::AwaitingAcceptance => "awaiting-acceptance",
            Self::Decided => "decided",
            Self::EscapedPriced => "escaped-priced",
            Self::EscapeReviewDue => "escape-review-due",
            Self::Expired => "expired",
            Self::Superseded => "superseded",
        }
    }

    /// Every state, in report order.
    pub const ALL: &'static [DispositionState] = &[
        Self::Undecided,
        Self::AwaitingAcceptance,
        Self::Decided,
        Self::EscapedPriced,
        Self::EscapeReviewDue,
        Self::Expired,
        Self::Superseded,
    ];
}

/// One decision's read state, keyed for the coverage roll-ups.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StateRow {
    pub decision: String,
    pub namespace: String,
    pub set: String,
    pub state: DispositionState,
    /// The decision whose latest version supersedes this one, when one does.
    pub superseded_by: Option<String>,
}

/// `old decision id -> superseding decision id`, read off latest versions.
pub fn supersession(view: &View) -> BTreeMap<String, String> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter_map(|p| {
            p.supersedes.as_ref().map(|old| (old.to_string(), p.decision.to_string()))
        })
        .collect()
}

/// The state of every decision in the log, keyed by decision id.
pub fn states(view: &View, today: NaiveDate) -> BTreeMap<String, StateRow> {
    let superseders = supersession(view);
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .map(|p| {
            let id = p.decision.to_string();
            let superseded_by = superseders.get(&id).cloned();
            let state = match &superseded_by {
                Some(_) => DispositionState::Superseded,
                None => allocation_state(view, p, today),
            };
            let row = StateRow {
                decision: id.clone(),
                namespace: p.decision.namespace().to_string(),
                set: p.set.clone(),
                state,
                superseded_by,
            };
            (id, row)
        })
        .collect()
}

fn allocation_state(view: &View, p: &DecisionVersion, today: NaiveDate) -> DispositionState {
    match &p.allocation {
        None => DispositionState::Undecided,
        Some(Allocation::Escaped { review_by, .. }) => {
            if *review_by < today {
                DispositionState::EscapeReviewDue
            } else {
                DispositionState::EscapedPriced
            }
        }
        Some(_) => acceptance_state(view, p, today),
    }
}

/// Live acceptances of the latest version decide between decided, expired,
/// and awaiting: any unexpired one carries the day, and an expired-only
/// signature is `expired` — the same reading `L003` fails on.
fn acceptance_state(view: &View, p: &DecisionVersion, today: NaiveDate) -> DispositionState {
    let live: Vec<_> = view
        .acceptances
        .iter()
        .map(|a| a.acceptance)
        .filter(|a| !view.is_revoked(a) && a.decision == p.decision && a.version == p.stored_hash)
        .collect();
    if live.iter().any(|a| !a.is_expired(today)) {
        DispositionState::Decided
    } else if !live.is_empty() {
        DispositionState::Expired
    } else {
        DispositionState::AwaitingAcceptance
    }
}

/// Walk a supersession chain forward from `id` to its current tip.
/// Cycles terminate at the first repeat rather than looping.
pub fn chain_from(id: &str, superseders: &BTreeMap<String, String>) -> Vec<String> {
    let mut chain = vec![id.to_string()];
    let mut cursor = id.to_string();
    while let Some(next) = superseders.get(&cursor) {
        if chain.contains(next) {
            break;
        }
        chain.push(next.clone());
        cursor = next.clone();
    }
    chain
}

#[path = "state_tests.rs"]
#[cfg(test)]
mod tests;
