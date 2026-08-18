//! Unit tests for the conformance reader, positive with negative.

use super::*;
use crate::registry::plan::plan_generation;

const BASE: &str = "https://registry.test/ns#";

fn params() -> crate::registry::params::RegistryParams {
    crate::registry::params::RegistryParams {
        owner: "Hafeok".into(),
        repo: "registry-under-test".into(),
        ratifier: "ratifier@example.test".into(),
        display_name: "Registry under test".into(),
        base_iri: BASE.into(),
        mint_date: "2026-08-17".into(),
        generated_by: "the test suite".into(),
    }
}

/// The shapes an instance generated with `BASE` actually ships.
fn instance_shapes() -> String {
    let plan = plan_generation(&params(), "test").expect("plan");
    plan.files
        .iter()
        .filter(|f| f.path.starts_with("shapes/"))
        .map(|f| f.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn evaluate(data: &str) -> Vec<Finding> {
    let shapes = instance_shapes();
    let extracted = shapes::extract(&shapes).expect("extract");
    assert!(extracted.unevaluable.is_empty(), "the shipped shapes must be readable: {:?}", extracted.unevaluable);
    assert!(!extracted.constraints.is_empty(), "the shipped shapes yielded no constraints");
    eval::run(data, &extracted.constraints).expect("evaluate")
}

fn reading(extra: &str) -> String {
    format!(
        "@prefix reg: <{BASE}> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:a-1 a reg:Assertion , reg:Reading ;\n\
             reg:subject reg:Order ; reg:predicate reg:hasPart ; reg:object reg:OrderLine ;\n\
             reg:value \"Order hasPart OrderLine\" ;\n\
             reg:asOf \"2026-08-17T00:00:00Z\"^^xsd:dateTime ;\n\
             reg:assurance \"hand-authored\" ;\n{extra} .\n"
    )
}

#[test]
fn a_conforming_reading_passes() {
    let findings = evaluate(&reading("    reg:provenance reg:controlled"));
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn an_institutional_reading_without_a_trust_decision_fails() {
    let findings = evaluate(&reading("    reg:provenance reg:institutional"));
    assert!(
        findings.iter().any(|f| f.message.contains("trust_decision")),
        "institutional provenance must require a trust decision: {findings:?}"
    );
}

#[test]
fn an_institutional_reading_with_a_trust_decision_passes() {
    let findings = evaluate(&reading("    reg:provenance reg:institutional ;\n    reg:trust_decision reg:td-1"));
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_reading_missing_its_value_fails() {
    let data = format!(
        "@prefix reg: <{BASE}> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:a-1 a reg:Assertion , reg:Reading ;\n\
             reg:subject reg:Order ; reg:predicate reg:hasPart ; reg:object reg:OrderLine ;\n\
             reg:asOf \"2026-08-17T00:00:00Z\"^^xsd:dateTime ;\n\
             reg:assurance \"hand-authored\" ; reg:provenance reg:controlled .\n"
    );
    let findings = evaluate(&data);
    assert!(findings.iter().any(|f| f.message.contains("carries a value")), "{findings:?}");
}

#[test]
fn a_provenance_outside_the_vocabulary_fails() {
    let findings = evaluate(&reading("    reg:provenance reg:hearsay"));
    assert!(findings.iter().any(|f| f.message.contains("controlled/observed")), "{findings:?}");
}

#[test]
fn a_file_with_two_assertions_fails_the_file_rule() {
    let data = format!(
        "@prefix reg: <{BASE}> .\nreg:a-1 a reg:Assertion .\nreg:a-2 a reg:Assertion .\n"
    );
    let findings = file_rule::check_file("graphs/canonical/two.ttl", &data, BASE);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, FindingKind::FileRule);
    assert!(findings[0].message.contains("carries 2"), "{:?}", findings[0]);
}

#[test]
fn a_file_with_one_assertion_or_one_decision_passes_the_file_rule() {
    for kind in ["Assertion", "Decision"] {
        let data = format!("@prefix reg: <{BASE}> .\nreg:x a reg:{kind} .\n");
        assert!(file_rule::check_file("graphs/canonical/one.ttl", &data, BASE).is_empty());
    }
}

#[test]
fn an_empty_file_fails_the_file_rule() {
    // The zero-triple tolerance went with the empty founding-decision slot.
    let findings = file_rule::check_file("graphs/canonical/empty.ttl", "# nothing\n", BASE);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("carries 0"), "{:?}", findings[0]);
}

#[test]
fn a_decision_missing_a_required_field_fails() {
    let data = format!(
        "@prefix reg: <{BASE}> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:d-1 a reg:Decision ; reg:title \"Keep a registry\" ;\n\
             reg:resolution \"We keep one.\" ; reg:region \"This instance.\" ;\n\
             reg:ratifiedBy \"ratifier@example.test\" ; reg:status \"ratified\" ;\n\
             reg:made \"2026-08-17\"^^xsd:date .\n"
    );
    let findings = evaluate(&data);
    assert!(findings.iter().any(|f| f.message.contains("falsifier")), "{findings:?}");
    assert!(!findings.iter().any(|f| f.message.contains("title")), "{findings:?}");
}

#[test]
fn a_decision_without_the_optional_fields_conforms() {
    // basis / acceptedCost / revisitIf are deliberately not required: a decision
    // may honestly have none, and demanding them produces filler.
    let data = format!(
        "@prefix reg: <{BASE}> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:d-1 a reg:Decision ; reg:title \"Keep a registry\" ;\n\
             reg:resolution \"We keep one.\" ; reg:region \"This instance.\" ;\n\
             reg:falsifier \"Ground kept outside it.\" ;\n\
             reg:ratifiedBy \"ratifier@example.test\" ; reg:status \"ratified\" ;\n\
             reg:made \"2026-08-17\"^^xsd:date .\n"
    );
    assert!(evaluate(&data).is_empty(), "{:?}", evaluate(&data));
}

#[test]
fn a_decision_dated_by_a_bare_string_fails() {
    let data = format!(
        "@prefix reg: <{BASE}> .\n\
         reg:d-1 a reg:Decision ; reg:title \"t\" ; reg:resolution \"r\" ; reg:region \"g\" ;\n\
             reg:falsifier \"f\" ; reg:ratifiedBy \"who\" ; reg:status \"s\" ; reg:made \"2026-08-17\" .\n"
    );
    assert!(evaluate(&data).iter().any(|f| f.message.contains("xsd:date")), "{:?}", evaluate(&data));
}

#[test]
fn the_shipped_decision_exemplar_conforms() {
    let plan = plan_generation(&params(), "test").expect("plan");
    let exemplar = plan
        .files
        .iter()
        .find(|f| f.path == "graphs/canonical/_exemplar-decision.ttl")
        .expect("the template ships a decision exemplar");
    assert!(evaluate(&exemplar.contents).is_empty(), "{:?}", evaluate(&exemplar.contents));
    assert!(file_rule::check_file(&exemplar.path, &exemplar.contents, BASE).is_empty());
}

#[test]
fn an_unsupported_shacl_construct_fails_closed() {
    let shapes = format!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix reg: <{BASE}> .\n\
         reg:S a sh:NodeShape ; sh:targetClass reg:Reading ;\n\
           sh:property [ sh:path reg:value ; sh:qualifiedValueShape reg:Other ] .\n"
    );
    let extracted = shapes::extract(&shapes).expect("extract");
    assert!(
        extracted.unevaluable.iter().any(|f| f.focus.contains("qualifiedValueShape")),
        "an unreadable construct must be reported, never skipped: {:?}",
        extracted.unevaluable
    );
    assert_eq!(extracted.unevaluable[0].kind, FindingKind::Unevaluable);
}

#[test]
fn a_property_path_expression_is_reported_rather_than_ignored() {
    let shapes = format!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix reg: <{BASE}> .\n\
         reg:S a sh:NodeShape ; sh:targetClass reg:Reading ;\n\
           sh:property [ sh:path [ sh:inversePath reg:value ] ; sh:minCount 1 ] .\n"
    );
    let extracted = shapes::extract(&shapes).expect("extract");
    assert!(!extracted.unevaluable.is_empty());
}

#[test]
fn a_base_iri_is_read_from_the_instances_own_prefix() {
    let ttl = format!("@prefix reg: <{BASE}> .\nreg:x a reg:Registry .\n");
    assert_eq!(prefix_binding(&ttl, "reg").as_deref(), Some(BASE));
}
