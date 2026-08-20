//! The declared rows, including their gating on the capability probe.

use super::*;
use crate::ground::facts::{DeclKind, Declaration, HierarchyEdge, OperationStatus, PropertyDecl};

const BASE: &str = "tag:x,2026:ground/";

fn answered(ops: &[&str]) -> Vec<OperationStatus> {
    ops.iter()
        .map(|o| OperationStatus { operation: (*o).into(), answered: true, detail: "ok".into() })
        .collect()
}

fn field(name: &str, type_name: &str) -> PropertyDecl {
    PropertyDecl {
        name: name.into(),
        type_name: type_name.into(),
        kind: crate::ground::facts::MemberKind::Field,
    }
}

fn decl(name: &str, kind: DeclKind, props: Vec<(&str, &str)>) -> Declaration {
    Declaration {
        name: name.into(),
        kind,
        module: "Acme.Sql".into(),
        file: "src/Entities/Settings.cs".into(),
        start_line: 9,
        end_line: 30,
        doc: None,
        properties: props
            .into_iter()
            .map(|(n, t)| PropertyDecl { name: n.into(), type_name: t.into(), kind: Default::default() })
            .collect(),
    }
}

fn facts(ops: &[&str], declarations: Vec<Declaration>, hierarchy: Vec<HierarchyEdge>) -> CorpusFacts {
    CorpusFacts {
        corpus: "corpus-backend".into(),
        git_ref: "deadbeef".into(),
        instrument: "test-instrument".into(),
        operations: answered(ops),
        declarations,
        hierarchy,
        references: Vec::new(),
        unread_files: Vec::new(),
    }
}

fn p() -> Proposer {
    Proposer::new("test-instrument")
}

#[test]
fn every_declared_type_becomes_a_class_candidate() {
    let f = facts(
        &["documentSymbol"],
        vec![decl("Settings", DeclKind::Class, vec![]), decl("ICommand", DeclKind::Interface, vec![])],
        Vec::new(),
    );
    let (out, outcome) = classes(BASE, &f, &p());
    assert_eq!(out.len(), 2);
    assert_eq!(outcome, Outcome::Fired { triples: 2 });
    assert!(out.iter().all(|a| a.triple.predicate == RDF_TYPE));
}

/// The gate, which is the whole reason the probe exists: an operation that
/// did not answer takes its row off, by name, rather than yielding zero.
#[test]
fn a_row_whose_operation_did_not_answer_reports_it_rather_than_finding_nothing() {
    let hierarchy = vec![HierarchyEdge { sub: "Settings".into(), sup: "ICommand".into() }];
    let f = facts(&["documentSymbol"], vec![decl("Settings", DeclKind::Class, vec![])], hierarchy);
    let mut notes = DeclaredNotes::default();
    let (out, outcome) = subclass(BASE, &f, &p(), &mut notes);
    assert!(out.is_empty());
    assert_eq!(outcome, Outcome::Unavailable { operations: vec!["typeHierarchy".into()] });
    assert_ne!(outcome, Outcome::FoundNothing, "these are different findings");
}

#[test]
fn a_row_past_the_gate_with_nothing_to_read_found_nothing() {
    let f = facts(
        &["documentSymbol", "typeHierarchy"],
        vec![decl("Settings", DeclKind::Class, vec![])],
        Vec::new(),
    );
    let mut notes = DeclaredNotes::default();
    let (out, outcome) = subclass(BASE, &f, &p(), &mut notes);
    assert!(out.is_empty());
    assert_eq!(outcome, Outcome::FoundNothing);
}

#[test]
fn a_supertype_outside_the_loaded_subset_is_counted_not_hidden() {
    let hierarchy = vec![HierarchyEdge { sub: "Settings".into(), sup: "IExternal".into() }];
    let f = facts(
        &["documentSymbol", "typeHierarchy"],
        vec![decl("Settings", DeclKind::Class, vec![])],
        hierarchy,
    );
    let mut notes = DeclaredNotes::default();
    let (out, _) = subclass(BASE, &f, &p(), &mut notes);
    assert_eq!(out.len(), 1, "the edge is still proposed");
    assert!(notes.supertypes_outside_subset.contains("IExternal"));
}

/// A property whose type is a declared type of this corpus is an object
/// property; a primitive is a datatype property with an xsd range; anything
/// else carries no range at all rather than a guessed one.
#[test]
fn property_kinds_follow_whether_the_range_resolves() {
    let f = facts(
        &["documentSymbol"],
        vec![
            decl(
                "Settings",
                DeclKind::Class,
                vec![("Id", "ProfileId"), ("Reason", "int?"), ("Key", "Guid")],
            ),
            decl("ProfileId", DeclKind::Struct, vec![]),
        ],
        Vec::new(),
    );
    let mut notes = DeclaredNotes::default();
    let (out, _) = properties(BASE, &f, &p(), &mut notes);
    let types: Vec<&Term> = out
        .iter()
        .filter(|a| a.triple.predicate == RDF_TYPE)
        .map(|a| &a.triple.object)
        .collect();
    assert!(types.contains(&&Term::iri(OWL_OBJECT_PROPERTY)), "{types:?}");
    assert!(types.contains(&&Term::iri(OWL_DATATYPE_PROPERTY)), "{types:?}");
    let ranges: Vec<String> =
        out.iter().filter(|a| a.triple.predicate == RDFS_RANGE).map(|a| a.triple.object.short()).collect();
    assert!(ranges.contains(&"ProfileId".to_string()), "{ranges:?}");
    assert!(ranges.contains(&"int".to_string()), "{ranges:?}");
    assert_eq!(notes.unranged_properties, ["Settings.Key"], "a GUID gets no range, and is named");
    assert_eq!(notes.nullable_properties, 1);
}

#[test]
fn the_same_triple_read_twice_collapses_to_one_assertion() {
    let f = facts(
        &["documentSymbol"],
        vec![decl("Settings", DeclKind::Class, vec![]), decl("Settings", DeclKind::Class, vec![])],
        Vec::new(),
    );
    let (out, outcome) = classes(BASE, &f, &p());
    assert_eq!(out.len(), 1);
    assert_eq!(outcome, Outcome::Fired { triples: 1 });
}

/// The row cannot tell implementation from domain structure, so it proposes
/// the field's property triples *and* names the field, letting a reviewer
/// batch what the row could not split.
#[test]
fn a_field_derived_property_is_proposed_and_named() {
    let mut owner = decl("Converter", DeclKind::Class, vec![("Value", "int")]);
    owner.properties.push(field("cache", "Dictionary"));
    let f = facts(&["documentSymbol"], vec![owner], Vec::new());
    let mut notes = DeclaredNotes::default();
    let (out, outcome) = properties(BASE, &f, &p(), &mut notes);
    assert!(matches!(outcome, Outcome::Fired { .. }));
    assert!(
        out.iter().any(|a| a.triple.subject.ends_with("prop-Converter-cache")),
        "the field still reaches the row"
    );
    assert_eq!(notes.field_derived, ["Converter.cache"]);
}
