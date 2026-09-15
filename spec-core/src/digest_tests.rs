use super::*;
use crate::closure::ClosureKind;
use crate::record::tests::opening;
use crate::record::ActRecord;
use chrono::{DateTime, Utc};

fn fixed_closure(kind: ClosureKind, dets: &[&str]) -> Closure {
    Closure {
        kind,
        principal: "emil@example.com".parse().expect("identity parses"),
        at: DateTime::parse_from_rfc3339("2026-09-16T08:30:11Z")
            .map(|t| t.with_timezone(&Utc))
            .expect("fixed timestamp parses"),
        determinations: dets.iter().map(|s| (*s).to_string()).collect(),
        binds: String::new(),
        signature: None,
    }
}

#[test]
fn the_two_forms_are_separate_domains() {
    assert_ne!(RECORD_FORM, CLOSURE_FORM);
}

#[test]
fn a_closure_digest_covers_the_opening_it_discharges() {
    let closure = fixed_closure(ClosureKind::NothingArose, &[]);
    let one = closure_digest("sha256:aa", &closure);
    let other = closure_digest("sha256:bb", &closure);
    assert_ne!(one, other, "the same closure over a different opening");
}

#[test]
fn a_closure_digest_covers_its_principal() {
    let mine = fixed_closure(ClosureKind::NothingArose, &[]);
    let mut theirs = mine.clone();
    theirs.principal = "someone-else@example.com".parse().expect("identity parses");
    assert_ne!(closure_digest("sha256:aa", &mine), closure_digest("sha256:aa", &theirs));
}

#[test]
fn determination_order_is_formatting_not_meaning() {
    let one = fixed_closure(ClosureKind::Determinations, &["det/a", "det/b"]);
    let other = fixed_closure(ClosureKind::Determinations, &["det/b", "det/a"]);
    assert_eq!(closure_digest("sha256:aa", &one), closure_digest("sha256:aa", &other));
}

#[test]
fn a_record_digest_is_stable_across_recomputation() {
    let record = ActRecord::open(opening("checkout-totals"));
    assert_eq!(record_digest(&record), record_digest(&record));
}
