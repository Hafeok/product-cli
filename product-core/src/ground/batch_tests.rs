//! Fixtures for the reviewer-facing batching.

use super::*;
use crate::ground::derivation::Derivation;
use crate::ground::propose::Proposer;
use crate::ground::rows::row;
use crate::ground::triple::{Term, Triple};

const DOC: &[&str] = &["documentSymbol"];

fn assertion(row_id: &str, subject: &str, d: Derivation) -> ProposedAssertion {
    let p = Proposer::new("test 1.0");
    let triple = Triple::new(format!("tag:x/{subject}"), "tag:x/p", Term::iri("tag:x/o"));
    p.propose(row(row_id).expect("row"), triple, vec![], d)
}

fn member(kind: MemberKind, subject: &str) -> ProposedAssertion {
    assertion(
        "properties",
        subject,
        Derivation::by("declared-member", DOC)
            .in_container(DeclKind::Class, "src/A.cs")
            .of_member(kind),
    )
}

/// The forcing case, as a batch: assertions derived from members the server
/// called *fields* land in their own batch, distinguishable from the
/// property-derived ones that read identically on every other axis.
#[test]
fn field_derived_assertions_batch_apart_from_property_derived_ones() {
    let all = vec![
        member(MemberKind::Property, "A"),
        member(MemberKind::Field, "B"),
        member(MemberKind::Property, "C"),
        member(MemberKind::Field, "D"),
    ];
    let out = batches(&all);
    assert_eq!(out.len(), 2, "{out:#?}");
    let fields: Vec<&Batch> =
        out.iter().filter(|b| b.member_kind == Some(MemberKind::Field)).collect();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].count, 2);
    // Same confidence, same relevance, same row — only the derivation splits
    // them, which is exactly what a ratified assertion could not say.
    assert_eq!(out[0].confidence, out[1].confidence);
    assert_eq!(out[0].relevance, out[1].relevance);
    assert_eq!(out[0].row, out[1].row);
}

/// A batch listing is the same listing on every run over the same facts:
/// the key order is total, and the ids inside a batch are sorted.
#[test]
fn batching_is_deterministic_under_input_order() {
    let forward = vec![
        member(MemberKind::Field, "B"),
        assertion("synonyms", "C", Derivation::by("identifier-word-split", DOC)),
        member(MemberKind::Property, "A"),
    ];
    let mut backward = forward.clone();
    backward.reverse();
    let a = batches(&forward);
    let b = batches(&backward);
    assert_eq!(a, b);
}

/// Confidence sorts best-first, so the batches a reviewer can take at pace
/// come first and the heuristic tail comes last.
#[test]
fn batches_lead_with_the_highest_confidence() {
    let all = vec![
        assertion("synonyms", "S", Derivation::by("identifier-word-split", DOC)),
        member(MemberKind::Property, "P"),
        assertion("modules", "M", Derivation::by("module-containment", DOC)),
    ];
    let out = batches(&all);
    let order: Vec<&str> = out.iter().map(|b| b.confidence.as_str()).collect();
    assert_eq!(order, ["high", "mid", "low"]);
}

/// The label is what a reviewer reads down a listing: both marks, the row,
/// the rule, the source kinds.
#[test]
fn a_batch_label_carries_the_pair_with_its_derivation() {
    let out = batches(&[member(MemberKind::Field, "B")]);
    let label = out[0].label();
    assert!(label.starts_with("high/unknown · properties · declared-member"), "{label}");
    assert!(label.contains("field"), "{label}");
    assert!(label.contains("class"), "{label}");
}

/// A batch names some members in full so a reviewer can read what it holds,
/// and lists every id so acting on it needs no second query.
#[test]
fn a_batch_samples_bounded_but_lists_every_member() {
    let all: Vec<ProposedAssertion> =
        (0..12).map(|i| member(MemberKind::Field, &format!("T{i}"))).collect();
    let out = batches(&all);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].count, 12);
    assert_eq!(out[0].ids.len(), 12);
    assert_eq!(out[0].sample.len(), SAMPLE);
}

/// Every batch carries both marks. A batch is the review unit, so a batch
/// with one mark would put the reviewer back where G0 found them.
#[test]
fn every_batch_carries_both_marks_with_the_basis() {
    let all = vec![
        member(MemberKind::Field, "B"),
        assertion("usage-relations", "U", Derivation::by("usage-construct", DOC)),
    ];
    for b in batches(&all) {
        assert!(!b.confidence.as_str().is_empty());
        assert_eq!(b.relevance, DomainRelevance::Unknown);
        assert_eq!(b.relevance_basis, RelevanceBasis::NotComputed);
        assert!(!b.stands_on.is_empty(), "a batch names what it stood on");
    }
}
