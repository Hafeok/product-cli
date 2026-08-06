//! Escape reporting — every escaped decision the graph can currently name.
//!
//! Three sections (PRD §7): the governance diff findings, claims past their
//! `revalidate_by` cadence (the stale-catalog mitigation, PRD §11), and
//! basis loss — decisions whose pinned `based_on` status or `changed` date
//! no longer matches the claim (PRD §6 rule 6). Cadence plus basis loss
//! only exist for format-2 entries; format-1 entries are silently outside
//! both checks, which the migration note documents.

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
    pub basis_loss: Vec<BasisLoss>,
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
    EscapesReport {
        diff: diff(store, detected),
        cadence: cadence_violations(store, today),
        basis_loss: basis_losses(store),
    }
}

/// Past cadence: strictly before today; retired claims carry no duty.
fn cadence_violations(store: &DddStore, today: &str) -> Vec<CadenceViolation> {
    let mut out: Vec<CadenceViolation> = store
        .claims
        .iter()
        .filter(|c| c.status != ClaimStatus::Retired)
        .filter_map(|c| {
            let by = c.revalidate_by.as_deref()?;
            (by < today).then(|| CadenceViolation {
                claim: c.id.clone(),
                status: c.status,
                revalidate_by: by.to_string(),
            })
        })
        .collect();
    out.sort_by(|a, b| a.claim.cmp(&b.claim));
    out
}

fn basis_losses(store: &DddStore) -> Vec<BasisLoss> {
    let mut out = Vec::new();
    for d in &store.decisions {
        for basis in &d.based_on {
            let Some(pin) = basis.pin() else {
                continue;
            };
            let current = store.claims.iter().find(|c| c.id == pin.claim);
            let moved = match current {
                Some(c) => c.status != pin.status || c.changed != pin.changed,
                None => true,
            };
            if moved {
                out.push(BasisLoss {
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
    out.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    out
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
