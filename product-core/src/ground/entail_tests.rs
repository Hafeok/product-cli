//! The graph stage: the containment join, the fixpoint, the measurement.

use super::*;
use crate::ground::facts::{DeclKind, Declaration, ReferenceSite};
use crate::ground::vocab::RDF_TYPE;

const BASE: &str = "tag:x,2026:ground/";

fn decl(name: &str, file: &str, start: u64, end: u64) -> Declaration {
    Declaration {
        name: name.into(),
        kind: DeclKind::Class,
        module: "Acme".into(),
        file: file.into(),
        start_line: start,
        end_line: end,
        doc: None,
        properties: Vec::new(),
    }
}

fn facts(declarations: Vec<Declaration>, references: Vec<ReferenceSite>) -> CorpusFacts {
    CorpusFacts {
        corpus: "corpus-backend".into(),
        git_ref: "abc".into(),
        instrument: "test".into(),
        operations: Vec::new(),
        declarations,
        hierarchy: Vec::new(),
        references,
        unread_files: Vec::new(),
    }
}

fn iri(name: &str) -> String {
    format!("{BASE}{name}")
}

/// A reference *position* is not yet a relation. The first CONSTRUCT joins it
/// to the declaration whose span contains it, which is what turns a use site
/// into a type-to-type edge.
#[test]
fn the_containment_join_turns_a_reference_position_into_an_edge() {
    let f = facts(
        vec![decl("Handler", "h.cs", 0, 20), decl("Settings", "s.cs", 0, 10)],
        vec![ReferenceSite { target: "Settings".into(), file: "h.cs".into(), line: 5 }],
    );
    let (derived, report) = run(BASE, &f, &[]).expect("run");
    let edge = Triple::new(iri("Handler"), reg::depends_on(BASE), Term::iri(iri("Settings")));
    assert!(derived.contains(&edge), "{derived:?}");
    assert_eq!(report.proposed(), 1);
}

/// A reference outside every declaration's span produces no edge: there is
/// no type it belongs to, and inventing one would be fabrication.
#[test]
fn a_reference_outside_every_declaration_yields_no_edge() {
    let f = facts(
        vec![decl("Handler", "h.cs", 0, 4), decl("Settings", "s.cs", 0, 10)],
        vec![ReferenceSite { target: "Settings".into(), file: "h.cs".into(), line: 50 }],
    );
    let (derived, _) = run(BASE, &f, &[]).expect("run");
    assert!(derived.is_empty(), "{derived:?}");
}

/// The closure is a genuine fixpoint: A→B→C→D needs more than one round, and
/// the last round adds nothing.
#[test]
fn the_transitive_closure_runs_to_a_fixpoint() {
    let declarations = vec![
        decl("A", "a.cs", 0, 10),
        decl("B", "b.cs", 0, 10),
        decl("C", "c.cs", 0, 10),
        decl("D", "d.cs", 0, 10),
    ];
    let references = vec![
        ReferenceSite { target: "B".into(), file: "a.cs".into(), line: 5 },
        ReferenceSite { target: "C".into(), file: "b.cs".into(), line: 5 },
        ReferenceSite { target: "D".into(), file: "c.cs".into(), line: 5 },
    ];
    let f = facts(declarations, references);
    let (derived, report) = run(BASE, &f, &[]).expect("run");
    assert_eq!(derived.len(), 3, "only the direct edges are proposed: {derived:?}");
    assert!(report.rounds >= 3, "a chain of three needs several rounds: {report:?}");
    // A→C, A→D, B→D are entailed, recomputable, and never proposed.
    assert_eq!(report.entailed(), 3, "{report:?}");
}

/// The lever the G0 input names: one parse per rule however many rounds run.
#[test]
fn each_rule_is_parsed_once_and_executed_per_round() {
    let f = facts(
        vec![decl("A", "a.cs", 0, 10), decl("B", "b.cs", 0, 10), decl("C", "c.cs", 0, 10)],
        vec![
            ReferenceSite { target: "B".into(), file: "a.cs".into(), line: 5 },
            ReferenceSite { target: "C".into(), file: "b.cs".into(), line: 5 },
        ],
    );
    let (_, report) = run(BASE, &f, &[]).expect("run");
    assert_eq!(report.parses, rules(BASE).len(), "one parse per rule, never per round");
    assert_eq!(report.executions, report.rounds * rules(BASE).len());
    assert!(report.rounds > 1, "the measurement is only meaningful past one round");
}

/// Subclass transitivity is entailment: recomputable from proposed
/// assertions, so it is measured and never written to the authority repo.
#[test]
fn subclass_transitivity_is_entailed_rather_than_proposed() {
    let f = facts(vec![decl("A", "a.cs", 0, 1)], Vec::new());
    let asserted = vec![
        Triple::new(iri("A"), RDFS_SUBCLASS_OF, Term::iri(iri("B"))),
        Triple::new(iri("B"), RDFS_SUBCLASS_OF, Term::iri(iri("C"))),
    ];
    let (derived, report) = run(BASE, &f, &asserted).expect("run");
    assert!(derived.is_empty(), "nothing from this rule is proposed: {derived:?}");
    let sco = report.rules.iter().find(|r| r.id == "subclass-transitive").expect("rule");
    assert_eq!(sco.triples, 1, "A subClassOf C");
    assert!(!sco.proposed);
}

/// Re-running the stage over the same facts derives the same set — the
/// fixpoint is a function of its input, not of its order.
#[test]
fn the_stage_is_deterministic() {
    let f = facts(
        vec![decl("A", "a.cs", 0, 10), decl("B", "b.cs", 0, 10)],
        vec![ReferenceSite { target: "B".into(), file: "a.cs".into(), line: 3 }],
    );
    let (first, a) = run(BASE, &f, &[]).expect("first");
    let (second, b) = run(BASE, &f, &[]).expect("second");
    assert_eq!(first, second);
    assert_eq!((a.rounds, a.proposed(), a.entailed()), (b.rounds, b.proposed(), b.entailed()));
}

/// A type declared but never referenced, and a reference to something the
/// subset never declared: both simply produce nothing.
#[test]
fn an_unreferenced_type_and_an_undeclared_target_both_yield_nothing() {
    let f = facts(
        vec![decl("Lonely", "l.cs", 0, 10)],
        vec![ReferenceSite { target: "Outside".into(), file: "l.cs".into(), line: 5 }],
    );
    let (derived, report) = run(BASE, &f, &[]).expect("run");
    assert!(derived.is_empty());
    assert_eq!(report.proposed(), 0);
    let _ = RDF_TYPE;
}
