use super::*;
use crate::closure::ClosureKind;

pub(crate) fn opening(slice: &str) -> Opening {
    Opening {
        id: "01K5CJ7Q3S8XN2VYB4M6E9TZRA".into(),
        act: "implement".into(),
        slice: slice.into(),
        act_ref: "act/settle-basket".into(),
        opened_at: DateTime::parse_from_rfc3339("2026-09-15T09:14:02Z")
            .map(|t| t.with_timezone(&Utc))
            .expect("fixed timestamp parses"),
        opened_by: "agent@example.invalid".parse().expect("identity parses"),
        base_revision: "9f3c1d0e".into(),
    }
}

#[test]
fn a_fresh_record_is_open() {
    let record = ActRecord::open(opening("checkout-totals"));
    assert!(record.is_open());
    assert_eq!(record.form, RECORD_FORM);
}

#[test]
fn the_opening_digest_seals_at_open() {
    let record = ActRecord::open(opening("checkout-totals"));
    assert_eq!(record.binds, record.computed_binds());
    assert!(record.binds.starts_with("sha256:"));
}

#[test]
fn a_different_slice_is_a_different_opening() {
    let a = ActRecord::open(opening("checkout-totals"));
    let b = ActRecord::open(opening("checkout-shipping"));
    assert_ne!(a.binds, b.binds);
}

#[test]
fn editing_an_opened_record_breaks_its_binding() {
    let mut record = ActRecord::open(opening("checkout-totals"));
    record.slice = "something-else".into();
    assert_ne!(record.binds, record.computed_binds());
}

#[test]
fn a_closure_does_not_change_the_opening_digest() {
    let mut record = ActRecord::open(opening("checkout-totals"));
    let sealed = record.binds.clone();
    record.closure = Some(Closure {
        kind: ClosureKind::NothingArose,
        principal: "emil@example.com".parse().expect("identity parses"),
        at: Utc::now(),
        determinations: Vec::new(),
        binds: "sha256:00".into(),
        signature: None,
    });
    assert_eq!(record.computed_binds(), sealed);
}
