//! Allocation cases: §4.4's per-store obligations, plus the stray-field gate.

use super::*;

fn date(s: &str) -> NaiveDate {
    s.parse().expect("date")
}

fn id(s: &str) -> Identity {
    s.parse().expect("identity")
}

fn dref(s: &str) -> DischargeRef {
    s.parse().expect("discharge")
}

#[test]
fn a_constraint_takes_its_discharge_pointers() {
    let f = AllocationFields { discharge: vec![dref("analyzer:DEC001")], ..Default::default() };
    let a = Allocation::assemble(AllocationKind::Constraint, f).expect("assemble");
    assert_eq!(a.kind(), AllocationKind::Constraint);
    assert_eq!(a.discharge().len(), 1);
}

#[test]
fn a_criterion_requires_a_discharge_ref_and_a_stage() {
    let bare = Allocation::assemble(AllocationKind::Criterion, AllocationFields::default())
        .expect_err("no discharge");
    assert!(bare.message.contains("requires `discharge`"), "{bare}");

    let no_stage = AllocationFields { discharge: vec![dref("test:T")], ..Default::default() };
    let err = Allocation::assemble(AllocationKind::Criterion, no_stage).expect_err("no stage");
    assert!(err.message.contains("requires `discharge_stage`"), "{err}");

    let ok = AllocationFields {
        discharge: vec![dref("test:T")],
        discharge_stage: Some(Stage::Pr),
        ..Default::default()
    };
    assert!(Allocation::assemble(AllocationKind::Criterion, ok).is_ok());
}

#[test]
fn an_otel_discharge_requires_a_stated_expectation() {
    let f = AllocationFields {
        discharge: vec![dref("otel:dec.004.deadletter")],
        discharge_stage: Some(Stage::Prod),
        ..Default::default()
    };
    let err = Allocation::assemble(AllocationKind::Criterion, f.clone()).expect_err("unstated");
    assert!(err.message.contains("unfalsifiable"), "{err}");

    let stated = AllocationFields {
        expectation: Some("dead-letter rate < 0.5% over 14d".into()),
        ..f
    };
    assert!(Allocation::assemble(AllocationKind::Criterion, stated).is_ok());
}

#[test]
fn a_judgment_requires_a_named_actor() {
    let err = Allocation::assemble(AllocationKind::Judgment, AllocationFields::default())
        .expect_err("no actor");
    assert!(err.message.contains("requires `actor`"), "{err}");

    let f = AllocationFields { actor: Some(id("emk@delegate.dk")), ..Default::default() };
    assert!(Allocation::assemble(AllocationKind::Judgment, f).is_ok());
}

#[test]
fn an_escape_requires_exposure_acceptor_and_review_date() {
    let full = AllocationFields {
        exposure: Some("Downstream consumer may break silently.".into()),
        accepted_by: Some(id("emk@delegate.dk")),
        review_by: Some(date("2026-09-01")),
        ..Default::default()
    };
    assert!(Allocation::assemble(AllocationKind::Escaped, full.clone()).is_ok());

    for drop in ["exposure", "accepted_by", "review_by"] {
        let mut f = full.clone();
        match drop {
            "exposure" => f.exposure = None,
            "accepted_by" => f.accepted_by = None,
            _ => f.review_by = None,
        }
        let err = Allocation::assemble(AllocationKind::Escaped, f).expect_err("incomplete");
        assert!(err.message.contains(drop), "dropping {drop} should be named: {err}");
        // An unpriced escape is one of the nine, not a parse-gate fault:
        // silent escape is the failure the tool exists to catch.
        assert_eq!(err.class, VerifyClass::L002, "dropping {drop}");
    }
}

#[test]
fn the_other_stores_obligations_are_parse_gate_faults() {
    let err = Allocation::assemble(AllocationKind::Judgment, AllocationFields::default())
        .expect_err("no actor");
    assert_eq!(err.class, VerifyClass::Schema);
}

#[test]
fn the_escape_acceptor_is_reachable_for_the_model_identity_rule() {
    let f = AllocationFields {
        exposure: Some("x".into()),
        accepted_by: Some(id("emk@delegate.dk")),
        review_by: Some(date("2026-09-01")),
        ..Default::default()
    };
    let a = Allocation::assemble(AllocationKind::Escaped, f).expect("assemble");
    assert_eq!(a.escape_acceptor().map(Identity::as_str), Some("emk@delegate.dk"));
    let c = Allocation::assemble(AllocationKind::Constraint, AllocationFields::default())
        .expect("assemble");
    assert!(c.escape_acceptor().is_none());
}

#[test]
fn a_field_the_store_cannot_read_is_a_fault_not_a_silent_ignore() {
    let f = AllocationFields {
        discharge: vec![dref("analyzer:DEC001")],
        exposure: Some("this belongs to an escape".into()),
        ..Default::default()
    };
    let err = Allocation::assemble(AllocationKind::Constraint, f).expect_err("stray");
    assert!(err.message.contains("does not read exposure"), "{err}");
    assert!(err.message.contains("governance that is not there"), "{err}");
}

#[test]
fn a_judgment_carrying_a_discharge_ref_is_a_stray() {
    let f = AllocationFields {
        actor: Some(id("emk@delegate.dk")),
        discharge: vec![dref("test:T")],
        ..Default::default()
    };
    let err = Allocation::assemble(AllocationKind::Judgment, f).expect_err("stray");
    assert!(err.message.contains("does not read discharge"), "{err}");
}
