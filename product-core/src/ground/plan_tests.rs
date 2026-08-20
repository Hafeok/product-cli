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
reg:GradedAssertionShape a sh:NodeShape ; sh:targetClass reg:GradedAssertion ;
  sh:property [ sh:path reg:derivationConfidence ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( reg:high reg:mid reg:low reg:not-derivable ) ;
                sh:message "one derivation confidence" ] ;
  sh:property [ sh:path reg:domainRelevance ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( reg:high reg:low reg:unknown ) ;
                sh:message "one domain relevance; unknown is a mark, not an omission" ] ;
  sh:property [ sh:path reg:relevanceBasis ; sh:minCount 1 ; sh:maxCount 1 ;
                sh:in ( reg:computed reg:defaulted reg:not-computed reg:not-applicable ) ;
                sh:message "a relevance basis" ] ;
  sh:property [ sh:path reg:derivedByRule ; sh:minCount 1 ;
                sh:message "the rule that produced it" ] ;
  sh:property [ sh:path reg:standsOn ; sh:minCount 1 ;
                sh:message "at least one probed operation" ] .
"#;

fn all_six() -> Vec<OperationStatus> {
    ["documentSymbol", "workspaceSymbol", "definition", "references", "hover", "typeHierarchy"]
        .iter()
        .map(|o| OperationStatus {
            operation: (*o).into(),
            answered: true,
            advertised: Some(true),
            divergence: None,
            detail: "ok".into(),
        })
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
    // One file per assertion, plus one run-evidence file for the whole run.
    assert_eq!(p.files.len(), p.assertions.len() + 1, "one file per assertion, plus the sidecar");
    let runs: Vec<&PlannedFile> =
        p.files.iter().filter(|f| f.path.starts_with("runs/")).collect();
    assert_eq!(runs.len(), 1, "exactly one run-evidence file");
    // The plan gates the *join*: the derivation half of the shape lives in the
    // sidecar, so a conformant plan is one whose evidence actually arrived.
    assert_eq!(p.sidecar_entries, p.assertions.len());
}

/// The batching a reviewer works from is on the plan, deterministic, and
/// covers every proposed assertion exactly once. G0 §7.6: the batching key
/// belongs in the output contract, not in the logs.
#[test]
fn the_plan_carries_a_stable_batching_of_every_assertion() {
    let p = plan();
    assert!(!p.batches.is_empty());
    let batched: usize = p.batches.iter().map(|b| b.count).sum();
    assert_eq!(batched, p.assertions.len());
    let ids: BTreeSet<&String> = p.batches.iter().flat_map(|b| b.ids.iter()).collect();
    assert_eq!(ids.len(), p.assertions.len(), "each assertion lands in exactly one batch");
    assert_eq!(p.batches, plan().batches, "same facts, same batches, same order");
    // And it survives serialisation: the reviewer's key is not `serde(skip)`.
    let json = serde_json::to_string(&p).expect("serialize");
    assert!(json.contains("\"batches\""), "the batching must reach the JSON");
    assert!(json.contains("\"relevance\""), "both marks must reach the JSON");
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
    assert_eq!(fk.row.confidence.as_str(), "not-derivable");
    assert_eq!(fk.row.relevance_basis.as_str(), "not-applicable");
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
    assert_eq!(second.already_ratified, ratified.len());
    // Evidence still covers what the run read: nothing is proposed, and the
    // derivation of every already-ratified assertion is regenerated. That is
    // what lets a cohort ratified before the evidence layer become auditable.
    assert_eq!(second.sidecar_entries, ratified.len());
    assert_eq!(second.files.len(), 1, "the run sidecar, and nothing else");
    assert!(second.files[0].path.starts_with("runs/"), "{:?}", second.files[0].path);
}

#[test]
fn planning_is_reproducible_given_the_same_as_of() {
    let a = plan();
    let b = plan();
    assert_eq!(a.files, b.files, "same facts, same as-of, same bytes");
}

#[test]
fn assertions_are_graded_on_both_axes_and_the_counts_add_up() {
    let p = plan();
    let total: usize = p.by_pair().iter().map(|(_, n)| n).sum();
    assert_eq!(total, p.assertions.len());
    let pairs: Vec<String> = p.by_pair().into_iter().map(|(g, _)| g).collect();
    assert!(pairs.iter().any(|p| p == "high/unknown"), "{pairs:?}");
    assert!(pairs.iter().any(|p| p == "mid/unknown"), "{pairs:?}");
    assert!(pairs.iter().any(|p| p == "low/unknown"), "{pairs:?}");
}

/// The fixture the change turns on, at plan level: not one assertion carries
/// a confidence without a relevance, or the reverse.
#[test]
fn no_assertion_carries_one_mark_without_the_other() {
    for a in plan().assertions {
        assert!(!a.confidence.as_str().is_empty(), "{}", a.id);
        assert_eq!(a.relevance.as_str(), "unknown", "{}", a.id);
        assert_ne!(a.relevance_basis.as_str(), "computed", "{}", a.id);
        assert!(!a.derivation.rule.is_empty(), "{}", a.id);
        assert!(!a.derivation.stands_on.is_empty(), "{}", a.id);
    }
}

/// No proposal may carry `inferred` provenance at declared assurance — the
/// overclaim per-triple review exists to catch, checked mechanically.
#[test]
fn no_inferred_assertion_claims_declared_assurance() {
    for a in plan().assertions {
        if a.provenance == crate::ground::rows::Provenance::Inferred {
            assert_ne!(
                a.confidence,
                crate::ground::axes::DerivationConfidence::High,
                "{}",
                a.triple.to_value()
            );
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
