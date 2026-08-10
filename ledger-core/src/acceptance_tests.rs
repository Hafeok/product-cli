//! Acceptance cases: scope forms, expiry, the reserved signature field.

use crate::testkit;

use super::*;

fn acceptance() -> Acceptance {
    testkit::acceptance(&testkit::sealed(testkit::version()))
}

#[test]
fn the_default_scope_is_the_version_it_signs() {
    let a = acceptance();
    assert_eq!(a.scope, AcceptanceScope::Version);
    assert!(a.schema_faults().is_empty());
}

#[test]
fn a_class_scope_carries_a_real_discharge_pointer() {
    let scope: AcceptanceScope = "class:analyzer:DEC001".parse().expect("parse");
    assert_eq!(scope.to_string(), "class:analyzer:DEC001");
    assert!("class:nonsense".parse::<AcceptanceScope>().is_err());
    assert!("whenever".parse::<AcceptanceScope>().is_err());
}

#[test]
fn expiry_is_judged_against_a_supplied_date() {
    let a = acceptance();
    assert!(!a.is_expired(testkit::date("2027-08-10")), "the expiry date itself is still live");
    assert!(a.is_expired(testkit::date("2027-08-11")));
}

#[test]
fn an_acceptance_with_no_expiry_never_goes_stale() {
    // The risk OD-6 exists to settle; recorded here so the behaviour is
    // deliberate rather than an oversight.
    let mut a = acceptance();
    a.expires_at = None;
    assert!(!a.is_expired(testkit::date("2099-01-01")));
}

#[test]
fn a_non_empty_signature_is_a_schema_fault_under_format_one() {
    let mut a = acceptance();
    a.signature = "-----BEGIN PGP SIGNATURE-----".into();
    let faults = a.schema_faults();
    assert_eq!(faults.len(), 1);
    assert!(faults[0].message.contains("reserved"), "{}", faults[0]);
}

#[test]
fn an_acceptance_round_trips_through_yaml() {
    let a = acceptance();
    let text = serde_yaml::to_string(&a).expect("serialize");
    assert!(!text.contains("signature"), "an empty reserved field is not written: {text}");
    assert_eq!(serde_yaml::from_str::<Acceptance>(&text).expect("parse"), a);
}

#[test]
fn a_revocation_names_the_acceptance_it_reverses_with_a_reason() {
    let text = format!(
        "acceptance: {}\nat: 2026-08-11T09:00:00Z\nby: fixture-human@example\nreason: filed against the wrong version\n",
        testkit::acceptance_id()
    );
    let r: Revocation = serde_yaml::from_str(&text).expect("parse");
    assert_eq!(r.acceptance, testkit::acceptance_id());
    assert!(!r.reason.is_empty());
}

#[test]
fn a_revocation_without_a_reason_does_not_parse() {
    let text = format!(
        "acceptance: {}\nat: 2026-08-11T09:00:00Z\nby: fixture-human@example\n",
        testkit::acceptance_id()
    );
    assert!(serde_yaml::from_str::<Revocation>(&text).is_err(), "a reversal states why");
}
