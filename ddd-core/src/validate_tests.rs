//! One passing plus one violating case per ontology rule (PRD §6).

use product_core::pf::validate::Violation;

use super::*;
use crate::claim::ClaimStatus;
use crate::decision::{BasedOn, BasisPin, Decision, DecisionKind};
use crate::manifest::{ManifestFile, ManifestRule, ManifestSet, Suppression};
use crate::pattern::PatternInstance;
use crate::predicate::{ArtifactClass, Predicate};

fn pred(id: &str) -> Predicate {
    Predicate {
        id: id.into(),
        format: 1,
        name: "P".into(),
        statement: "accept c iff ok".into(),
        artifact_class: ArtifactClass::Code,
        ground: Vec::new(),
        tolerance: "exact".into(),
        rejection_witness: "w".into(),
        depends_on: Vec::new(),
        obligations: Vec::new(),
        closure_claims: Vec::new(),
        proxy_gap: None,
        notes: None,
    }
}

fn claim(id: &str) -> Claim {
    Claim {
        format: 1,
        id: id.into(),
        statement: "s".into(),
        status: ClaimStatus::Projected,
        evidence: None,
        falsifier: Some("the killing observation".into()),
        owner: "none".into(),
        changed: "2026-08-01".into(),
        revalidate_by: None,
        depends_on: Vec::new(),
        refines: Vec::new(),
        predicate: None,
    }
}

fn decision(id: &str, based_on: &[&str]) -> Decision {
    Decision {
        format: 1,
        id: id.into(),
        kind: DecisionKind::Decision,
        title: "T".into(),
        rationale: "R".into(),
        principal: "Emil".into(),
        based_on: based_on.iter().map(|s| BasedOn::from(*s)).collect(),
        date: None,
        notes: None,
    }
}

fn mrule(rule_id: &str) -> ManifestRule {
    ManifestRule {
        rule_id: rule_id.into(),
        severity: "warning".into(),
        decision: None,
        governance: None,
        suppression: None,
    }
}

fn mset(rules: Vec<ManifestRule>) -> ManifestSet {
    ManifestSet { name: "analyzers".into(), file: ManifestFile { format: 1, rules } }
}

fn pattern(id: &str, predicate: &str) -> PatternInstance {
    PatternInstance {
        format: 1,
        id: id.into(),
        pattern_predicate: predicate.into(),
        instance: "here".into(),
        obligations: Default::default(),
        decision: None,
        notes: None,
    }
}

fn msgs(vs: &[Violation]) -> String {
    vs.iter()
        .map(|v| format!("[{}] {}: {} ({})", v.focus, v.path, v.message, v.severity))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_names(vs: &[Violation], focus: &str, needle: &str) {
    assert!(
        vs.iter().any(|v| v.focus == focus && v.message.contains(needle)),
        "expected a violation on '{focus}' containing '{needle}', got:\n{}",
        msgs(vs)
    );
}

#[test]
fn a_clean_store_is_conformant() {
    let mut s = DddStore::default();
    s.predicates.push(pred("pred/a/x"));
    s.claims.push(claim("DDD-a-1"));
    s.decisions.push(decision("dec/a/adopt", &["DDD-a-1"]));
    let mut r = mrule("CA2007");
    r.decision = Some("dec/a/adopt".into());
    s.manifests.push(mset(vec![r]));
    s.patterns.push(pattern("pat/a/i", "pred/a/x"));
    let vs = validate_store(&s);
    assert!(vs.is_empty(), "{}", msgs(&vs));
}

#[test]
fn rule2_depends_on_cycles_are_named_with_decoded_ids() {
    let mut s = DddStore::default();
    let mut a = pred("pred/a/x");
    a.depends_on = vec!["pred/a/y".into()];
    let mut b = pred("pred/a/y");
    b.depends_on = vec!["pred/a/x".into()];
    s.predicates.extend([a, b]);
    let vs = validate_store(&s);
    assert_names(&vs, "pred/a/x", "acyclic");
    assert_names(&vs, "pred/a/y", "acyclic");
}

#[test]
fn rule2_refines_must_hit_an_existing_claim() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.refines = vec!["DDD-a-9".into()];
    s.claims.push(c);
    let vs = validate_store(&s);
    assert_names(&vs, "DDD-a-1", "refines target 'DDD-a-9' does not exist");

    let mut ok = DddStore::default();
    let mut c1 = claim("DDD-a-1");
    c1.refines = vec!["DDD-a-2".into()];
    ok.claims.extend([c1, claim("DDD-a-2")]);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn rule2_refines_cycles_are_violations() {
    let mut s = DddStore::default();
    let mut c1 = claim("DDD-a-1");
    c1.refines = vec!["DDD-a-2".into()];
    let mut c2 = claim("DDD-a-2");
    c2.refines = vec!["DDD-a-1".into()];
    s.claims.extend([c1, c2]);
    assert_names(&validate_store(&s), "DDD-a-1", "refinement cycle");
}

#[test]
fn rule3_a_decision_without_basis_is_a_violation() {
    let mut s = DddStore::default();
    s.decisions.push(decision("dec/a/orphan", &[]));
    assert_names(&validate_store(&s), "dec/a/orphan", "at least one basedOn");
}

#[test]
fn rule3_a_dangling_basis_is_a_violation() {
    let mut s = DddStore::default();
    s.decisions.push(decision("dec/a/adopt", &["DDD-a-9"]));
    assert_names(&validate_store(&s), "dec/a/adopt", "basedOn claim 'DDD-a-9' does not exist");
}

#[test]
fn rule3_a_blank_principal_is_a_violation() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    let mut d = decision("dec/a/adopt", &["DDD-a-1"]);
    d.principal = "  ".into();
    s.decisions.push(d);
    assert_names(&validate_store(&s), "dec/a/adopt", "name its principal");
}

#[test]
fn rule4_an_ungoverned_entry_warns_without_blocking() {
    let mut s = DddStore::default();
    let mut r = mrule("CA2007");
    r.governance = Some("UNGOVERNED".into());
    s.manifests.push(mset(vec![r]));
    let vs = validate_store(&s);
    assert_names(&vs, "analyzers/CA2007", "UNGOVERNED");
    assert_eq!(blocking(&vs), 0, "{}", msgs(&vs));
}

#[test]
fn rule4_an_unmapped_entry_is_a_violation() {
    let mut s = DddStore::default();
    s.manifests.push(mset(vec![mrule("CA2007")]));
    let vs = validate_store(&s);
    assert_names(&vs, "analyzers/CA2007", "maps to no decision");
    assert_eq!(blocking(&vs), 1);
}

#[test]
fn rule4_a_dangling_governing_decision_is_a_violation() {
    let mut s = DddStore::default();
    let mut r = mrule("CA2007");
    r.decision = Some("dec/a/gone".into());
    s.manifests.push(mset(vec![r]));
    assert_names(&validate_store(&s), "analyzers/CA2007", "'dec/a/gone' does not exist");
}

#[test]
fn rule4_a_suppression_must_cite_a_risk_acceptance() {
    let mut s = DddStore::default();
    let mut r = mrule("CA2007");
    r.decision = Some("dec/a/adopt".into());
    r.suppression = Some(Suppression { risk_acceptance: None, reason: None });
    s.claims.push(claim("DDD-a-1"));
    s.decisions.push(decision("dec/a/adopt", &["DDD-a-1"]));
    s.manifests.push(mset(vec![r]));
    assert_names(&validate_store(&s), "analyzers/CA2007", "must cite a risk-acceptance");
}

#[test]
fn rule4_a_citation_must_resolve_to_a_risk_acceptance() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    s.decisions.push(decision("dec/a/adopt", &["DDD-a-1"]));
    let mut r = mrule("CA2007");
    r.decision = Some("dec/a/adopt".into());
    r.suppression =
        Some(Suppression { risk_acceptance: Some("dec/a/adopt".into()), reason: None });
    s.manifests.push(mset(vec![r]));
    // Cites a decision, not a risk acceptance — still a violation.
    assert_names(&validate_store(&s), "analyzers/CA2007", "cited risk acceptance");

    let mut ok = DddStore::default();
    ok.claims.push(claim("DDD-a-1"));
    ok.decisions.push(decision("dec/a/adopt", &["DDD-a-1"]));
    let mut ra = decision("risk/a/accepted", &[]);
    ra.kind = DecisionKind::RiskAcceptance;
    ok.decisions.push(ra);
    let mut r = mrule("CA2007");
    r.decision = Some("dec/a/adopt".into());
    r.suppression =
        Some(Suppression { risk_acceptance: Some("risk/a/accepted".into()), reason: None });
    ok.manifests.push(mset(vec![r]));
    let vs = validate_store(&ok);
    assert!(vs.is_empty(), "{}", msgs(&vs));
}

#[test]
fn rule5_a_pattern_needs_an_existing_catalog_predicate() {
    let mut s = DddStore::default();
    s.patterns.push(pattern("pat/a/i", "pred/a/gone"));
    assert_names(&validate_store(&s), "pat/a/i", "'pred/a/gone' does not exist");

    let mut blank = DddStore::default();
    blank.patterns.push(pattern("pat/a/j", " "));
    assert_names(&validate_store(&blank), "pat/a/j", "must reference a pattern predicate");
}

#[test]
fn claim_ladder_falsifier_and_evidence_presence() {
    let mut s = DddStore::default();
    let mut live = claim("DDD-a-1");
    live.falsifier = None;
    s.claims.push(live);
    assert_names(&validate_store(&s), "DDD-a-1", "observation that would kill it");

    let mut r = DddStore::default();
    let mut retired = claim("DDD-a-2");
    retired.status = ClaimStatus::Retired;
    retired.falsifier = None;
    retired.evidence = None;
    r.claims.push(retired);
    let vs = validate_store(&r);
    // Retired needs no falsifier but keeps the evidence that killed it.
    assert!(!vs.iter().any(|v| v.path == "falsifier"), "{}", msgs(&vs));
    assert_names(&vs, "DDD-a-2", "what its status rests on");
}

#[test]
fn declared_formats_beyond_v3_are_named() {
    let mut s = DddStore::default();
    let mut p = pred("pred/a/x");
    p.format = 4;
    s.predicates.push(p);
    assert_names(&validate_store(&s), "pred/a/x", "declares format 4");
}

#[test]
fn revalidate_by_under_format_one_is_a_violation() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.revalidate_by = Some("2027-01-01".into());
    s.claims.push(c);
    assert_names(&validate_store(&s), "DDD-a-1", "format-2 field");

    let mut ok = DddStore::default();
    let mut c2 = claim("DDD-a-2");
    c2.format = 2;
    c2.revalidate_by = Some("2027-01-01".into());
    ok.claims.push(c2);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn pinned_edges_under_format_one_are_a_violation() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    let mut d = decision("dec/a/adopt", &[]);
    d.based_on = vec![BasedOn::Pinned(BasisPin {
        claim: "DDD-a-1".into(),
        status: ClaimStatus::Projected,
        changed: "2026-08-01".into(),
    })];
    s.decisions.push(d);
    assert_names(&validate_store(&s), "dec/a/adopt", "format 2");
}

#[test]
fn a_format_two_decision_must_pin_every_edge() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    let mut d = decision("dec/a/adopt", &["DDD-a-1"]);
    d.format = 2;
    s.decisions.push(d);
    assert_names(&validate_store(&s), "dec/a/adopt", "pins every basedOn edge");

    let mut ok = DddStore::default();
    ok.claims.push(claim("DDD-a-1"));
    let mut d2 = decision("dec/a/adopt", &[]);
    d2.format = 2;
    d2.based_on = vec![BasedOn::Pinned(BasisPin {
        claim: "DDD-a-1".into(),
        status: ClaimStatus::Projected,
        changed: "2026-08-01".into(),
    })];
    ok.decisions.push(d2);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn diff_or_detect_sections_under_format_one_are_a_violation() {
    let s = DddStore {
        config: Some(crate::config::DddConfig {
            diff: Some(Default::default()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_names(&validate_store(&s), "config.yaml", "format 2");
}

#[test]
fn duplicate_ids_are_violations() {
    let mut s = DddStore::default();
    s.predicates.push(pred("pred/a/x"));
    s.patterns.push(pattern("pred/a/x", "pred/a/x"));
    assert_names(&validate_store(&s), "pred/a/x", "filed 2 times");
}

#[test]
fn schema_faults_carry_through_validation() {
    let mut s = DddStore::default();
    s.schema_violations.push(Violation {
        focus: "bad.yaml".into(),
        path: "status".into(),
        message: "predicates carry no status (ontology rule 1)".into(),
        severity: "violation".into(),
    });
    assert_names(&validate_store(&s), "bad.yaml", "rule 1");
}
