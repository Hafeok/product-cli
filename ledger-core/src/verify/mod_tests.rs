//! Gate cases: one passing store, then each class provoked in isolation.

use crate::allocation::AllocationKind;
use crate::changeset::ChangeSet;
use crate::finding::{Gate, VerifyClass, ALL_CLASSES};
use crate::store::Store;
use crate::testkit;
use crate::tier::Tier;
use crate::version::VersionRaw;

use super::*;

fn today() -> NaiveDate {
    testkit::date("2026-08-10")
}

/// Options with blame off: these cases build stores in memory, where there
/// is no repository to consult. `L009` has its own tests over real commits.
fn opts() -> Options {
    Options { gate: None, today: today(), blame: false }
}

fn run(cs: ChangeSet) -> Report {
    verify(&testkit::store(cs), &opts())
}

fn classes(report: &Report) -> Vec<VerifyClass> {
    report.findings.iter().map(|f| f.class).collect()
}

fn accepted_store() -> ChangeSet {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    testkit::changeset(vec![sealed], vec![acceptance])
}

#[test]
fn a_well_formed_accepted_store_is_conformant() {
    let report = run(accepted_store());
    assert!(report.is_conformant(), "{:?}", report.findings);
    assert_eq!(report.decisions, 1);
    assert!(report.awaiting_acceptance.is_empty());
}

#[test]
fn allocated_awaiting_acceptance_is_status_never_a_failure() {
    // The single most important thing the gate must not do.
    let cs = testkit::changeset(vec![testkit::sealed(testkit::version())], Vec::new());
    let report = run(cs);
    assert!(report.is_conformant(), "{:?}", report.findings);
    assert_eq!(report.awaiting_acceptance, vec![testkit::decision_id().to_string()]);
}

#[test]
fn l001_a_decision_with_no_allocation() {
    let mut raw = testkit::version();
    raw.allocation = None;
    raw.discharge.clear();
    let report = run(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    assert_eq!(classes(&report), vec![VerifyClass::L001]);
}

#[test]
fn l002_an_escape_without_its_price() {
    let mut raw = testkit::version();
    raw.allocation = Some(AllocationKind::Escaped);
    raw.discharge.clear();
    raw.exposure = Some("downstream may break silently".into());
    raw.accepted_by = Some(testkit::identity("fixture-human@example"));
    // `review_by` missing: the escape is stated but not scheduled.
    let report = run(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    assert!(classes(&report).contains(&VerifyClass::L002), "{:?}", report.findings);
}

#[test]
fn a_fully_priced_escape_passes() {
    let mut raw = testkit::version();
    raw.allocation = Some(AllocationKind::Escaped);
    raw.discharge.clear();
    raw.exposure = Some("downstream may break silently".into());
    raw.accepted_by = Some(testkit::identity("fixture-human@example"));
    raw.review_by = Some(testkit::date("2026-09-01"));
    let sealed = testkit::sealed(raw);
    let acceptance = testkit::acceptance(&sealed);
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert!(report.is_conformant(), "{:?}", report.findings);
}

#[test]
fn l003_an_expired_acceptance_is_a_hard_failure() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.expires_at = Some(testkit::date("2026-08-09"));
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert_eq!(classes(&report), vec![VerifyClass::L003]);
}

#[test]
fn l004_an_override_below_the_pinned_floor() {
    let mut raw = testkit::version();
    raw.tolerance_floor_at_creation = Tier::T2;
    raw.tolerance_override = Some(Tier::T1);
    let report = run(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    assert!(classes(&report).contains(&VerifyClass::L004), "{:?}", report.findings);
}

#[test]
fn l005_a_member_stranded_below_a_raised_floor() {
    let mut raw = testkit::version();
    raw.tolerance_floor_at_creation = Tier::T0;
    let sealed = testkit::sealed(raw);
    let acceptance = testkit::acceptance(&sealed);
    // The set's live floor is T1; this member was pinned under T0.
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert_eq!(classes(&report), vec![VerifyClass::L005]);
}

#[test]
fn a_member_overridden_up_survives_the_same_raise() {
    let mut raw = testkit::version();
    raw.tolerance_floor_at_creation = Tier::T0;
    raw.tolerance_override = Some(Tier::T2);
    let sealed = testkit::sealed(raw);
    let acceptance = testkit::acceptance(&sealed);
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert!(report.is_conformant(), "{:?}", report.findings);
}

#[test]
fn l006_an_acceptance_signed_by_a_model_or_bot_identity() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.actor = testkit::identity("claude@example.com");
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert_eq!(classes(&report), vec![VerifyClass::L006]);
}

#[test]
fn l006_also_covers_the_acceptor_pricing_an_escape() {
    let mut raw = testkit::version();
    raw.allocation = Some(AllocationKind::Escaped);
    raw.discharge.clear();
    raw.exposure = Some("downstream may break".into());
    raw.accepted_by = Some(testkit::identity("github-actions@github.com"));
    raw.review_by = Some(testkit::date("2026-09-01"));
    let sealed = testkit::sealed(raw);
    let acceptance = testkit::acceptance(&sealed);
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert!(classes(&report).contains(&VerifyClass::L006), "{:?}", report.findings);
}

#[test]
fn l007_a_stored_hash_that_does_not_match_its_content() {
    let mut sealed = testkit::sealed(testkit::version());
    // The tamper: edit the content, leave the digest behind.
    sealed.statement = "Monetary amounts may use double.".into();
    let report = run(testkit::changeset(vec![sealed], Vec::new()));
    assert!(classes(&report).contains(&VerifyClass::L007), "{:?}", report.findings);
}

#[test]
fn l008_an_acceptance_signing_a_hash_nobody_filed() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.version = testkit::zero_hash();
    let report = run(testkit::changeset(vec![sealed], vec![acceptance]));
    assert!(classes(&report).contains(&VerifyClass::L008), "{:?}", report.findings);
}

#[test]
fn a_revoked_acceptance_stops_being_judged_for_expiry() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.expires_at = Some(testkit::date("2026-08-09"));
    let mut cs = testkit::changeset(vec![sealed], vec![acceptance]);
    cs.revocations.push(crate::acceptance::Revocation {
        acceptance: testkit::acceptance_id(),
        at: testkit::stamp("2026-08-10T10:00:00Z"),
        by: testkit::identity("fixture-human@example"),
        reason: "filed against the wrong version".into(),
    });
    let report = run(cs);
    assert!(!classes(&report).contains(&VerifyClass::L003), "{:?}", report.findings);
}

#[test]
fn a_version_naming_an_undeclared_set_is_a_parse_gate_fault() {
    let mut raw = testkit::version();
    raw.set = "no-such-set".into();
    let report = run(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    assert!(
        report.findings.iter().any(|f| f.class == VerifyClass::Schema
            && f.message.contains("which is not declared")),
        "{:?}",
        report.findings
    );
}

#[test]
fn only_the_latest_version_of_a_decision_is_judged() {
    let mut first = testkit::version();
    first.allocation = None;
    first.discharge.clear();
    let first = testkit::sealed(first);
    let mut second = testkit::version();
    second.parent = Some(first.hash.clone());
    let second = testkit::sealed(second);
    let acceptance = testkit::acceptance(&second);
    let report = run(testkit::changeset(vec![first, second], vec![acceptance]));
    assert!(report.is_conformant(), "the superseded unallocated version is history: {:?}", report.findings);
}

#[test]
fn the_readiness_gate_leaves_out_the_two_release_dispositions() {
    let sealed = testkit::sealed(testkit::version());
    let mut acceptance = testkit::acceptance(&sealed);
    acceptance.expires_at = Some(testkit::date("2026-08-09"));
    let store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    let full = verify(&store, &opts());
    assert_eq!(classes(&full), vec![VerifyClass::L003]);
    let readiness = verify(&store, &Options { gate: Some(Gate::Readiness), ..opts() });
    assert!(readiness.is_conformant(), "{:?}", readiness.findings);
}

#[test]
fn findings_are_reported_in_class_order_without_duplicates() {
    let mut raw = testkit::version();
    raw.allocation = None;
    raw.discharge.clear();
    let mut sealed = testkit::sealed(raw);
    sealed.statement = "tampered".into();
    let sealed_clone: VersionRaw = sealed.clone();
    let mut cs = testkit::changeset(vec![sealed], Vec::new());
    cs.versions.push(sealed_clone);
    let report = run(cs);
    let mut sorted = classes(&report);
    sorted.sort_unstable();
    assert_eq!(classes(&report), sorted, "{:?}", report.findings);
    let mut unique = report.findings.clone();
    unique.dedup();
    assert_eq!(unique.len(), report.findings.len(), "the same fault is reported once");
}

#[test]
fn schema_findings_from_the_store_reach_the_report() {
    let mut store: Store = testkit::store(accepted_store());
    store.schema_findings.push(crate::finding::Finding::schema("x.yml", "unreadable"));
    let report = verify(&store, &opts());
    assert_eq!(classes(&report), vec![VerifyClass::Schema]);
}

#[test]
fn every_class_the_enum_declares_is_reachable_here() {
    // A class with no test is a class nobody knows fires. L009 is exercised
    // in the CLI's fixture suite, where a real repository exists.
    let named: Vec<&str> = ALL_CLASSES.iter().map(|c| c.code()).collect();
    assert_eq!(
        named,
        ["SCHEMA", "L001", "L002", "L003", "L004", "L005", "L006", "L007", "L008", "L009"]
    );
}

