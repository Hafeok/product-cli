//! Unit tests for `why` resolution over an in-memory store.

use super::*;
use crate::manifest::ManifestSet;

const CLAIM: &str = "\
format: 1
id: DDD-a-1
statement: the arrangement closes the predicate
status: reported
evidence: exercised twice
falsifier: a counterexample repo
owner: none
changed: 2026-08-01
";

const DECISION: &str = "\
format: 1
id: dec/a/adopt
title: Adopt the arrangement
rationale: it pays rent
principal: Emil
based_on: [DDD-a-1]
date: 2026-08-02
";

const MANIFEST: &str = "\
format: 1
rules:
  - rule_id: CA2007
    severity: warning
    decision: dec/a/adopt
  - rule_id: BCP037
    severity: error
    governance: UNGOVERNED
";

fn store() -> DddStore {
    let mut s = DddStore::default();
    s.claims.push(serde_yaml::from_str(CLAIM).expect("claim"));
    s.decisions.push(serde_yaml::from_str(DECISION).expect("decision"));
    s.manifests.push(ManifestSet {
        name: "analyzers".into(),
        file: serde_yaml::from_str(MANIFEST).expect("manifest"),
    });
    s
}

#[test]
fn a_decision_renders_its_full_chain() {
    let text = render_why(&store(), "dec/a/adopt").expect("resolve");
    for needle in [
        "decision dec/a/adopt",
        "principal: Emil",
        "rationale: it pays rent",
        "claim DDD-a-1 [reported]",
        "falsifier: a counterexample repo",
    ] {
        assert!(text.contains(needle), "missing '{needle}' in:\n{text}");
    }
}

#[test]
fn a_bare_diagnostic_id_resolves_through_its_decision() {
    let text = render_why(&store(), "CA2007").expect("resolve");
    assert!(text.contains("diagnostic analyzers/CA2007"), "{text}");
    assert!(text.contains("governed by:"), "{text}");
    assert!(text.contains("principal: Emil"), "{text}");
}

#[test]
fn an_ungoverned_diagnostic_says_so() {
    let text = render_why(&store(), "analyzers/BCP037").expect("resolve");
    assert!(text.contains("UNGOVERNED — no decision filed yet"), "{text}");
}

#[test]
fn a_claim_id_renders_the_claim() {
    let text = render_why(&store(), "DDD-a-1").expect("resolve");
    assert!(text.starts_with("claim DDD-a-1 [reported]"), "{text}");
}

#[test]
fn an_unknown_id_resolves_to_none() {
    assert!(render_why(&store(), "dec/a/missing").is_none());
}

// Three-way resolution (dec/ddd/why-resolves-three-ways): a store miss is
// not automatically an unknown id.

/// Detection that knows one configured rule the manifest has never seen.
fn detected_with(ns: &str, rule_id: &str) -> DetectedState {
    let mut d = DetectedState::default();
    d.rules.insert(
        (ns.to_string(), rule_id.to_string()),
        crate::detect::DetectedRule {
            configured: Some(crate::configured::ConfiguredRule {
                rule_id: rule_id.to_string(),
                severity: "warning".into(),
                disabled: false,
                source: ".editorconfig".into(),
                scope: Some("*.cs".into()),
            }),
            emitted: None,
        },
    );
    d
}

#[test]
fn a_detected_rule_with_no_manifest_entry_is_unfiled_not_unknown() {
    let detected = detected_with("analyzers", "CA1848");
    match resolve_why(&store(), Some(&detected), "CA1848") {
        WhyResolution::DetectedUnfiled { namespace, rule_id, sources } => {
            assert_eq!(namespace, "analyzers");
            assert_eq!(rule_id, "CA1848");
            // The same how-detected line `diff` prints.
            assert!(sources.contains(".editorconfig"), "{sources}");
        }
        other => panic!("expected DetectedUnfiled, got {other:?}"),
    }
}

#[test]
fn a_fully_qualified_detected_id_resolves_the_same_way() {
    let detected = detected_with("analyzers", "CA1848");
    assert!(matches!(
        resolve_why(&store(), Some(&detected), "analyzers/CA1848"),
        WhyResolution::DetectedUnfiled { .. }
    ));
}

#[test]
fn a_manifest_entry_still_wins_over_detection() {
    // CA2007 is both detected and filed: the chain must render, not the
    // unfiled branch.
    let detected = detected_with("analyzers", "CA2007");
    match resolve_why(&store(), Some(&detected), "CA2007") {
        WhyResolution::Resolved(text) => assert!(text.contains("governed by:"), "{text}"),
        other => panic!("expected Resolved, got {other:?}"),
    }
}

#[test]
fn an_id_no_source_knows_is_still_not_found() {
    let detected = detected_with("analyzers", "CA1848");
    assert!(matches!(
        resolve_why(&store(), Some(&detected), "CA9999"),
        WhyResolution::NotFound
    ));
}

#[test]
fn without_detection_the_resolution_falls_back_to_not_found() {
    assert!(matches!(resolve_why(&store(), None, "CA1848"), WhyResolution::NotFound));
}
