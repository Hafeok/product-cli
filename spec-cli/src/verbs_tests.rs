use super::record::{closure_kind, CloseArgs};
use super::resolve_identity;
use spec_core::closure::ClosureKind;

fn close_args(nothing_arose: bool, dets: &[&str]) -> CloseArgs {
    CloseArgs {
        id: "01K5CJ7Q3S8XN2VYB4M6E9TZRA".into(),
        principal: None,
        determinations: dets.iter().map(|s| (*s).to_string()).collect(),
        nothing_arose,
        json: false,
    }
}

#[test]
fn nothing_arose_alone_is_a_declaration() {
    assert_eq!(closure_kind(&close_args(true, &[])).ok(), Some(ClosureKind::NothingArose));
}

#[test]
fn determinations_alone_is_a_declaration() {
    assert_eq!(
        closure_kind(&close_args(false, &["det/a"])).ok(),
        Some(ClosureKind::Determinations)
    );
}

#[test]
fn silence_closes_nothing() {
    let err = closure_kind(&close_args(false, &[])).expect_err("silence is not a closure");
    assert!(err.to_string().contains("a closure declares something"), "{err}");
}

#[test]
fn nothing_arose_cannot_carry_determinations() {
    let err = closure_kind(&close_args(true, &["det/a"])).expect_err("contradiction");
    assert!(err.to_string().contains("contradicts"), "{err}");
}

#[test]
fn an_explicit_identity_wins_over_git() {
    let resolved = resolve_identity(std::path::Path::new("."), Some("emil@example.com"))
        .expect("explicit identity parses");
    assert_eq!(resolved.as_str(), "emil@example.com");
}

#[test]
fn a_malformed_identity_is_refused_rather_than_normalised() {
    let err = resolve_identity(std::path::Path::new("."), Some("not-an-address"))
        .expect_err("an identity resolves by email");
    assert!(err.to_string().contains("not-an-address"), "{err}");
}

#[test]
fn an_act_address_derives_from_the_name_a_principal_typed() {
    assert_eq!(super::ratify::slugify("Settle a basket"), "settle-a-basket");
    assert_eq!(super::ratify::slugify("  "), "unnamed");
}
