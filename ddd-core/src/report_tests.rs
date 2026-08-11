//! Cadence plus basis-loss detection over hand-built stores.

use super::*;
use crate::claim::Claim;
use crate::decision::{BasedOn, BasisPin, Decision, DecisionKind};
use crate::detect::DetectedState;

fn claim(id: &str, status: ClaimStatus, changed: &str) -> Claim {
    Claim {
        format: 2,
        id: id.into(),
        statement: "s".into(),
        status,
        evidence: Some("e".into()),
        falsifier: Some("f".into()),
        owner: "none".into(),
        changed: changed.into(),
        revalidate_by: None,
        version_index: None,
        depends_on: Vec::new(),
        refines: Vec::new(),
        predicate: None,
    }
}

fn pinned_decision(id: &str, claim: &str, status: ClaimStatus, changed: &str) -> Decision {
    Decision {
        format: 2,
        id: id.into(),
        kind: DecisionKind::Decision,
        title: "T".into(),
        rationale: "R".into(),
        principal: "Emil".into(),
        based_on: vec![BasedOn::Pinned(BasisPin {
        basis_type: None,
            claim: claim.into(),
            status,
            changed: changed.into(),
        })],
        date: None,
        notes: None,
    }
}

fn report(store: &DddStore) -> EscapesReport {
    report_escapes(store, &DetectedState::default(), "2026-08-06")
}

#[test]
fn a_claim_past_its_cadence_is_flagged() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1", ClaimStatus::Reported, "2026-01-01");
    c.revalidate_by = Some("2026-06-01".into());
    s.claims.push(c);
    let r = report(&s);
    assert_eq!(r.cadence.len(), 1);
    assert_eq!(r.cadence[0].claim, "DDD-a-1");
    assert_eq!(r.cadence[0].revalidate_by, "2026-06-01");
}

#[test]
fn future_cadence_or_no_cadence_is_clean() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1", ClaimStatus::Reported, "2026-01-01");
    c.revalidate_by = Some("2027-01-01".into());
    s.claims.push(c);
    s.claims.push(claim("DDD-a-2", ClaimStatus::Reported, "2026-01-01"));
    assert!(report(&s).is_clean());
}

#[test]
fn a_retired_claim_carries_no_cadence_duty() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1", ClaimStatus::Retired, "2026-01-01");
    c.revalidate_by = Some("2026-06-01".into());
    s.claims.push(c);
    assert!(report(&s).cadence.is_empty());
}

#[test]
fn a_status_change_since_the_pin_is_basis_loss() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1", ClaimStatus::Retired, "2026-08-05"));
    s.decisions.push(pinned_decision("dec/a/adopt", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let r = report(&s);
    assert_eq!(r.basis_loss.len(), 1);
    let loss = &r.basis_loss[0];
    assert_eq!(loss.decision, "dec/a/adopt");
    assert_eq!(loss.pinned_status, ClaimStatus::Reported);
    assert_eq!(loss.current_status, Some(ClaimStatus::Retired));
}

#[test]
fn a_changed_date_alone_is_also_basis_loss() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1", ClaimStatus::Reported, "2026-08-05"));
    s.decisions.push(pinned_decision("dec/a/adopt", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let r = report(&s);
    assert_eq!(r.basis_loss.len(), 1);
    assert_eq!(r.basis_loss[0].current_changed.as_deref(), Some("2026-08-05"));
}

#[test]
fn a_matching_pin_is_clean_and_plain_edges_are_outside_the_check() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    s.decisions.push(pinned_decision("dec/a/adopt", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let mut v1 = pinned_decision("dec/a/old", "DDD-a-1", ClaimStatus::Reported, "2026-08-02");
    v1.format = 1;
    v1.based_on = vec![BasedOn::Plain("DDD-a-1".into())];
    s.decisions.push(v1);
    assert!(report(&s).is_clean());
}

#[test]
fn a_vanished_claim_is_basis_loss_with_no_current_state() {
    let mut s = DddStore::default();
    s.decisions.push(pinned_decision("dec/a/adopt", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let r = report(&s);
    assert_eq!(r.basis_loss.len(), 1);
    assert_eq!(r.basis_loss[0].current_status, None);
}

// Coverage (dec/ddd/report-coverage-explicit): an unpinned edge is
// uncheckable, not clean, and must never fail the gate.

#[test]
fn an_unpinned_edge_is_reported_as_uncheckable_not_clean() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let mut plain = pinned_decision("dec/a/old", "DDD-a-1", ClaimStatus::Reported, "2026-08-02");
    plain.format = 1;
    plain.based_on = vec![BasedOn::Plain("DDD-a-1".into())];
    s.decisions.push(plain);
    let r = report(&s);
    assert_eq!(r.basis_coverage.pinned, 0);
    assert_eq!(r.basis_coverage.unpinned.len(), 1);
    assert_eq!(r.basis_coverage.unpinned[0].decision, "dec/a/old");
    assert_eq!(r.basis_coverage.unpinned[0].claim, "DDD-a-1");
    // Uncheckable is a note, never an escape.
    assert!(r.is_clean(), "unpinned edges must not fail the gate");
}

#[test]
fn basis_coverage_counts_both_sides_of_a_mixed_graph() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    s.decisions.push(pinned_decision("dec/a/one", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    s.decisions.push(pinned_decision("dec/a/two", "DDD-a-1", ClaimStatus::Reported, "2026-08-02"));
    let mut plain = pinned_decision("dec/a/old", "DDD-a-1", ClaimStatus::Reported, "2026-08-02");
    plain.based_on = vec![BasedOn::Plain("DDD-a-1".into())];
    s.decisions.push(plain);
    let r = report(&s);
    assert_eq!(r.basis_coverage.pinned, 2);
    assert_eq!(r.basis_coverage.unpinned.len(), 1);
}

#[test]
fn a_decision_with_no_basis_contributes_to_neither_side() {
    let mut s = DddStore::default();
    let mut bare = pinned_decision("dec/a/bare", "DDD-a-1", ClaimStatus::Reported, "2026-08-02");
    bare.based_on = Vec::new();
    s.decisions.push(bare);
    let r = report(&s);
    assert_eq!(r.basis_coverage.pinned, 0);
    assert!(r.basis_coverage.unpinned.is_empty());
    assert!(r.is_clean());
}

#[test]
fn cadence_coverage_separates_dated_claims_from_undated_ones() {
    let mut s = DddStore::default();
    let mut dated = claim("DDD-a-1", ClaimStatus::Reported, "2026-01-01");
    dated.revalidate_by = Some("2027-01-01".into());
    s.claims.push(dated);
    s.claims.push(claim("DDD-a-2", ClaimStatus::Reported, "2026-01-01"));
    // Retired claims carry no duty, so they are outside the check entirely.
    s.claims.push(claim("DDD-a-3", ClaimStatus::Retired, "2026-01-01"));
    let r = report(&s);
    assert_eq!(r.cadence_coverage.dated, 1);
    assert_eq!(r.cadence_coverage.undated, vec!["DDD-a-2".to_string()]);
    assert!(r.is_clean());
}
