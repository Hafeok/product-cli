//! Escape reporting — every escaped decision the graph can currently name.
//!
//! Three sections (PRD §7): the governance diff findings, claims past their
//! `revalidate_by` cadence (the stale-catalog mitigation, PRD §11), and
//! basis loss — decisions whose pinned `based_on` status or `changed` date
//! no longer matches the claim (PRD §6 rule 6). Cadence plus basis loss
//! only exist for format-2 entries, so each section also reports its own
//! coverage: what it checked and what it could not see
//! (`dec/ddd/report-coverage-explicit`). Coverage is a note, never an
//! escape — formats 1-3 are all valid, so an unpinned edge must not fail
//! the gate.

use serde::Serialize;

use crate::claim::ClaimStatus;
use crate::detect::DetectedState;
use crate::diff::{diff, DiffReport};
use crate::store::DddStore;

/// The full escape report.
#[derive(Debug, Serialize)]
pub struct EscapesReport {
    pub diff: DiffReport,
    pub cadence: Vec<CadenceViolation>,
    pub cadence_coverage: CadenceCoverage,
    pub basis_loss: Vec<BasisLoss>,
    pub basis_coverage: BasisCoverage,
}

/// What the cadence check could and could not see. A claim with no
/// `revalidate_by` carries no revalidation duty to miss, so it is outside
/// the check rather than clean within it.
#[derive(Debug, Default, Serialize)]
pub struct CadenceCoverage {
    /// Live claims carrying a `revalidate_by` — the checked set.
    pub dated: usize,
    /// Live claims with no `revalidate_by`, by id.
    pub undated: Vec<String>,
}

/// What the basis-loss check could and could not see. Basis loss is only
/// detectable across a pinned (format-2) edge; a plain edge carries no
/// recorded state to compare against.
#[derive(Debug, Default, Serialize)]
pub struct BasisCoverage {
    /// Pinned `based_on` edges — the checked set.
    pub pinned: usize,
    /// Unpinned edges, as (decision, claim) pairs.
    pub unpinned: Vec<UnpinnedBasis>,
}

/// A `based_on` edge carrying no pin, so its claim may have moved unseen.
#[derive(Debug, Serialize)]
pub struct UnpinnedBasis {
    pub decision: String,
    pub claim: String,
}

/// A live claim whose revalidation date has passed.
#[derive(Debug, Serialize)]
pub struct CadenceViolation {
    pub claim: String,
    pub status: ClaimStatus,
    pub revalidate_by: String,
}

/// A pinned basedOn edge whose claim has moved since the decision.
#[derive(Debug, Serialize)]
pub struct BasisLoss {
    pub decision: String,
    pub claim: String,
    pub pinned_status: ClaimStatus,
    pub pinned_changed: String,
    /// `None` when the claim no longer exists at all.
    pub current_status: Option<ClaimStatus>,
    pub current_changed: Option<String>,
}

impl EscapesReport {
    pub fn is_clean(&self) -> bool {
        self.diff.findings.is_empty() && self.cadence.is_empty() && self.basis_loss.is_empty()
    }
}

/// Assemble the report; `today` is an ISO date (`YYYY-MM-DD`) so cadence
/// comparison is a plain string ordering.
pub fn report_escapes(store: &DddStore, detected: &DetectedState, today: &str) -> EscapesReport {
    let (cadence, cadence_coverage) = cadence_scan(store, today);
    let (basis_loss, basis_coverage) = basis_scan(store);
    EscapesReport {
        diff: diff(store, detected),
        cadence,
        cadence_coverage,
        basis_loss,
        basis_coverage,
    }
}

/// Past cadence: strictly before today; retired claims carry no duty. The
/// coverage half counts dated claims and names undated ones, from the same
/// pass so the two can never disagree.
fn cadence_scan(store: &DddStore, today: &str) -> (Vec<CadenceViolation>, CadenceCoverage) {
    let mut violations = Vec::new();
    let mut coverage = CadenceCoverage::default();
    for c in store.claims.iter().filter(|c| c.status != ClaimStatus::Retired) {
        let Some(by) = c.revalidate_by.as_deref() else {
            coverage.undated.push(c.id.clone());
            continue;
        };
        coverage.dated += 1;
        if by < today {
            violations.push(CadenceViolation {
                claim: c.id.clone(),
                status: c.status,
                revalidate_by: by.to_string(),
            });
        }
    }
    violations.sort_by(|a, b| a.claim.cmp(&b.claim));
    coverage.undated.sort();
    (violations, coverage)
}

/// Pinned edges are compared against the claim; unpinned edges are counted
/// as uncheckable rather than skipped silently.
fn basis_scan(store: &DddStore) -> (Vec<BasisLoss>, BasisCoverage) {
    let mut losses = Vec::new();
    let mut coverage = BasisCoverage::default();
    for d in &store.decisions {
        for basis in &d.based_on {
            let Some(pin) = basis.pin() else {
                coverage.unpinned.push(UnpinnedBasis {
                    decision: d.id.clone(),
                    claim: basis.claim_id().to_string(),
                });
                continue;
            };
            coverage.pinned += 1;
            let current = store.claims.iter().find(|c| c.id == pin.claim);
            let moved = match current {
                Some(c) => c.status != pin.status || c.changed != pin.changed,
                None => true,
            };
            if moved {
                losses.push(BasisLoss {
                    decision: d.id.clone(),
                    claim: pin.claim.clone(),
                    pinned_status: pin.status,
                    pinned_changed: pin.changed.clone(),
                    current_status: current.map(|c| c.status),
                    current_changed: current.map(|c| c.changed.clone()),
                });
            }
        }
    }
    losses.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    coverage.unpinned.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    (losses, coverage)
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
