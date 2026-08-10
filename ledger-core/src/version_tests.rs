//! Version cases: the wire-to-domain gate, plus what it classifies.

use crate::finding::VerifyClass;
use crate::testkit;
use crate::tier::Tier;

use super::*;

#[test]
fn a_well_formed_version_assembles() {
    let v = DecisionVersion::assemble(&testkit::version()).expect("assemble");
    assert_eq!(v.set, "ledger-design");
    assert_eq!(v.tolerance.effective(), Tier::T1);
    assert!(v.allocation.is_some());
    assert_eq!(v.based_on.len(), 1);
}

#[test]
fn a_below_floor_override_is_class_l004_not_a_generic_schema_fault() {
    // §4.2.1 requires it rejected at write; the same path must name it when
    // it is hand-edited into a file.
    let mut raw = testkit::version();
    raw.tolerance_floor_at_creation = Tier::T2;
    raw.tolerance_override = Some(Tier::T0);
    let f = DecisionVersion::assemble(&raw).expect_err("below floor");
    assert_eq!(f.class, VerifyClass::L004);
    assert!(f.message.contains("is below the set floor T2"), "{f}");
}

#[test]
fn an_unpriced_escape_is_class_l002() {
    let mut raw = testkit::version();
    raw.allocation = Some(crate::allocation::AllocationKind::Escaped);
    raw.discharge.clear();
    raw.exposure = Some("downstream may break".into());
    let f = DecisionVersion::assemble(&raw).expect_err("unpriced");
    assert_eq!(f.class, VerifyClass::L002);
}

#[test]
fn a_version_with_no_allocation_assembles_and_stays_unallocated() {
    // Enumerated-but-unallocated is a real intermediate state; the gate
    // fails on it as L001, but the file is not malformed.
    let mut raw = testkit::version();
    raw.allocation = None;
    raw.discharge.clear();
    let v = DecisionVersion::assemble(&raw).expect("assemble");
    assert!(v.allocation.is_none());
}

#[test]
fn an_empty_statement_is_a_schema_fault() {
    let mut raw = testkit::version();
    raw.statement = "   ".into();
    let f = DecisionVersion::assemble(&raw).expect_err("empty");
    assert_eq!(f.class, VerifyClass::Schema);
    assert!(f.message.contains("`statement` is empty"), "{f}");
}

#[test]
fn strings_are_trimmed_on_the_way_into_the_domain_form() {
    let mut raw = testkit::version();
    raw.statement = "  spaced out  ".into();
    raw.set = " ledger-design ".into();
    let v = DecisionVersion::assemble(&raw).expect("assemble");
    assert_eq!(v.statement, "spaced out");
    assert_eq!(v.set, "ledger-design");
}

#[test]
fn a_version_round_trips_through_yaml() {
    let sealed = testkit::sealed(testkit::version());
    let text = serde_yaml::to_string(&sealed).expect("serialize");
    let back: VersionRaw = serde_yaml::from_str(&text).expect("parse");
    assert_eq!(back, sealed);
}

#[test]
fn absent_optionals_are_not_written_back_as_nulls() {
    let text = serde_yaml::to_string(&testkit::version()).expect("serialize");
    assert!(!text.contains("null"), "{text}");
    assert!(!text.contains("supersedes"), "{text}");
}

#[test]
fn an_unknown_key_is_rejected_rather_than_silently_dropped() {
    let mut text = serde_yaml::to_string(&testkit::version()).expect("serialize");
    text.push_str("tolerance: T0\n");
    let err = serde_yaml::from_str::<VersionRaw>(&text).expect_err("unknown field");
    assert!(err.to_string().contains("unknown field"), "{err}");
}

#[test]
fn a_basis_pointer_is_a_single_token() {
    assert!("prd:decision-ledger-prd#4.2.1".parse::<BasisRef>().is_ok());
    assert!("DDD-ledger-01".parse::<BasisRef>().is_ok());
    assert!("see the PRD".parse::<BasisRef>().is_err());
    assert!("   ".parse::<BasisRef>().is_err());
}
