//! Tests for the authoring-scope model, both oracles, the completeness join.
//!
//! The oracle cases mirror `authoring-scope/oracles/scope_check.py::_self_test`
//! (the framework package this slice ports), so the Rust port and the reference
//! Python stay behaviourally locked.

use std::collections::{BTreeMap, HashSet};

use crate::pf::authoring_scope::*;
use crate::pf::authoring_scope_enforce::*;
use crate::pf::authoring_scope_join::*;

/// The Figma reference scope (mirrors reference/figma.authoring-scope.json).
fn figma() -> AuthoringScope {
    fn a(kind: &str, c: Completeness, ch: &str) -> Authored {
        Authored { kind: kind.into(), completeness: Some(c), channel: Some(ch.into()) }
    }
    AuthoringScope {
        tool: "figma".into(),
        adapter: "figma-bridge".into(),
        authors: vec![
            a("ui-step", Completeness::Partial, "frame-structure"),
            a("aio", Completeness::Sufficient, "native-annotation"),
            a("aio", Completeness::Partial, "component-name"),
            a("state-annotation", Completeness::Sufficient, "native-annotation"),
            a("page-graph", Completeness::Sufficient, "frame-structure"),
            a("context-of-use", Completeness::Partial, "native-annotation"),
            a("accessibility-criteria", Completeness::Partial, "native-annotation"),
            a("content-reference", Completeness::Partial, "frame-structure"),
            a("token-source", Completeness::Sufficient, "variable-export"),
        ],
        excluded: [
            "domain-structure", "trigger", "command", "event", "view", "decider",
            "projector", "state-space", "journey", "quality-demand", "data-shape",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        process_slice: Some("ui-steps-aios-page-graph".into()),
    }
}

fn bare(tool: &str, adapter: &str, kinds: &[&str]) -> AuthoringScope {
    AuthoringScope {
        tool: tool.into(),
        adapter: adapter.into(),
        authors: kinds
            .iter()
            .map(|k| Authored { kind: (*k).into(), completeness: None, channel: None })
            .collect(),
        excluded: vec![],
        process_slice: None,
    }
}

fn item(kind: &str, element: &str) -> SubmissionItem {
    SubmissionItem { kind: kind.into(), element: Some(element.into()) }
}

fn sample_submission() -> Submission {
    Submission {
        authored: vec![
            item("aio", "tz-field"),
            item("page-graph", "settings-screen"),
            item("decider", "no-overlap-rule"), // excluded!
        ],
        unauthored_candidates: vec![
            item("aio", "mystery-dropdown"), // within scope, unauthored
            item("event", "booking-events"), // outside scope
        ],
    }
}

// --- enforcement (mirrors scope_check.py) -----------------------------------

#[test]
fn out_of_scope_authorship_rejected_decider_via_figma() {
    let (valid, f) = enforce(&figma(), &sample_submission());
    assert!(!valid);
    assert_eq!(f.rejected_out_of_scope.len(), 1);
    assert_eq!(f.rejected_out_of_scope[0].kind, "decider");
}

#[test]
fn in_scope_authorship_accepted() {
    let (_, f) = enforce(&figma(), &sample_submission());
    assert_eq!(f.accepted.len(), 2);
}

#[test]
fn gap_split_within_scope_fix_in_figma() {
    let (_, f) = enforce(&figma(), &sample_submission());
    assert_eq!(f.unauthored_within_scope.len(), 1);
    assert_eq!(f.unauthored_within_scope[0].element.as_deref(), Some("mystery-dropdown"));
}

#[test]
fn gap_split_outside_scope_route_elsewhere() {
    let (_, f) = enforce(&figma(), &sample_submission());
    assert_eq!(f.outside_scope.len(), 1);
    assert_eq!(f.outside_scope[0].element.as_deref(), Some("booking-events"));
}

// --- completeness join (mirrors scope_check.py) -----------------------------

fn join_fixture() -> (Vec<AuthoringScope>, BTreeMap<String, HashSet<String>>, Vec<String>) {
    let pcli = bare(
        "product-cli",
        "native",
        &["domain-structure", "trigger", "command", "event", "view", "decider", "projector", "accessibility-criteria"],
    );
    let required: Vec<String> = ["aio", "page-graph", "accessibility-criteria", "domain-structure", "event", "decider", "journey"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut authored: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    authored.insert("figma".into(), ["aio", "page-graph", "accessibility-criteria"].iter().map(|s| s.to_string()).collect());
    authored.insert("product-cli".into(), ["domain-structure", "event", "decider"].iter().map(|s| s.to_string()).collect());
    (vec![figma(), pcli], authored, required)
}

#[test]
fn join_aio_covered_by_figma() {
    let (scopes, authored, required) = join_fixture();
    let (_, rep) = completeness_join(&required, &scopes, &authored);
    assert_eq!(rep["aio"], Coverage::Covered { by: vec!["figma".into()] });
}

#[test]
fn join_decider_covered_by_product_cli() {
    let (scopes, authored, required) = join_fixture();
    let (_, rep) = completeness_join(&required, &scopes, &authored);
    assert!(rep["decider"].is_covered());
}

#[test]
fn join_criteria_covered_partial_authors_compose() {
    let (scopes, authored, required) = join_fixture();
    let (_, rep) = completeness_join(&required, &scopes, &authored);
    assert!(rep["accessibility-criteria"].is_covered());
}

#[test]
fn join_journey_uncovered_no_connected_author() {
    let (scopes, authored, required) = join_fixture();
    let (_, rep) = completeness_join(&required, &scopes, &authored);
    assert_eq!(rep["journey"].status(), "uncovered");
}

#[test]
fn join_overall_incomplete_because_journey_uncovered() {
    let (scopes, authored, required) = join_fixture();
    let (complete, _) = completeness_join(&required, &scopes, &authored);
    assert!(!complete);
}

#[test]
fn join_connecting_a_journey_author_completes_the_what() {
    let (mut scopes, mut authored, required) = join_fixture();
    scopes.push(bare("miro", "miro-bridge", &["journey"]));
    authored.insert("miro".into(), ["journey"].iter().map(|s| s.to_string()).collect());
    let (complete, _) = completeness_join(&required, &scopes, &authored);
    assert!(complete);
}

#[test]
fn coverable_but_unauthored_when_scope_includes_but_none_authored() {
    // A required kind a connected tool could author, but no one has.
    let required = vec!["token-source".to_string()];
    let authored: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let (complete, rep) = completeness_join(&required, &[figma()], &authored);
    assert_eq!(rep["token-source"].status(), "coverable-but-unauthored");
    assert!(!complete);
}

// --- model + validation -----------------------------------------------------

#[test]
fn yaml_round_trips() {
    let s = figma();
    assert_eq!(AuthoringScope::from_yaml(&s.to_yaml().expect("to")).expect("from"), s);
}

#[test]
fn the_figma_reference_scope_is_valid() {
    assert!(validate_scope(&figma()).is_empty());
}

#[test]
fn a_derived_kind_in_authors_is_flagged() {
    let mut s = bare("bad", "bad-bridge", &["aio"]);
    s.authors.push(Authored { kind: "state-space".into(), completeness: None, channel: None });
    let findings = validate_scope(&s);
    assert!(findings.iter().any(|v| v.message.contains("DERIVED") && v.message.contains("state-space")));
}

#[test]
fn an_unknown_kind_is_flagged() {
    let s = bare("bad", "bad-bridge", &["not-a-real-kind"]);
    assert!(validate_scope(&s).iter().any(|v| v.path == "authors"));
}

#[test]
fn a_scope_without_authors_is_flagged() {
    let s = bare("empty", "empty-bridge", &[]);
    assert!(validate_scope(&s).iter().any(|v| v.path == "authors"));
}
