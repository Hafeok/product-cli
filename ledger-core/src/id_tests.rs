//! Identifier cases: ULID shape, namespace shape, round-trip through serde.

use super::*;

const ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXV";

#[test]
fn a_well_formed_decision_id_round_trips() {
    let id: DecisionId = format!("dec:hafeok.ledger/{ULID}").parse().expect("parse");
    assert_eq!(id.namespace(), "hafeok.ledger");
    assert_eq!(id.ulid(), ULID);
    assert_eq!(id.to_string(), format!("dec:hafeok.ledger/{ULID}"));
}

#[test]
fn the_scheme_is_required() {
    let err = format!("hafeok.ledger/{ULID}").parse::<DecisionId>().expect_err("no scheme");
    assert!(err.contains("expected `dec:<namespace>/<ulid>`"), "{err}");
}

#[test]
fn the_excluded_crockford_letters_are_rejected() {
    // I, L, O and U are excluded so a transcribed id cannot be read two ways.
    for bad in ['I', 'L', 'O', 'U'] {
        let ulid = format!("{bad}{}", &ULID[1..]);
        assert!(validate_ulid(&ulid).is_err(), "{bad} should be outside the alphabet");
    }
}

#[test]
fn a_ulid_of_the_wrong_length_names_the_length() {
    let err = validate_ulid("01K2C4").expect_err("too short");
    assert!(err.contains("is 6 characters"), "{err}");
}

#[test]
fn a_first_character_above_seven_overflows_the_timestamp() {
    let err = validate_ulid(&format!("8{}", &ULID[1..])).expect_err("overflow");
    assert!(err.contains("overflows"), "{err}");
}

#[test]
fn namespaces_are_lowercase_dot_separated_scopes() {
    assert!(format!("dec:Hafeok.Ledger/{ULID}").parse::<DecisionId>().is_err());
    assert!(format!("dec:hafeok..ledger/{ULID}").parse::<DecisionId>().is_err());
    assert!(format!("dec:hafeok-org.ledger2/{ULID}").parse::<DecisionId>().is_ok());
}

#[test]
fn a_namespace_is_not_optional() {
    let err = format!("dec:{ULID}").parse::<DecisionId>().expect_err("no slash");
    assert!(err.contains("no `/`"), "{err}");
}

#[test]
fn change_set_and_acceptance_ids_carry_their_own_schemes() {
    let cs: ChangeSetId = format!("cs:{ULID}").parse().expect("parse");
    assert_eq!(cs.to_string(), format!("cs:{ULID}"));
    let acc: AcceptanceId = format!("acc:{ULID}").parse().expect("parse");
    assert_eq!(acc.to_string(), format!("acc:{ULID}"));
    assert!(format!("acc:{ULID}").parse::<ChangeSetId>().is_err(), "schemes do not cross");
}

#[test]
fn ids_serialise_as_their_rendered_string() {
    let id: DecisionId = format!("dec:hafeok.ledger/{ULID}").parse().expect("parse");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, format!("\"dec:hafeok.ledger/{ULID}\""));
    let back: DecisionId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, id);
}

#[test]
fn ids_sort_by_creation_time_because_ulids_do() {
    let earlier: ChangeSetId = "cs:01K2C4YQJ3F8M0PT5W7NZ9RDXV".parse().expect("parse");
    let later: ChangeSetId = "cs:01K2C5YQJ3F8M0PT5W7NZ9RDXV".parse().expect("parse");
    assert!(earlier < later);
}
