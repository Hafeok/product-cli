use super::*;
use crate::closure::ClosureKind;
use crate::record::tests::opening;
use chrono::Utc;

fn closed_with(principal: &str, kind: ClosureKind, dets: &[&str]) -> ActRecord {
    let mut record = ActRecord::open(opening("checkout-totals"));
    let mut closure = Closure {
        kind,
        principal: principal.parse().expect("identity parses"),
        at: Utc::now(),
        determinations: dets.iter().map(|s| (*s).to_string()).collect(),
        binds: String::new(),
    };
    closure.binds = closure_digest(&record.computed_binds(), &closure);
    record.closure = Some(closure);
    record
}

fn classes(record: &ActRecord) -> Vec<Class> {
    judge(record).into_iter().map(|f| f.class).collect()
}

#[test]
fn s001_an_open_record_fails() {
    let record = ActRecord::open(opening("checkout-totals"));
    assert_eq!(classes(&record), vec![Class::S001]);
}

#[test]
fn a_well_formed_closure_passes() {
    let record = closed_with("emil@example.com", ClosureKind::NothingArose, &[]);
    assert!(judge(&record).is_empty(), "{:?}", judge(&record));
}

#[test]
fn s002_a_machine_principal_cannot_close() {
    for machine in ["ci@example.com", "noreply@example.com", "claude@example.com"] {
        let record = closed_with(machine, ClosureKind::NothingArose, &[]);
        assert!(
            classes(&record).contains(&Class::S002),
            "{machine} should be refused as a closer"
        );
    }
}

#[test]
fn s003_an_edited_opening_breaks_the_closure_binding() {
    let mut record = closed_with("emil@example.com", ClosureKind::NothingArose, &[]);
    record.slice = "a-different-slice".into();
    assert!(classes(&record).contains(&Class::S003));
}

#[test]
fn s004_determinations_with_nothing_filed_fails() {
    let record = closed_with("emil@example.com", ClosureKind::Determinations, &[]);
    assert!(classes(&record).contains(&Class::S004));
}

#[test]
fn s004_nothing_arose_with_a_list_fails() {
    let record = closed_with("emil@example.com", ClosureKind::NothingArose, &["det/a"]);
    assert!(classes(&record).contains(&Class::S004));
}

#[test]
fn a_determinations_closure_that_files_something_passes() {
    let record = closed_with("emil@example.com", ClosureKind::Determinations, &["det/a"]);
    assert!(judge(&record).is_empty(), "{:?}", judge(&record));
}

#[test]
fn judge_store_reports_every_record() {
    let open_one = ActRecord::open(opening("a"));
    let closed = closed_with("emil@example.com", ClosureKind::NothingArose, &[]);
    let findings = judge_store(&[open_one, closed]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].class, Class::S001);
}

#[test]
fn a_finding_renders_its_class_and_record() {
    let record = ActRecord::open(opening("checkout-totals"));
    let rendered = judge(&record)[0].to_string();
    assert!(rendered.starts_with("S001: 01K5CJ7Q3S8XN2VYB4M6E9TZRA — "), "{rendered}");
}
