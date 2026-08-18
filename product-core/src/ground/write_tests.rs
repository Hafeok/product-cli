//! The writer: prefix shortening, field order, byte stability.

use super::*;
use crate::ground::rows::{row, Grade, Provenance};
use crate::ground::triple::Triple;
use crate::ground::vocab::{OWL_CLASS, RDF_TYPE};

const BASE: &str = "tag:emil@okkels-klein.dk,2026-08-17:ground/";

fn ctx() -> WriteContext {
    WriteContext {
        base_iri: BASE.into(),
        as_of: "2026-08-18T00:00:00Z".into(),
        corpus: "corpus-backend".into(),
        git_ref: "3b0a56b33b2282e45e53c73d3df2026ac506bd01".into(),
        corpus_iri: format!("{BASE}corpus-corpus-backend-at-3b0a56b33b22"),
        run_iri: format!("{BASE}extraction-corpus-backend-3b0a56b33b22"),
    }
}

fn assertion() -> ProposedAssertion {
    let p = crate::ground::propose::Proposer::new("roslyn 5.11.0");
    p.propose(
        row("classes").expect("row"),
        Triple::new(format!("{BASE}Settings"), RDF_TYPE, Term::iri(OWL_CLASS)),
        vec!["src/Entities/Settings.cs:10".into()],
    )
}

#[test]
fn an_assertion_file_carries_the_whole_reading_tuple() {
    let text = assertion_file(&assertion(), &ctx());
    let fields: Vec<(&str, &str)> = statement_fields(&text);
    let by_key = |k: &str| fields.iter().find(|(key, _)| *key == k).map(|(_, v)| *v);
    assert_eq!(by_key("a"), Some("reg:Assertion , reg:Reading"));
    assert_eq!(by_key("reg:subject"), Some("reg:Settings"));
    assert_eq!(by_key("reg:predicate"), Some("rdf:type"));
    assert_eq!(by_key("reg:object"), Some("owl:Class"));
    assert_eq!(by_key("reg:value"), Some("\"Settings type Class\""));
    assert_eq!(by_key("reg:asOf"), Some("\"2026-08-18T00:00:00Z\"^^xsd:dateTime"));
    assert_eq!(by_key("reg:provenance"), Some("reg:observed"));
    assert_eq!(by_key("reg:assuranceGrade"), Some("reg:high"));
    assert_eq!(by_key("reg:mappingRow"), Some("\"classes\""));
    assert_eq!(by_key("reg:evidence"), Some("\"src/Entities/Settings.cs:10\""));
    assert!(by_key("reg:assurance").is_some_and(|v| v.contains("row classes")), "{text}");
    assert!(by_key("prov:wasDerivedFrom").is_some(), "{text}");
    assert!(by_key("prov:wasAttributedTo").is_some(), "{text}");
    assert!(text.trim_end().ends_with('.'), "the statement closes:\n{text}");
}

#[test]
fn iris_shorten_to_bound_prefixes_where_the_local_part_is_safe() {
    assert_eq!(shorten(&format!("{BASE}Settings"), BASE), "reg:Settings");
    assert_eq!(shorten(RDF_TYPE, BASE), "rdf:type");
    assert_eq!(shorten(OWL_CLASS, BASE), "owl:Class");
    assert_eq!(
        shorten("http://www.w3.org/2001/XMLSchema#int", BASE),
        "xsd:int"
    );
}

/// A local part a prefixed name cannot carry falls back to the full IRI
/// rather than being rewritten into something that parses differently.
#[test]
fn an_unsafe_local_part_falls_back_to_the_full_iri() {
    let awkward = format!("{BASE}has.a.dot");
    assert_eq!(shorten(&awkward, BASE), format!("<{awkward}>"));
    assert_eq!(shorten("urn:unbound:thing", BASE), "<urn:unbound:thing>");
}

/// The point of owning the writer: the same assertion is the same bytes,
/// every time, whatever order the evidence arrived in.
#[test]
fn the_same_assertion_writes_the_same_bytes() {
    let mut a = assertion();
    a.evidence = vec!["b.cs:2".into(), "a.cs:1".into(), "b.cs:2".into()];
    let mut b = assertion();
    b.evidence = vec!["a.cs:1".into(), "b.cs:2".into()];
    assert_eq!(assertion_file(&a, &ctx()), assertion_file(&b, &ctx()));
}

#[test]
fn grade_and_provenance_are_written_from_the_row() {
    let a = assertion();
    assert_eq!(a.grade, Grade::High);
    assert_eq!(a.provenance, Provenance::Observed);
}

/// Split the statement into (predicate, object) pairs, so a test asserts the
/// content rather than the column the writer happens to align to.
fn statement_fields(text: &str) -> Vec<(&str, &str)> {
    text.lines()
        .filter(|l| l.starts_with("    "))
        .filter_map(|l| {
            let body = l.trim().trim_end_matches([';', '.']).trim();
            body.split_once(char::is_whitespace).map(|(k, v)| (k, v.trim()))
        })
        .collect()
}
