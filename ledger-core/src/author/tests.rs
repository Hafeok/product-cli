//! The verbs, each round-tripped then refused, over a tempdir store.

use chrono::{DateTime, Utc};

use crate::allocation::AllocationKind;
use crate::finding::VerifyClass;
use crate::id::DecisionId;
use crate::mint::UlidMint;
use crate::set::Ground;
use crate::testkit;
use crate::tier::Tier;
use crate::verify::{self, Options};

use super::*;

fn now() -> DateTime<Utc> {
    testkit::stamp("2026-08-10T12:00:00Z")
}

/// A scaffolded store with a human author and a deterministic mint.
fn author(dir: &Path) -> Author {
    crate::init::apply_init(dir, &crate::init::plan_init(dir)).expect("init");
    Author::new(dir, testkit::identity("fixture-human@example"), now(), UlidMint::fixed(1_755_000_000_000))
}

fn declare(a: &mut Author) {
    a.declare(DeclareArgs {
        id: "verbs".into(),
        title: "Verb round-trips".into(),
        tolerance_floor: Tier::T1,
        ground: Ground::Characterised,
        owner: None,
        notes: None,
    })
    .expect("declare");
}

/// The constraint `add` the fixtures file, varying only in statement and override.
fn constraint_add(statement: &str, tolerance_override: Option<Tier>) -> AddArgs {
    AddArgs {
        set: "verbs".into(),
        statement: statement.into(),
        namespace: Some("fixture.verbs".into()),
        allocation: AllocationArgs {
            store: Some(AllocationKind::Constraint),
            discharge: vec!["analyzer:DEC001".parse().expect("ref")],
            ..AllocationArgs::default()
        },
        tolerance_override,
        based_on: Vec::new(),
        revisit_if: Vec::new(),
        note: None,
    }
}

fn add_constraint(a: &mut Author) -> DecisionId {
    let applied = a
        .add(constraint_add("Monetary amounts use decimal, never double.", None))
        .expect("add");
    extract_decision(&applied)
}

fn extract_decision(applied: &Applied) -> DecisionId {
    let line = applied.lines.first().expect("line");
    let id = line.split_whitespace().nth(1).expect("id in line");
    id.parse().expect("decision id")
}

fn gate(dir: &Path) -> Vec<VerifyClass> {
    let store = crate::store::load(dir);
    let report = verify::verify(
        &store,
        &Options { gate: None, today: testkit::date("2026-08-10"), blame: false },
    );
    report.findings.iter().map(|f| f.class).collect()
}

#[test]
fn declare_add_allocate_accept_round_trip_and_survive_verify() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    a.accept(AcceptArgs { decision: id.clone(), expires_at: Some(testkit::date("2027-08-10")) })
        .expect("accept");
    assert_eq!(gate(dir.path()), Vec::<VerifyClass>::new());
    let text = report::status(&crate::store::load(dir.path()), testkit::date("2026-08-10"));
    assert!(text.contains("decided"), "{text}");
}

#[test]
fn an_unallocated_add_is_the_one_sanctioned_pendency() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let applied = a
        .add(AddArgs {
            set: "verbs".into(),
            statement: "Which retry budget applies to exports?".into(),
            namespace: Some("fixture.verbs".into()),
            allocation: AllocationArgs::default(),
            tolerance_override: None,
            based_on: Vec::new(),
        revisit_if: Vec::new(),
            note: None,
        })
        .expect("unallocated add writes");
    assert!(applied.lines.iter().any(|l| l.contains("unallocated")), "{:?}", applied.lines);
    assert_eq!(gate(dir.path()), vec![VerifyClass::L001]);
}

#[test]
fn add_refuses_a_below_floor_override_with_the_gate_class() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let err = a
        .add(constraint_add("An override below the floor.", Some(Tier::T0)))
        .expect_err("below-floor override");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::L004), "{findings:?}");
    assert!(crate::store::load(dir.path()).log.is_empty(), "nothing was written");
}

#[test]
fn allocate_files_a_new_version_with_the_parent_linked() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    a.allocate(
        &id,
        AllocationArgs {
            store: Some(AllocationKind::Judgment),
            actor: Some(testkit::identity("fixture-human@example")),
            ..AllocationArgs::default()
        },
    )
    .expect("allocate");
    let store = crate::store::load(dir.path());
    let view = crate::verify::view::View::build(&store);
    let latest = view.latest.get(&id.to_string()).and_then(|i| view.versions.get(*i)).expect("tip");
    assert!(latest.raw.parent.is_some(), "the revision links its parent");
    assert_eq!(gate(dir.path()), Vec::<VerifyClass>::new());
}

#[test]
fn allocate_refuses_a_model_judgment_with_l010() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    let err = a
        .allocate(
            &id,
            AllocationArgs {
                store: Some(AllocationKind::Judgment),
                actor: Some(testkit::identity("claude@example.com")),
                ..AllocationArgs::default()
            },
        )
        .expect_err("model judgment");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::L010), "{findings:?}");
}

#[test]
fn escape_prices_with_the_author_as_acceptor_and_refuses_a_model_author() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    a.escape(&id, EscapeArgs { exposure: "exports may drift".into(), review_by: testkit::date("2026-09-01") })
        .expect("escape");
    assert_eq!(gate(dir.path()), Vec::<VerifyClass>::new());

    // The same act as a model identity is refused with the gate's class.
    let mut model = Author::new(
        dir.path(),
        testkit::identity("claude@example.com"),
        now(),
        UlidMint::fixed(1_755_000_100_000),
    );
    let second = add_constraint(&mut model);
    let err = model
        .escape(&second, EscapeArgs { exposure: "x".into(), review_by: testkit::date("2026-09-01") })
        .expect_err("model escape acceptor");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::L006), "{findings:?}");
}

#[test]
fn revise_moves_the_hash_and_a_stale_parent_is_a_conflict_not_a_merge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    let store = crate::store::load(dir.path());
    let view = crate::verify::view::View::build(&store);
    let first_hash =
        view.latest.get(&id.to_string()).and_then(|i| view.versions.get(*i)).expect("tip").raw.hash.clone();

    a.revise(&id, ReviseArgs {
        statement: "Monetary amounts use decimal everywhere, including imports.".into(),
        based_on: Vec::new(),
        revisit_if: None,
        note: None,
        expected_parent: Some(first_hash.clone()),
    })
    .expect("revise from the tip");

    let err = a
        .revise(&id, ReviseArgs {
            statement: "A second fork from the same parent.".into(),
            based_on: Vec::new(),
        revisit_if: None,
        note: None,
            expected_parent: Some(first_hash),
        })
        .expect_err("stale parent");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");
    let AuthorError::Conflict(message) = err else { unreachable!() };
    assert!(message.contains("merge is L3"), "{message}");
}

#[path = "sign_tests.rs"]
mod sign_tests;
