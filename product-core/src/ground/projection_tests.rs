//! Fixtures for the projection join, including the defect it exists to catch.

use super::*;
use crate::ground::derivation::Derivation;
use crate::ground::facts::{DeclKind, MemberKind};
use crate::ground::propose::Proposer;
use crate::ground::rows::row;
use crate::ground::sidecar;
use crate::ground::triple::{ProposedAssertion, Term, Triple};
use crate::ground::write::{assertion_file, WriteContext};

const BASE: &str = "tag:x,2026:ground/";
const DOC: &[&str] = &["documentSymbol"];

fn ctx() -> WriteContext {
    WriteContext {
        base_iri: BASE.into(),
        as_of: "2026-08-20T00:00:00Z".into(),
        corpus: "corpus-backend".into(),
        git_ref: "3b0a56b3".into(),
        corpus_iri: format!("{BASE}source-corpus-backend-at-3b0a56b3"),
        run_iri: format!("{BASE}extraction-corpus-backend-3b0a56b3"),
    }
}

fn assertion(subject: &str, member: MemberKind, path: &str) -> ProposedAssertion {
    Proposer::new("roslyn 5.11.0").propose(
        row("properties").expect("row"),
        Triple::new(format!("{BASE}{subject}"), format!("{BASE}p"), Term::plain("v")),
        vec![],
        Derivation::by("declared-member", DOC)
            .in_container(DeclKind::Class, path)
            .of_member(member),
    )
}

/// An assertion as the 0.2.0 writer emitted it: `reg:assuranceGrade`, no
/// `reg:GradedAssertion`, no relevance, no derivation anywhere.
fn ratified_at_0_2_0(id: &str) -> String {
    format!(
        "@prefix reg: <{BASE}> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         reg:assertion-{id}\n\
             a reg:Assertion , reg:Reading ;\n\
             reg:subject reg:Order ;\n\
             reg:predicate reg:hasPart ;\n\
             reg:object reg:OrderLine ;\n\
             reg:value \"Order hasPart OrderLine\" ;\n\
             reg:asOf \"2026-08-17T00:00:00Z\"^^xsd:dateTime ;\n\
             reg:provenance reg:observed ;\n\
             reg:assurance \"high — mapping rule\" ;\n\
             reg:assuranceGrade reg:high ;\n\
             reg:mappingRow \"classes\" .\n"
    )
}

fn vocabulary() -> String {
    let raw = include_str!("../../../docs/g-track/registry-template/vocabulary/reg.ttl");
    raw.replace("https://REGISTRY-HOST.example/ns#", BASE)
}

fn shapes() -> String {
    let raw = include_str!("../../../docs/g-track/registry-template/shapes/derivation.ttl");
    raw.replace("https://REGISTRY-HOST.example/ns#", BASE)
}

fn sources(graded: &[ProposedAssertion], legacy: &[String]) -> ProjectionSources {
    let ctx = ctx();
    ProjectionSources {
        graphs: graded
            .iter()
            .map(|a| assertion_file(a, &ctx))
            .chain(legacy.iter().cloned())
            .collect(),
        runs: vec![sidecar::run_file(graded, &ctx)],
        vocabulary: vocabulary(),
    }
}

/// The backfill: an assertion ratified before the split answers a
/// `reg:derivationConfidence` query at the projection, with the mark it
/// already carries, without a byte of it changing.
#[test]
fn the_pre_split_cohort_gains_axis_one_by_entailment() {
    let s = sources(&[], &[ratified_at_0_2_0("aaaa")]);
    let rows = query_projection(
        &s,
        &format!(
            "SELECT ?a WHERE {{ ?a <{BASE}derivationConfidence> <{BASE}high> }}"
        ),
    )
    .expect("query");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0][0].contains("assertion-aaaa"), "{rows:?}");
}

/// **The defect Emil found at Gate 1, as a fixture.**
///
/// The equivalence entailment puts `reg:derivationConfidence` on every
/// pre-0.3.0 assertion at the projection. A shape targeting that predicate's
/// presence would pass in the authority repo — where validation runs before
/// entailment — and fail here. The shape targets `reg:GradedAssertion`, which
/// the writer stamps and nothing entails, so the older cohort stays untargeted
/// on both sides of the projection.
#[test]
fn entailment_does_not_drag_the_pre_split_cohort_into_the_new_shape() {
    let s = sources(&[], &[ratified_at_0_2_0("aaaa")]);

    // The entailment really does fire — otherwise this fixture would pass for
    // the wrong reason.
    let entailed = query_projection(
        &s,
        &format!("SELECT ?a WHERE {{ ?a <{BASE}derivationConfidence> ?c }}"),
    )
    .expect("query");
    assert_eq!(entailed.len(), 1, "the equivalence must actually entail");

    // And the shape still does not target it.
    let typed = query_projection(
        &s,
        &format!("SELECT ?a WHERE {{ ?a a <{BASE}GradedAssertion> }}"),
    )
    .expect("query");
    assert!(typed.is_empty(), "nothing may entail the generation marker: {typed:?}");

    let (findings, evaluated) = validate_projection(&s, &shapes()).expect("validate");
    assert!(evaluated > 0, "the shape must have been read");
    assert!(findings.is_empty(), "{findings:#?}");
}

/// A 0.3.0 assertion joined with its run evidence satisfies the whole shape —
/// both marks from the graph layer, rule and operations from the evidence
/// layer. The join is what is validated.
#[test]
fn a_graded_assertion_conforms_only_once_its_evidence_is_joined() {
    let a = assertion("A", MemberKind::Field, "src/Converters/Foo.cs");
    let joined = sources(std::slice::from_ref(&a), &[]);
    let (findings, _) = validate_projection(&joined, &shapes()).expect("validate");
    assert!(findings.is_empty(), "{findings:#?}");

    // Without the evidence layer the same assertion fails, loudly, on the
    // derivation half — an assertion whose evidence never arrived is a finding
    // rather than a quiet pass.
    let unjoined = ProjectionSources { runs: Vec::new(), ..joined };
    let (findings, _) = validate_projection(&unjoined, &shapes()).expect("validate");
    assert!(
        findings.iter().any(|f| f.message.contains("rule that produced it")),
        "{findings:#?}"
    );
}

/// **Group B is recoverable from the graph by query** — the forcing argument,
/// as a test. Nothing in a ratified 0.2.0 assertion could say this.
#[test]
fn the_field_derived_group_is_a_query_over_the_projection() {
    let all = [
        assertion("A", MemberKind::Field, "src/Converters/Foo.cs"),
        assertion("B", MemberKind::Property, "src/Entities/Bar.cs"),
        assertion("C", MemberKind::Field, "src/Converters/Baz.cs"),
    ];
    let s = sources(&all, &[]);
    let rows =
        query_projection(&s, &format!("SELECT ?a WHERE {{ ?a <{BASE}memberKind> <{BASE}field> }}"))
            .expect("query");
    assert_eq!(rows.len(), 2, "{rows:?}");
}

/// **Group A is recoverable from the graph by query** — over `reg:sourcePath`,
/// a fact the extractor recorded rather than a judgement it made. The
/// predicate is the reviewer's; the extractor filtered nothing.
#[test]
fn the_generated_code_group_is_a_query_over_recorded_paths() {
    let all = [
        assertion("A", MemberKind::Property, "src/Sql/Migrations/20260101_Add.cs"),
        assertion("B", MemberKind::Property, "src/Entities/Bar.cs"),
        assertion("C", MemberKind::Property, "src/Sql/Migrations/20260202_Drop.cs"),
    ];
    let s = sources(&all, &[]);
    let rows = query_projection(
        &s,
        &format!(
            "SELECT ?a WHERE {{ ?a <{BASE}sourcePath> ?p . FILTER(CONTAINS(STR(?p), \"/Migrations/\")) }}"
        ),
    )
    .expect("query");
    assert_eq!(rows.len(), 2, "{rows:?}");
}

/// The two groups are recoverable *independently*: a query for one does not
/// return the other, which is what makes them separate review batches rather
/// than one undifferentiated block of rejections.
#[test]
fn the_two_groups_are_separable_from_one_another() {
    let all = [
        assertion("A", MemberKind::Field, "src/Converters/Foo.cs"),
        assertion("B", MemberKind::Property, "src/Sql/Migrations/20260101_Add.cs"),
    ];
    let s = sources(&all, &[]);
    let fields =
        query_projection(&s, &format!("SELECT ?a WHERE {{ ?a <{BASE}memberKind> <{BASE}field> }}"))
            .expect("query");
    let generated = query_projection(
        &s,
        &format!(
            "SELECT ?a WHERE {{ ?a <{BASE}sourcePath> ?p . FILTER(CONTAINS(STR(?p), \"/Migrations/\")) }}"
        ),
    )
    .expect("query");
    assert_eq!(fields.len(), 1);
    assert_eq!(generated.len(), 1);
    assert_ne!(fields[0][0], generated[0][0]);
}
