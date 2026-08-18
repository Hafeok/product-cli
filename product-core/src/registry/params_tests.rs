//! Unit tests for parameter validation.

use super::*;

fn params() -> RegistryParams {
    RegistryParams {
        owner: "Hafeok".into(),
        repo: "ground-registry-g0".into(),
        ratifier: "emil@okkels-klein.dk".into(),
        display_name: "Ground Registry (G0 validation)".into(),
        base_iri: "tag:emil@okkels-klein.dk,2026-08-17:ground/".into(),
        mint_date: "2026-08-17".into(),
        generated_by: "the registry generator".into(),
    }
}

#[test]
fn the_g0_parameters_validate() {
    assert!(params().validate().is_ok());
}

#[test]
fn sigils_in_values_are_not_rejected() {
    // Validation is about meaning, never about which characters a value carries.
    let mut p = params();
    p.ratifier = "a&b\\1$0@example.test".into();
    p.display_name = "R&D — \"the\" registry".into();
    assert!(p.validate().is_ok());
}

#[test]
fn an_empty_parameter_is_refused() {
    let mut p = params();
    p.generated_by = "  ".into();
    let err = p.validate().expect_err("empty parameter must be refused");
    assert!(err.to_string().contains("--generated-by"), "{err}");
}

#[test]
fn a_non_date_is_refused() {
    let mut p = params();
    p.mint_date = "17-08-2026".into();
    assert!(p.validate().is_err());
}

#[test]
fn a_base_iri_without_a_terminator_is_refused() {
    let mut p = params();
    p.base_iri = "https://example.test/ns".into();
    let err = p.validate().expect_err("must end in # or /");
    assert!(err.to_string().contains("--base-iri"), "{err}");
}

#[test]
fn a_base_iri_the_parser_rejects_is_refused() {
    let mut p = params();
    p.base_iri = "not an iri at all/".into();
    assert!(p.validate().is_err());
}

#[test]
fn a_repository_name_with_a_separator_is_refused() {
    let mut p = params();
    p.repo = "Hafeok/ground-registry-g0".into();
    assert!(p.validate().is_err());
}
