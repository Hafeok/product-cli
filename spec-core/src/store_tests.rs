use super::*;
use crate::check::Class;
use crate::record::tests::opening;

fn closing(principal: &str, kind: ClosureKind, dets: &[&str]) -> Closing {
    Closing {
        kind,
        principal: principal.parse().expect("identity parses"),
        at: Utc::now(),
        determinations: dets.iter().map(|s| (*s).to_string()).collect(),
    }
}

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn an_opened_record_lands_on_disk_and_reads_back() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    let reloaded = load(&record_path(dir.path(), &opened.id)).expect("load");
    assert_eq!(reloaded, opened);
    assert!(reloaded.is_open());
}

#[test]
fn closing_seals_a_binding_the_gate_accepts() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    let Closed::Sealed(closed) = close(
        dir.path(),
        &opened.id,
        closing("emil@example.com", ClosureKind::NothingArose, &[]),
    )
    .expect("close") else {
        panic!("a well-formed closure must seal");
    };
    assert!(!closed.is_open());
    assert!(check::judge(&closed).is_empty());
}

#[test]
fn close_refuses_a_machine_principal_with_the_gate_class() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    let Closed::Refused(findings) = close(
        dir.path(),
        &opened.id,
        closing("ci@example.com", ClosureKind::NothingArose, &[]),
    )
    .expect("the gate answers rather than erroring") else {
        panic!("a machine must not close a record");
    };
    assert_eq!(findings.first().map(|f| f.class), Some(Class::S002));
}

#[test]
fn a_refused_close_leaves_the_record_open_on_disk() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    let _ = close(
        dir.path(),
        &opened.id,
        closing("ci@example.com", ClosureKind::NothingArose, &[]),
    );
    let reloaded = load(&record_path(dir.path(), &opened.id)).expect("load");
    assert!(reloaded.is_open(), "the refused write must not have landed");
}

#[test]
fn close_refuses_a_kind_that_disagrees_with_its_payload() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    let Closed::Refused(findings) = close(
        dir.path(),
        &opened.id,
        closing("emil@example.com", ClosureKind::Determinations, &[]),
    )
    .expect("the gate answers rather than erroring") else {
        panic!("determinations with nothing filed must be refused");
    };
    assert_eq!(findings.first().map(|f| f.class), Some(Class::S004));
}

#[test]
fn a_closed_record_cannot_be_closed_again() {
    let dir = repo();
    let opened = open(dir.path(), opening("checkout-totals")).expect("open");
    close(dir.path(), &opened.id, closing("emil@example.com", ClosureKind::NothingArose, &[]))
        .expect("first close");
    let err = close(
        dir.path(),
        &opened.id,
        closing("emil@example.com", ClosureKind::NothingArose, &[]),
    )
    .expect_err("a correction is a new record");
    assert!(err.to_string().contains("already closed"), "{err}");
}

#[test]
fn an_empty_store_has_nothing_to_judge() {
    let dir = repo();
    assert!(load_all(dir.path()).expect("load_all").is_empty());
}

#[test]
fn load_all_finds_every_record_and_the_gate_sees_the_open_one() {
    let dir = repo();
    let mut first = opening("a");
    first.id = "01K5CJ7Q3S8XN2VYB4M6E9TZR1".into();
    let mut second = opening("b");
    second.id = "01K5CJ7Q3S8XN2VYB4M6E9TZR2".into();
    open(dir.path(), first).expect("open a");
    let b = open(dir.path(), second).expect("open b");
    close(dir.path(), &b.id, closing("emil@example.com", ClosureKind::NothingArose, &[]))
        .expect("close b");

    let all = load_all(dir.path()).expect("load_all");
    assert_eq!(all.len(), 2);
    let findings = check::judge_store(&all);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].class, Class::S001);
}
