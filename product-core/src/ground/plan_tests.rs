//! The whole pipeline over one small corpus: rows, files, shapes, re-runs.

use std::collections::BTreeSet;

use super::*;
use crate::ground::facts::{
    CorpusFacts, DeclKind, Declaration, HierarchyEdge, OperationStatus, PropertyDecl, ReferenceSite,
};

const BASE: &str = "tag:emil@okkels-klein.dk,2026-08-17:ground/";

/// The shapes an instance ships, reduced to what the proposal must satisfy.
const SHAPES: &str = r#"
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix reg: <tag:emil@okkels-klein.dk,2026-08-17:ground/> .
reg:AssertionShape a sh:NodeShape ; sh:targetClass reg:Assertion ;
  sh:property [ sh:path reg:subject ; sh:minCount 1 ; sh:maxCount 1 ; sh:nodeKind sh:IRI ;
                sh:message "one subject, an IRI" ] ;
  sh:property [ sh:path reg:predicate ; sh:minCount 1 ; sh:maxCount 1 ; sh:nodeKind sh:IRI ;
                sh:message "one predicate, an IRI" ] ;
  sh:property [ sh:path reg:object ; sh:minCount 1 ; sh:maxCount 1 ; sh:message "one object" ] .
reg:ReadingShape a sh:NodeShape ; sh:targetClass reg:Reading ;
  sh:property [ sh:path reg:value ; sh:minCount 1 ; sh:message "a value" ] ;
  sh:property [ sh:path reg:asOf ; sh:minCount 1 ; sh:maxCount 1 ; sh:message "one as-of" ] ;
  sh:property [ sh:path reg:provenance ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( reg:controlled reg:observed reg:inferred reg:institutional ) ;
                sh:message "provenance in the vocabulary" ] ;
  sh:property [ sh:path reg:assurance ; sh:minCount 1 ; sh:message "an assurance" ] .
"#;

fn all_six() -> Vec<OperationStatus> {
    ["documentSymbol", "workspaceSymbol", "definition", "references", "hover", "typeHierarchy"]
        .iter()
        .map(|o| OperationStatus { operation: (*o).into(), answered: true, detail: "ok".into() })
        .collect()
}

fn corpus() -> CorpusFacts {
    let settings = Declaration {
        name: "ChargingProfileSettings".into(),
        kind: DeclKind::Class,
        module: "Acme.Sql".into(),
        file: "src/Entities/Settings.cs".into(),
        start_line: 9,
        end_line: 30,
        doc: Some("Local cache of a profile.".into()),
        properties: vec![
            PropertyDecl { name: "Id".into(), type_name: "ProfileId".into(), kind: Default::default() },
            PropertyDecl { name: "DisabledReason".into(), type_name: "int?".into(), kind: Default::default() },
        ],
    };
    let profile_id = Declaration {
        name: "ProfileId".into(),
        kind: DeclKind::Struct,
        module: "Acme.Primitives".into(),
        file: "src/ValueObjects/ProfileId.cs".into(),
        start_line: 3,
        end_line: 3,
        doc: None,
        properties: Vec::new(),
    };
    CorpusFacts {
        corpus: "corpus-backend".into(),
        git_ref: "3b0a56b33b2282e45e53c73d3df2026ac506bd01".into(),
        instrument: "test-instrument 1.0".into(),
        operations: all_six(),
        declarations: vec![settings, profile_id],
        hierarchy: vec![HierarchyEdge {
            sub: "ProfileId".into(),
            sup: "IStringIdentity".into(),
        }],
        references: vec![ReferenceSite {
            target: "ProfileId".into(),
            file: "src/Entities/Settings.cs".into(),
            line: 12,
        }],
        unread_files: Vec::new(),
    }
}

fn params() -> ExtractionParams {
    ExtractionParams {
        base_iri: BASE.into(),
        as_of: "2026-08-18T00:00:00Z".into(),
        graph_dir: "graphs/canonical".into(),
    }
}

fn plan() -> ExtractionPlan {
    plan_extraction(&corpus(), &params(), SHAPES, &BTreeSet::new()).expect("plan")
}

#[test]
fn the_proposal_conforms_to_the_instances_own_shapes() {
    let p = plan();
    assert!(p.conformant(), "{:?}", p.shacl);
    assert!(p.constraints_evaluated > 0);
    assert!(!p.assertions.is_empty());
    assert_eq!(p.files.len(), p.assertions.len(), "one file per assertion");
}

#[test]
fn every_row_of_the_table_is_accounted_for() {
    let p = plan();
    let ids: BTreeSet<&str> = p.rows.iter().map(|r| r.row.id).collect();
    let expected: BTreeSet<&str> = crate::ground::rows::ROWS.iter().map(|r| r.id).collect();
    assert_eq!(ids, expected, "a row missing from the report is a silent row");
}

/// The foreign-key row is reported as not-derivable, never as zero.
#[test]
fn the_foreign_key_row_reports_its_absence() {
    let p = plan();
    let fk = p.rows.iter().find(|r| r.row.id == "foreign-keys").expect("row");
    assert!(matches!(fk.outcome, crate::ground::rows::Outcome::NotDerivable { .. }));
    assert_eq!(fk.row.grade.as_str(), "not-derivable");
}

/// The gate reaches the plan: with typeHierarchy silent, the subclass row is
/// unavailable and the rest of the run is unaffected.
#[test]
fn a_silent_operation_takes_only_its_own_rows_off() {
    let mut facts = corpus();
    facts.operations.retain(|o| o.operation != "typeHierarchy");
    let p = plan_extraction(&facts, &params(), SHAPES, &BTreeSet::new()).expect("plan");
    let sub = p.rows.iter().find(|r| r.row.id == "subclass").expect("row");
    assert!(matches!(sub.outcome, crate::ground::rows::Outcome::Unavailable { .. }));
    let classes = p.rows.iter().find(|r| r.row.id == "classes").expect("row");
    assert!(matches!(classes.outcome, crate::ground::rows::Outcome::Fired { .. }));
}

/// Content-derived identity, end to end: the same corpus plans the same file
/// set with the same names, and an already-ratified id is not re-proposed.
#[test]
fn a_re_run_proposes_nothing_that_is_already_ratified() {
    let first = plan();
    let ratified: BTreeSet<String> = first.assertions.iter().map(|a| a.id.clone()).collect();
    let second = plan_extraction(&corpus(), &params(), SHAPES, &ratified).expect("plan");
    assert!(second.assertions.is_empty(), "{:?}", second.assertions);
    assert!(second.files.is_empty());
    assert_eq!(second.already_ratified, ratified.len());
}

#[test]
fn planning_is_reproducible_given_the_same_as_of() {
    let a = plan();
    let b = plan();
    assert_eq!(a.files, b.files, "same facts, same as-of, same bytes");
}

#[test]
fn assertions_are_graded_and_the_counts_add_up() {
    let p = plan();
    let total: usize = p.by_grade().iter().map(|(_, n)| n).sum();
    assert_eq!(total, p.assertions.len());
    let grades: Vec<&str> = p.by_grade().iter().map(|(g, _)| *g).collect();
    assert!(grades.contains(&"high"), "{grades:?}");
    assert!(grades.contains(&"mid"), "{grades:?}");
    assert!(grades.contains(&"low"), "{grades:?}");
}

/// No proposal may carry `inferred` provenance at declared assurance — the
/// overclaim per-triple review exists to catch, checked mechanically.
#[test]
fn no_inferred_assertion_claims_declared_assurance() {
    for a in plan().assertions {
        if a.provenance == crate::ground::rows::Provenance::Inferred {
            assert_ne!(a.grade, crate::ground::rows::Grade::High, "{}", a.triple.to_value());
        }
    }
}

#[test]
fn the_report_names_the_fixpoint_and_the_rows() {
    let text = crate::ground::render_extraction(&plan());
    assert!(text.contains("by §5.2 row:"), "{text}");
    assert!(text.contains("fixpoint:"), "{text}");
    assert!(text.contains("not derivable"), "{text}");
    assert!(text.contains("entailed, not proposed"), "{text}");
}
