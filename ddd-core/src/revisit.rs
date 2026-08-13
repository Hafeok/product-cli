//! The reopen edge — a claim whose death reopens a decision (format 7).
//!
//! Ruled by the principal (2026-08-13), adopting into `.ddd/` the one shape
//! the ledger format specifies at spec v1.5 / `format: 4` (§3.7). A
//! `revisit_if` edge is the converse of ground: the decision does not rest
//! on the claim, so falsifying the claim does not undermine the decision —
//! it obliges someone to look at the decision again.
//!
//! The separation is structural, not conventional. A reopen edge lives in
//! its own field, carries its own type, and is scanned by its own pass, so
//! `based_on`'s basis-loss scan cannot see it and `why` cannot render it as
//! ground. A movement here is a **reopen** finding: a distinct class with a
//! distinct message, because "the ground you stood on moved" and "the
//! tripwire you set has fired" are different facts about a decision, and a
//! report that merges them tells the reader neither.

use serde::{Deserialize, Serialize};

use crate::claim::ClaimStatus;
use crate::store::DddStore;

/// One `revisit_if` edge: the claim, pinned as a claim basis is pinned so
/// the same exact comparison is available. Pinning is what makes a reopen
/// detectable at all — an unpinned tripwire cannot fire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisitEdge {
    /// The claim id whose death reopens the decision.
    pub claim: String,
    /// The claim's status when the edge was filed.
    pub status: ClaimStatus,
    /// The claim's `changed` date when the edge was filed.
    pub changed: String,
    /// The claim's canonical content hash when the edge was filed
    /// (`ddd.claim-content.v1`). Required at format 7: the exactness the
    /// M8 content pin bought must not be lost by moving an edge.
    pub content: String,
}

/// A reopen edge whose claim has moved: the decision is due a fresh look.
/// Deliberately *not* a [`crate::report::BasisLoss`] — same shape of
/// comparison, different fact, different report.
#[derive(Debug, Serialize)]
pub struct Reopen {
    pub decision: String,
    pub claim: String,
    pub pinned_status: ClaimStatus,
    pub pinned_changed: String,
    pub pinned_content: String,
    /// `None` when the claim no longer exists at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_status: Option<ClaimStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_changed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_content: Option<String>,
}

impl Reopen {
    /// The finding line. It says *reopen*, never *basis loss*: a reader who
    /// sees this must know the decision needs revisiting, not that its
    /// ground has gone.
    pub fn message(&self) -> String {
        let moved = match (&self.current_status, &self.current_changed) {
            (Some(s), Some(c)) => format!("now {}@{c}", s.as_str()),
            _ => "the claim no longer exists".to_string(),
        };
        format!(
            "{} — reopen: {} was pinned at {}@{} and has moved ({moved}); the decision is due a fresh look, its ground is untouched",
            self.decision,
            self.claim,
            self.pinned_status.as_str(),
            self.pinned_changed,
        )
    }
}

/// Every reopen edge in the store, compared against its claim's current
/// content. Returns the fired edges plus how many were checked — the same
/// state-your-coverage discipline `basis_scan` follows.
pub fn reopen_scan(store: &DddStore) -> (Vec<Reopen>, usize) {
    let mut fired = Vec::new();
    let mut checked = 0usize;
    for d in &store.decisions {
        for edge in &d.revisit_if {
            checked += 1;
            let current = store.claims.iter().find(|c| c.id == edge.claim);
            let current_content = current.map(crate::basis_pin::claim_content_hash);
            let moved = current_content.as_deref() != Some(edge.content.as_str());
            if moved {
                fired.push(Reopen {
                    decision: d.id.clone(),
                    claim: edge.claim.clone(),
                    pinned_status: edge.status,
                    pinned_changed: edge.changed.clone(),
                    pinned_content: edge.content.clone(),
                    current_status: current.map(|c| c.status),
                    current_changed: current.map(|c| c.changed.clone()),
                    current_content,
                });
            }
        }
    }
    fired.sort_by(|a, b| (&a.decision, &a.claim).cmp(&(&b.decision, &b.claim)));
    (fired, checked)
}

/// Format gates for the reopen edges of one decision.
///
/// Three rules, each closing a way the ruling could be lost: the field is
/// format 7, every edge pins exactly, and no claim is both ground and
/// tripwire — an edge that is read both ways is exactly the conflation the
/// distinct edge type exists to end.
pub fn gates(d: &crate::decision::Decision) -> Vec<product_core::pf::validate::Violation> {
    let mut out = Vec::new();
    let mut push = |message: String| {
        out.push(product_core::pf::validate::Violation {
            focus: d.id.clone(),
            path: "revisit_if".to_string(),
            message,
            severity: "violation".to_string(),
        })
    };
    if d.revisit_if.is_empty() {
        return out;
    }
    if d.format < crate::validate::REVISIT_FORMAT {
        push("reopen edges are format 7 — declare `format: 7`".to_string());
    }
    for edge in &d.revisit_if {
        if edge.content.trim().is_empty() {
            push(format!(
                "the reopen edge on {} carries no content hash — an unpinned tripwire cannot fire",
                edge.claim
            ));
        }
        if d.based_on.iter().any(|b| b.claim_id() == Some(edge.claim.as_str())) {
            push(format!(
                "{} is filed as both ground and reopen edge — a claim is one or the other, which is the whole point of the distinct edge type",
                edge.claim
            ));
        }
    }
    out
}

#[cfg(test)]
#[path = "revisit_tests.rs"]
mod tests;
