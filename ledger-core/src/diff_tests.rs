//! The semantic diff over in-memory stores: each report field provoked.

use crate::store::Store;
use crate::testkit;
use crate::tier::Tier;

use super::{descends, diff};

fn base() -> Store {
    testkit::store(testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new()))
}

#[test]
fn identical_readings_diff_to_nothing() {
    let report = diff("a", &base(), "b", &base());
    assert!(report.is_empty(), "{report:?}");
    assert!(super::render(&report).contains("no decision-level change"));
}

#[test]
fn a_revision_is_a_fast_forward_tip_movement() {
    let root = testkit::sealed(testkit::version());
    let mut child = testkit::version();
    child.parent = Some(root.hash.clone());
    child.statement = "Restated.".into();
    let child = testkit::sealed(child);
    let to = testkit::store(testkit::changeset(vec![root, child], Vec::new()));

    let report = diff("a", &base(), "b", &to);
    assert_eq!(report.tips_moved.len(), 1, "{report:?}");
    let moved = &report.tips_moved[0];
    assert!(moved.fast_forward, "a child of the old tip is a fast-forward");
    assert!(moved.allocation.is_none(), "the allocation did not change");
    assert!(super::render(&report).contains("fast-forward"), "{}", super::render(&report));
}

#[test]
fn an_escape_shows_its_allocation_transition() {
    let root = testkit::sealed(testkit::version());
    let mut escaped = testkit::version();
    escaped.parent = Some(root.hash.clone());
    escaped.allocation = Some(crate::allocation::AllocationKind::Escaped);
    escaped.discharge.clear();
    escaped.exposure = Some("exports may drift".into());
    escaped.accepted_by = Some(testkit::identity("fixture-human@example"));
    escaped.review_by = Some(testkit::date("2026-09-01"));
    let escaped = testkit::sealed(escaped);
    let to = testkit::store(testkit::changeset(vec![root, escaped], Vec::new()));

    let report = diff("a", &base(), "b", &to);
    assert_eq!(report.tips_moved[0].allocation.as_deref(), Some("constraint -> escaped"));
}

#[test]
fn an_unrelated_tip_is_divergence_not_a_revision() {
    // The target never filed the old tip: its chain cannot reach it.
    let mut other = testkit::version();
    other.statement = "A different history entirely.".into();
    let to = testkit::store(testkit::changeset(vec![testkit::sealed(other)], Vec::new()));
    let report = diff("a", &base(), "b", &to);
    assert_eq!(report.tips_moved.len(), 1, "{report:?}");
    assert!(!report.tips_moved[0].fast_forward, "no parent path to the old tip");
    assert!(super::render(&report).contains("DIVERGED"), "{}", super::render(&report));
}

#[test]
fn added_decisions_sets_floors_and_acceptances_are_reported() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let mut to = testkit::store(testkit::changeset(vec![sealed.clone()], vec![acceptance]));
    let mut second_set = testkit::set();
    second_set.id = "second".into();
    second_set.tolerance_floor = Tier::T2;
    to.sets.push(second_set);
    to.sets[0].tolerance_floor = Tier::T2;

    let empty = Store::default();
    let report = diff("a", &empty, "b", &to);
    assert_eq!(report.decisions_added.len(), 1, "{report:?}");
    assert_eq!(report.sets_declared.len(), 2, "an empty store knows no sets");
    assert_eq!(report.acceptances_gained.len(), 1);

    let from = base();
    let report = diff("a", &from, "b", &to);
    assert_eq!(report.sets_declared, vec!["second (floor T2)"]);
    assert_eq!(report.floors_changed, vec!["ledger-design: T1 -> T2"]);
    assert_eq!(report.acceptances_gained.len(), 1);
    assert!(report.decisions_added.is_empty());
}

#[test]
fn a_removed_decision_and_a_lost_acceptance_are_stated_not_hidden() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let from = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    let report = diff("a", &from, "b", &Store::default());
    assert_eq!(report.decisions_removed.len(), 1, "{report:?}");
    assert_eq!(report.acceptances_lost.len(), 1);
}

#[test]
fn a_fork_in_the_target_is_reported_as_awaiting_arbitration() {
    let root = testkit::sealed(testkit::version());
    let mut left = testkit::version();
    left.parent = Some(root.hash.clone());
    left.statement = "left".into();
    let mut right = testkit::version();
    right.parent = Some(root.hash.clone());
    right.statement = "right".into();
    let to = testkit::store(testkit::changeset(
        vec![root, testkit::sealed(left), testkit::sealed(right)],
        Vec::new(),
    ));
    let report = diff("a", &base(), "b", &to);
    assert_eq!(report.forked, vec![testkit::decision_id().to_string()]);
}

#[test]
fn a_new_supersession_is_reported_with_its_successor() {
    let old = testkit::sealed(testkit::version());
    let mut successor = testkit::version();
    successor.decision =
        "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY5".parse().expect("decision id");
    successor.statement = "The narrowed replacement.".into();
    successor.supersedes = Some(testkit::decision_id());
    let to = testkit::store(testkit::changeset(
        vec![old.clone(), testkit::sealed(successor)],
        Vec::new(),
    ));
    let from = testkit::store(testkit::changeset(vec![old], Vec::new()));
    let report = diff("a", &from, "b", &to);
    assert_eq!(report.superseded.len(), 1, "{report:?}");
    assert!(report.superseded[0].contains("01K2C4YQJ3F8M0PT5W7NZ9RDY5"), "{report:?}");
}

#[test]
fn descends_walks_the_parent_chain_and_terminates_on_a_cycle() {
    let root = testkit::sealed(testkit::version());
    let mut child = testkit::version();
    child.parent = Some(root.hash.clone());
    child.statement = "Restated.".into();
    let child = testkit::sealed(child);
    let store = testkit::store(testkit::changeset(vec![root.clone(), child.clone()], Vec::new()));
    let view = crate::verify::view::View::build(&store);
    let id = testkit::decision_id().to_string();
    assert!(descends(&view, &id, &child.hash, &root.hash));
    assert!(!descends(&view, &id, &root.hash, &child.hash), "ancestry is directed");
}
