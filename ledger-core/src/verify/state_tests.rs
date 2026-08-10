//! The seven disposition states, each provoked from a minimal store.

use crate::allocation::AllocationKind;
use crate::testkit;
use crate::verify::view::View;

use super::*;

fn today() -> NaiveDate {
    testkit::date("2026-08-10")
}

fn state_of(cs: crate::changeset::ChangeSet) -> DispositionState {
    let store = testkit::store(cs);
    let view = View::build(&store);
    let rows = states(&view, today());
    rows.get(&testkit::decision_id().to_string()).expect("row").state
}

#[test]
fn an_unallocated_decision_is_undecided() {
    let mut raw = testkit::version();
    raw.allocation = None;
    raw.discharge.clear();
    let cs = testkit::changeset(vec![testkit::sealed(raw)], Vec::new());
    assert_eq!(state_of(cs), DispositionState::Undecided);
}

#[test]
fn an_unsigned_allocation_awaits_acceptance() {
    let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
    assert_eq!(state_of(cs), DispositionState::AwaitingAcceptance);
}

#[test]
fn a_live_unexpired_acceptance_is_decided() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let cs = testkit::changeset(vec![sealed], vec![acceptance]);
    assert_eq!(state_of(cs), DispositionState::Decided);
}

#[test]
fn a_revoked_acceptance_returns_the_decision_to_awaiting() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let mut cs = testkit::changeset(vec![sealed], vec![acceptance]);
    cs.revocations.push(crate::acceptance::Revocation {
        acceptance: testkit::acceptance_id(),
        at: testkit::stamp("2026-08-10T10:00:00Z"),
        by: testkit::identity("fixture-human@example"),
        reason: "wrong version".into(),
    });
    assert_eq!(state_of(cs), DispositionState::AwaitingAcceptance);
}

fn escaped(review_by: &str) -> crate::version::VersionRaw {
    let mut raw = testkit::version();
    raw.allocation = Some(AllocationKind::Escaped);
    raw.discharge.clear();
    raw.exposure = Some("downstream may break silently".into());
    raw.accepted_by = Some(testkit::identity("fixture-human@example"));
    raw.review_by = Some(testkit::date(review_by));
    raw
}

#[test]
fn an_escape_with_a_future_review_date_is_priced() {
    let cs = testkit::changeset(vec![testkit::sealed(escaped("2026-09-01"))], Vec::new());
    assert_eq!(state_of(cs), DispositionState::EscapedPriced);
}

#[test]
fn an_escape_past_its_review_date_is_due() {
    let cs = testkit::changeset(vec![testkit::sealed(escaped("2026-08-01"))], Vec::new());
    assert_eq!(state_of(cs), DispositionState::EscapeReviewDue);
}

#[test]
fn an_acceptance_past_its_expiry_is_expired() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.expires_at = Some(testkit::date("2026-08-01"));
    let cs = testkit::changeset(vec![sealed], vec![acceptance]);
    assert_eq!(state_of(cs), DispositionState::Expired);
}

#[test]
fn a_superseded_decision_is_terminal_whatever_else_holds() {
    // The old decision is decided; a second decision's latest version
    // supersedes it, and that reading wins.
    let old = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&old);
    let successor_id: crate::id::DecisionId =
        "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY0".parse().expect("id");
    let mut successor = testkit::version();
    successor.decision = successor_id;
    successor.supersedes = Some(testkit::decision_id());
    let cs = testkit::changeset(vec![old, testkit::sealed(successor)], vec![acceptance]);
    let store = testkit::store(cs);
    let view = View::build(&store);
    let rows = states(&view, today());
    let old_row = rows.get(&testkit::decision_id().to_string()).expect("row");
    assert_eq!(old_row.state, DispositionState::Superseded);
    assert_eq!(
        old_row.superseded_by.as_deref(),
        Some("dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY0")
    );
}

#[test]
fn a_chain_walks_to_its_tip() {
    let mut map = BTreeMap::new();
    map.insert("a".to_string(), "b".to_string());
    map.insert("b".to_string(), "c".to_string());
    assert_eq!(chain_from("a", &map), vec!["a", "b", "c"]);
    assert_eq!(chain_from("c", &map), vec!["c"]);
}

#[test]
fn the_vocabulary_is_closed_at_seven() {
    assert_eq!(DispositionState::ALL.len(), 7);
    let names: Vec<&str> = DispositionState::ALL.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        names,
        [
            "undecided",
            "awaiting-acceptance",
            "decided",
            "escaped-priced",
            "escape-review-due",
            "expired",
            "superseded"
        ]
    );
}
