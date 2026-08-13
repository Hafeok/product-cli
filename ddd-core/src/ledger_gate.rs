//! The ledger's reading, surfaced through `ddd report` (M8 phase 4).
//!
//! Readiness and completeness are the ledger's gates — `ddd` delegates
//! to `ledger verify` and restates the result rather than growing a
//! second rulebook. The exact basis-loss check over migrated entries
//! lives here too: a `claim:`/`watched:`/`indeterminate:` token pins a
//! content hash, and drift is pinned-hash ≠ current-hash (invariant 2).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::store::DddStore;

/// The ledger's verdict as one report section.
#[derive(Debug, Serialize)]
pub struct LedgerSection {
    /// `ledger verify --gate readiness` would pass.
    pub readiness: bool,
    /// `ledger verify --gate completeness` would pass.
    pub completeness: bool,
    pub findings: Vec<String>,
    pub awaiting_acceptance: usize,
    /// Disposition counts across the store (the coverage instrument).
    pub dispositions: BTreeMap<String, usize>,
    /// Exact pin drift on migrated entries' claim tokens.
    pub pin_drift: Vec<PinDrift>,
    /// Claim-token pins checked (the coverage denominator).
    pub pins_checked: usize,
    /// Reopen edges that have fired, from ledger versions' `revisit_if`.
    /// A separate list from `pin_drift` for the same reason the store's
    /// report keeps them apart: a fired tripwire is not a lost basis.
    pub reopened: Vec<Reopened>,
    /// Reopen edges checked (the coverage denominator for that list).
    pub reopen_checked: usize,
}

/// One ledger `revisit_if` edge whose claim has moved.
#[derive(Debug, Serialize)]
pub struct Reopened {
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical: Option<String>,
    pub claim: String,
    pub pinned: String,
    /// `None` when the claim no longer exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
}

/// One migrated basis pin whose claim content moved.
#[derive(Debug, Serialize)]
pub struct PinDrift {
    pub decision: String,
    /// The historical id, when the concordance maps one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical: Option<String>,
    /// `claim` | `indeterminate` — an indeterminate edge drifting is
    /// still worth seeing, but it never grounded the decision. `watched`
    /// was retired here on 2026-08-13: a watched edge is a `revisit_if`
    /// edge now, and it reports through [`Reopened`], not through this
    /// list.
    pub marker: String,
    pub claim: String,
    pub pinned: String,
    /// `None` when the claim no longer exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
}

/// The ledger section, when the repo carries a `.decisions/` store.
pub fn ledger_section(
    repo_root: &Path,
    store: &DddStore,
    today: chrono::NaiveDate,
) -> Option<LedgerSection> {
    if !repo_root.join(ledger_core::STORE_DIR).is_dir() {
        return None;
    }
    let ledger = ledger_core::store::load(repo_root);
    let full = ledger_core::verify::verify(
        &ledger,
        &ledger_core::verify::Options { gate: None, today, blame: false },
    );
    let readiness = gate_passes(&ledger, today, ledger_core::finding::Gate::Readiness);
    let completeness = gate_passes(&ledger, today, ledger_core::finding::Gate::Completeness);
    let coverage = ledger_core::coverage::coverage(&ledger, today, None);
    let mut dispositions: BTreeMap<String, usize> = BTreeMap::new();
    for state in coverage.states.values() {
        *dispositions.entry(state.as_str().to_string()).or_default() += 1;
    }
    let (pin_drift, pins_checked) = pin_scan(store, &ledger);
    let (reopened, reopen_checked) = reopen_scan(store, &ledger);
    Some(LedgerSection {
        readiness,
        completeness,
        findings: full
            .findings
            .iter()
            .map(ToString::to_string)
            .chain(full.graph.iter().map(|g| format!("[{}] {}: {}", g.class, g.subject, g.message)))
            .collect(),
        awaiting_acceptance: full.awaiting_acceptance.len(),
        dispositions,
        pin_drift,
        pins_checked,
        reopened,
        reopen_checked,
    })
}

/// Every `revisit_if` edge on a latest version, compared against the
/// claim's current content. Reads a different field from [`pin_scan`] and
/// reports into a different list: the ruling is that these are two facts,
/// so nothing here may fold into the basis-loss output.
fn reopen_scan(store: &DddStore, ledger: &ledger_core::store::Store) -> (Vec<Reopened>, usize) {
    let concordance = crate::concordance::load(&store.dir);
    let view = ledger_core::verify::view::View::build(ledger);
    let mut fired = Vec::new();
    let mut checked = 0usize;
    for viewed in view.latest_versions() {
        for token in &viewed.raw.revisit_if {
            let Some((claim_id, pinned)) = parse_reopen(&token.to_string()) else {
                continue;
            };
            checked += 1;
            let current = store
                .claims
                .iter()
                .find(|c| c.id == claim_id)
                .map(crate::basis_pin::claim_content_hash);
            if current.as_deref() != Some(pinned.as_str()) {
                let decision = viewed.raw.decision.to_string();
                fired.push(Reopened {
                    historical: concordance
                        .as_ref()
                        .and_then(|c| c.historical_of(&decision))
                        .map(str::to_string),
                    decision,
                    claim: claim_id,
                    pinned,
                    current,
                });
            }
        }
    }
    fired.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    (fired, checked)
}

/// `claim:DDD-x-01@sha256:…` → (`DDD-x-01`, `sha256:…`). A reopen token
/// carries no marker vocabulary of its own: the field it sits in already
/// says what kind of edge it is.
fn parse_reopen(token: &str) -> Option<(String, String)> {
    let rest = token.strip_prefix("claim:")?;
    let (claim, hash) = rest.split_once('@')?;
    hash.starts_with("sha256:").then(|| (claim.to_string(), hash.to_string()))
}

fn gate_passes(
    ledger: &ledger_core::store::Store,
    today: chrono::NaiveDate,
    gate: ledger_core::finding::Gate,
) -> bool {
    ledger_core::verify::verify(
        ledger,
        &ledger_core::verify::Options { gate: Some(gate), today, blame: false },
    )
    .is_conformant()
}

/// Every `claim:`/`watched:`/`indeterminate:` token on a latest version,
/// compared exactly against the claim's current content hash.
fn pin_scan(store: &DddStore, ledger: &ledger_core::store::Store) -> (Vec<PinDrift>, usize) {
    let concordance = crate::concordance::load(&store.dir);
    let view = ledger_core::verify::view::View::build(ledger);
    let mut drift = Vec::new();
    let mut checked = 0usize;
    for viewed in view.latest_versions() {
        for token in &viewed.raw.based_on {
            let token = token.to_string();
            let Some((marker, claim_id, pinned)) = parse_pin(&token) else {
                continue;
            };
            checked += 1;
            let current = store
                .claims
                .iter()
                .find(|c| c.id == claim_id)
                .map(crate::basis_pin::claim_content_hash);
            if current.as_deref() != Some(pinned.as_str()) {
                let decision = viewed.raw.decision.to_string();
                drift.push(PinDrift {
                    historical: concordance
                        .as_ref()
                        .and_then(|c| c.historical_of(&decision))
                        .map(str::to_string),
                    decision,
                    marker: marker.to_string(),
                    claim: claim_id.to_string(),
                    pinned,
                    current,
                });
            }
        }
    }
    drift.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    (drift, checked)
}

/// `claim:DDD-x-01@sha256:…` → (`claim`, `DDD-x-01`, `sha256:…`).
fn parse_pin(token: &str) -> Option<(&str, &str, String)> {
    let (marker, rest) = token.split_once(':')?;
    // `watched` is deliberately absent: as of the 2026-08-13 ruling a
    // watched edge is a `revisit_if` edge in its own field, and a basis
    // scan that still recognised the marker would keep reading it as
    // ground — the exact conflation the ruling ends.
    if !matches!(marker, "claim" | "indeterminate") {
        return None;
    }
    let (claim, hash) = rest.split_once('@')?;
    hash.starts_with("sha256:").then(|| (marker, claim, hash.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_tokens_parse_and_others_are_ignored() {
        let (m, c, h) = parse_pin("claim:DDD-x-01@sha256:abc").expect("parse");
        assert_eq!((m, c, h.as_str()), ("claim", "DDD-x-01", "sha256:abc"));
        assert!(parse_pin("indeterminate:DDD-y-02@sha256:def").is_some());
        assert!(parse_pin("ddd-content:dec/x/y@sha256:abc").is_none());
        assert!(parse_pin("mandate:dec/x/y").is_none());
        assert!(parse_pin("claim:DDD-x-01").is_none());
    }

    /// The retirement, asserted rather than assumed: a stray `watched:`
    /// token left in a `based_on` list is no longer read as a basis pin,
    /// so it can never surface as basis loss again.
    #[test]
    fn a_watched_marker_is_no_longer_a_basis_pin() {
        assert!(parse_pin("watched:DDD-y-02@sha256:def").is_none());
    }

    #[test]
    fn reopen_tokens_parse_from_the_field_without_a_marker_vocabulary() {
        let (claim, hash) = parse_reopen("claim:DDD-adapter-02@sha256:b333").expect("parse");
        assert_eq!((claim.as_str(), hash.as_str()), ("DDD-adapter-02", "sha256:b333"));
        assert!(parse_reopen("watched:DDD-adapter-02@sha256:b333").is_none());
        assert!(parse_reopen("claim:DDD-adapter-02").is_none());
    }
}
