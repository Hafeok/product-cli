//! Discharge-pointer cases: every scheme, the unknown-scheme fault, ordering.

use super::*;

fn parse(s: &str) -> DischargeRef {
    s.parse().expect("parse")
}

#[test]
fn every_scheme_round_trips_through_its_string_form() {
    for s in [
        "analyzer:DEC001-no-float-money",
        "test:ExportIdempotencyTests",
        "policy:deny-public-blob",
        "whatif:no-public-network",
        "otel:dec.004.deadletter",
        "actor:emk@delegate.dk",
    ] {
        assert_eq!(parse(s).to_string(), s, "round trip");
    }
}

#[test]
fn an_unknown_scheme_names_the_schemes_that_exist() {
    let err = "doc:decision-ledger-prd#12".parse::<DischargeRef>().expect_err("unknown");
    assert!(err.contains("`doc` is not a discharge scheme"), "{err}");
    assert!(err.contains("analyzer | test | policy"), "{err}");
}

#[test]
fn a_pointer_without_a_scheme_is_a_fault() {
    let err = "DEC001".parse::<DischargeRef>().expect_err("no scheme");
    assert!(err.contains("has no scheme"), "{err}");
}

#[test]
fn an_empty_payload_is_a_fault() {
    let err = "analyzer:".parse::<DischargeRef>().expect_err("empty");
    assert!(err.contains("empty `analyzer` payload"), "{err}");
}

#[test]
fn an_actor_pointer_carries_a_real_identity() {
    assert!("actor:not-an-address".parse::<DischargeRef>().is_err());
    assert_eq!(parse("actor:EMK@Delegate.dk").to_string(), "actor:emk@delegate.dk");
}

#[test]
fn otel_pointers_are_flagged_because_they_oblige_an_expectation() {
    assert!(parse("otel:dec.004.deadletter").is_otel());
    assert!(!parse("test:RetryPolicyTests").is_otel());
}

#[test]
fn pointers_order_so_a_reordered_list_canonicalises_the_same() {
    let mut refs = [parse("test:B"), parse("analyzer:A"), parse("test:A")];
    refs.sort();
    let rendered: Vec<String> = refs.iter().map(ToString::to_string).collect();
    assert_eq!(rendered, ["analyzer:A", "test:A", "test:B"]);
}

#[test]
fn stages_order_by_the_ground_they_expose() {
    assert!(Stage::Pr < Stage::Dev && Stage::Dev < Stage::Staging && Stage::Staging < Stage::Prod);
    assert_eq!(Stage::Prod.to_string(), "prod");
    let parsed: Stage = serde_yaml::from_str("staging").expect("parse");
    assert_eq!(parsed, Stage::Staging);
}
