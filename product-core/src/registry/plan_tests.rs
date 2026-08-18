//! Unit tests for planning a generation.

use super::*;
use crate::registry::params::RegistryParams;

/// The parameters the discarded G0 instance was minted from, carried forward
/// verbatim from the ratified handoff.
fn g0_params() -> RegistryParams {
    RegistryParams {
        owner: "Hafeok".into(),
        repo: "ground-registry-g0".into(),
        ratifier: "emil@okkels-klein.dk".into(),
        display_name: "Ground Registry (G0 validation)".into(),
        base_iri: "tag:emil@okkels-klein.dk,2026-08-17:ground/".into(),
        mint_date: "2026-08-17".into(),
        generated_by: "product-cli registry generator".into(),
    }
}

fn plan() -> GenerationPlan {
    plan_generation(&g0_params(), "0.6.0").expect("the template renders under the gate")
}

fn file<'a>(p: &'a GenerationPlan, path: &str) -> &'a str {
    &p.files.iter().find(|f| f.path == path).expect("file in plan").contents
}

#[test]
fn the_template_renders_under_the_gate() {
    let p = plan();
    assert_eq!(p.files.len(), TEMPLATE.len());
    assert_eq!(p.template_version, TEMPLATE_VERSION);
}

#[test]
fn every_parameter_reaches_the_tree() {
    let p = plan();
    for label in ["--owner", "--repo", "--ratifier", "--display-name", "--base-iri", "--date", "--generated-by"] {
        assert!(p.gate.sites_per_label[label] >= 1, "{label} reached no file");
    }
}

#[test]
fn the_base_iri_is_byte_identical_in_the_output() {
    let p = plan();
    let generation = file(&p, "GENERATION.ttl");
    assert!(generation.contains("@prefix reg:  <tag:emil@okkels-klein.dk,2026-08-17:ground/> ."));
    assert!(generation.contains("reg:baseIri           \"tag:emil@okkels-klein.dk,2026-08-17:ground/\" ;"));
}

#[test]
fn template_md_travels_unsubstituted() {
    let p = plan();
    let embedded = TEMPLATE.iter().find(|f| f.path == "TEMPLATE.md").expect("TEMPLATE.md");
    assert_eq!(file(&p, "TEMPLATE.md"), embedded.text);
    assert!(file(&p, "TEMPLATE.md").contains("{{OWNER_ORG}}"), "the instance keeps its own record of the placeholders");
}

#[test]
fn no_placeholder_survives_a_substituted_file() {
    let p = plan();
    for f in p.files.iter().filter(|f| f.path != "TEMPLATE.md") {
        assert_eq!(crate::registry::verify::first_placeholder(&f.contents), None, "{}", f.path);
    }
}

#[test]
fn the_birth_provenance_records_both_versions() {
    let p = plan();
    let ttl = file(&p, "GENERATION.ttl");
    assert!(ttl.contains(&format!("reg:templateVersion   \"{TEMPLATE_VERSION}\"")));
    assert!(ttl.contains("reg:generatorVersion  \"0.6.0\""));
}

#[test]
fn a_value_carrying_sigils_survives_planning() {
    let mut params = g0_params();
    params.display_name = "R&D \\1 $0 — the {{OWNER_ORG}} registry".into();
    let p = plan_generation(&params, "0.6.0").expect("hostile values are data");
    assert!(file(&p, "GENERATION.ttl").contains("R&D \\1 $0 — the {{OWNER_ORG}} registry"));
}

#[test]
fn planning_writes_nothing() {
    // The plan is held in memory; only `apply_generation` touches disk. A gate
    // failure therefore leaves no tree at all.
    let mut params = g0_params();
    params.base_iri = "https://example.test/ns".into();
    assert!(plan_generation(&params, "0.6.0").is_err());
}
