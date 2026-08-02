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
