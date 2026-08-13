//! Reopen edges over hand-built stores: firing, coverage, the gates, and
//! the separation from basis loss.

use super::*;
use crate::claim::Claim;
use crate::decision::{BasedOn, BasisPin, Decision, DecisionKind};

fn claim(status: ClaimStatus, changed: &str) -> Claim {
    Claim {
        format: 2,
        id: "DDD-adapter-02".into(),
        statement: "Internal members leak across boundaries.".into(),
        status,
        evidence: None,
        falsifier: Some("a boundary defect traced to an internal member".into()),
        owner: "none".into(),
        changed: changed.into(),
        revalidate_by: None,
        version_index: None,
        depends_on: Vec::new(),
        refines: Vec::new(),
        predicate: None,
    }
}

fn decision(revisit: Vec<RevisitEdge>, based_on: Vec<BasedOn>) -> Decision {
    Decision {
        format: 7,
        id: "dec/ddd/internal-not-surface".into(),
        kind: DecisionKind::Decision,
        title: "internal is not contract surface".into(),
        rationale: "App-repo posture is the default.".into(),
        principal: "Emil".into(),
        based_on,
        revisit_if: revisit,
        date: Some("2026-08-06".into()),
        notes: None,
    }
}

fn edge(at: &Claim) -> RevisitEdge {
    RevisitEdge {
        claim: at.id.clone(),
        status: at.status,
        changed: at.changed.clone(),
        content: crate::basis_pin::claim_content_hash(at),
    }
}

fn store_of(d: Decision, c: Claim) -> DddStore {
    let mut store = DddStore::default();
    store.decisions.push(d);
    store.claims.push(c);
    store
}

fn mandate() -> BasedOn {
    BasedOn::Typed(crate::decision::TypedBasis {
        basis_type: crate::decision::BasisType::Mandate,
        statement: Some("The principal's ruling.".into()),
        reference: None,
    })
}

#[test]
fn an_unmoved_reopen_edge_does_not_fire_but_is_counted() {
    let c = claim(ClaimStatus::Projected, "2026-08-06");
    let store = store_of(decision(vec![edge(&c)], vec![mandate()]), c);
    let (fired, checked) = reopen_scan(&store);
    assert!(fired.is_empty());
    assert_eq!(checked, 1);
}

#[test]
fn a_status_move_on_a_reopen_edge_fires_a_reopen_and_says_so() {
    let pinned = claim(ClaimStatus::Projected, "2026-08-06");
    let e = edge(&pinned);
    let moved = claim(ClaimStatus::Retired, "2026-08-13");
    let store = store_of(decision(vec![e], vec![mandate()]), moved);
    let (fired, checked) = reopen_scan(&store);
    assert_eq!(checked, 1);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].current_status, Some(ClaimStatus::Retired));
    let message = fired[0].message();
    assert!(message.contains("reopen:"), "{message}");
    assert!(message.contains("due a fresh look"), "{message}");
    assert!(
        !message.contains("basis loss") && !message.contains("basis-loss"),
        "a reopen must never report as basis loss: {message}"
    );
}

/// The load-bearing separation: a reopen edge is invisible to the
/// basis-loss scan, whatever it does. If this ever fails, the ruling has
/// been undone somewhere.
#[test]
fn a_fired_reopen_edge_never_appears_in_basis_loss() {
    let pinned = claim(ClaimStatus::Projected, "2026-08-06");
    let e = edge(&pinned);
    let moved = claim(ClaimStatus::Retired, "2026-08-13");
    let store = store_of(decision(vec![e], vec![mandate()]), moved);
    let (losses, coverage) = crate::report::basis_scan_for_tests(&store);
    assert!(losses.is_empty(), "a reopen edge surfaced as basis loss: {losses:?}");
    assert_eq!(coverage.pinned, 0, "a reopen edge was counted as a pinned basis");
    let (fired, _) = reopen_scan(&store);
    assert_eq!(fired.len(), 1, "and it did fire on its own channel");
}

#[test]
fn a_vanished_claim_fires_with_no_current_state() {
    let pinned = claim(ClaimStatus::Projected, "2026-08-06");
    let e = edge(&pinned);
    let mut store = DddStore::default();
    store.decisions.push(decision(vec![e], vec![mandate()]));
    let (fired, _) = reopen_scan(&store);
    assert_eq!(fired.len(), 1);
    assert!(fired[0].current_status.is_none());
    assert!(fired[0].message().contains("no longer exists"));
}

#[test]
fn reopen_edges_below_format_seven_are_a_violation() {
    let c = claim(ClaimStatus::Projected, "2026-08-06");
    let mut d = decision(vec![edge(&c)], vec![mandate()]);
    d.format = 6;
    let out = gates(&d);
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].message.contains("format: 7"), "{:?}", out[0]);
}

#[test]
fn an_unpinned_reopen_edge_is_a_violation_because_it_can_never_fire() {
    let c = claim(ClaimStatus::Projected, "2026-08-06");
    let mut e = edge(&c);
    e.content = String::new();
    let out = gates(&decision(vec![e], vec![mandate()]));
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].message.contains("cannot fire"), "{:?}", out[0]);
}

#[test]
fn one_claim_cannot_be_both_ground_and_tripwire() {
    let c = claim(ClaimStatus::Projected, "2026-08-06");
    let ground = BasedOn::Pinned(BasisPin {
        basis_type: Some(crate::decision::BasisType::Claim),
        claim: c.id.clone(),
        status: c.status,
        changed: c.changed.clone(),
        content: Some(crate::basis_pin::claim_content_hash(&c)),
    });
    let out = gates(&decision(vec![edge(&c)], vec![ground]));
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].message.contains("both ground and reopen"), "{:?}", out[0]);
}

/// Re-typing an edge from ground to tripwire changes what the decision
/// says, so it must move the decision's content hash — that is what makes
/// the migration a re-decision the ledger can pin, not a silent rewrite.
#[test]
fn moving_an_edge_from_ground_to_reopen_moves_the_decision_content_hash() {
    let c = claim(ClaimStatus::Projected, "2026-08-06");
    let ground = BasedOn::Pinned(BasisPin {
        basis_type: Some(crate::decision::BasisType::Claim),
        claim: c.id.clone(),
        status: c.status,
        changed: c.changed.clone(),
        content: Some(crate::basis_pin::claim_content_hash(&c)),
    });
    let before = decision(Vec::new(), vec![mandate(), ground]);
    let after = decision(vec![edge(&c)], vec![mandate()]);
    assert_ne!(
        crate::basis_pin::decision_content_hash(&before),
        crate::basis_pin::decision_content_hash(&after)
    );
}
