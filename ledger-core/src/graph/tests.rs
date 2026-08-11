//! Determinism of the emission, plus each graph shape provoked.

use crate::testkit;
use crate::verify::{self, Options};

use super::shapes::graph_findings;
use super::turtle::emit;
use super::GraphClass;

fn opts() -> Options {
    Options { gate: None, today: testkit::date("2026-08-10"), blame: false }
}

#[test]
fn two_emissions_of_one_store_are_the_same_bytes() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    assert_eq!(emit(&store), emit(&store));
}

#[test]
fn the_emission_carries_prov_attribution_and_open_basis_tokens() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    let ttl = emit(&store);
    assert!(ttl.contains("prov:wasAttributedTo <mailto:fixture-human@example>"), "{ttl}");
    assert!(ttl.contains("ledger:basedOn \"prd:decision-ledger-prd#4.2.1\""), "{ttl}");
    assert!(ttl.contains("ledger:signsVersion"), "{ttl}");
}

#[test]
fn a_conformant_store_has_no_graph_findings() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    assert_eq!(graph_findings(&store), Vec::new());
}

#[test]
fn g001_a_supersession_edge_to_nowhere() {
    let mut raw = testkit::version();
    raw.supersedes = Some("dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY9".parse().expect("id"));
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    let findings = graph_findings(&store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G001);
    assert!(findings[0].subject.contains("dec:hafeok.ledger/"), "{findings:?}");
    assert!(findings[0].message.contains("01K2C4YQJ3F8M0PT5W7NZ9RDY9"), "{findings:?}");
}

#[test]
fn g002_a_parent_hash_nobody_filed() {
    let mut raw = testkit::version();
    raw.parent = Some(testkit::zero_hash());
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    let findings = graph_findings(&store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G002);
}

#[test]
fn g003_a_version_whose_decision_was_never_introduced() {
    let sealed = testkit::sealed(testkit::version());
    let mut cs = testkit::changeset(vec![sealed], Vec::new());
    cs.decisions.clear();
    let store = testkit::store(cs);
    let findings = graph_findings(&store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G003);
    assert_eq!(findings[0].subject, testkit::decision_id().to_string());
}

#[test]
fn graph_findings_fail_verify_as_a_distinct_stage() {
    let mut raw = testkit::version();
    raw.supersedes = Some("dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY9".parse().expect("id"));
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    let report = verify::verify(&store, &opts());
    assert!(report.findings.is_empty(), "the file stage is clean: {:?}", report.findings);
    assert!(!report.is_conformant(), "the graph stage still fails the run");
    let text = crate::render::render(&report);
    assert!(text.contains("graph stage — 1 finding(s):"), "{text}");
    assert!(text.contains("[G001]"), "{text}");
}

#[test]
fn g004_a_forked_version_chain() {
    // One root, two children claiming it as parent: two writers diverged.
    let root = testkit::sealed(testkit::version());
    let mut left = testkit::version();
    left.parent = Some(root.hash.clone());
    left.statement = "the left writer's revision".into();
    let mut right = testkit::version();
    right.parent = Some(root.hash.clone());
    right.statement = "the right writer's revision".into();
    let store = testkit::store(testkit::changeset(
        vec![root, testkit::sealed(left), testkit::sealed(right)],
        Vec::new(),
    ));
    let findings = graph_findings(&store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G004);
    assert_eq!(findings[0].subject, testkit::decision_id().to_string());
    assert!(findings[0].message.contains("merge --resolve"), "{findings:?}");
}

#[test]
fn a_linear_chain_is_not_a_fork() {
    let root = testkit::sealed(testkit::version());
    let mut child = testkit::version();
    child.parent = Some(root.hash.clone());
    child.statement = "a revision, single-writer".into();
    let store =
        testkit::store(testkit::changeset(vec![root, testkit::sealed(child)], Vec::new()));
    assert_eq!(graph_findings(&store), Vec::new());
}

#[test]
fn a_reconciliation_closes_the_fork_it_names() {
    // The G004 store, plus the arbitration act: a version extending the
    // left tip that names the right tip as merged_from. One tip remains.
    let root = testkit::sealed(testkit::version());
    let mut left = testkit::version();
    left.parent = Some(root.hash.clone());
    left.statement = "the left writer's revision".into();
    let left = testkit::sealed(left);
    let mut right = testkit::version();
    right.parent = Some(root.hash.clone());
    right.statement = "the right writer's revision".into();
    let right = testkit::sealed(right);
    let mut reconciled = left.clone();
    reconciled.parent = Some(left.hash.clone());
    reconciled.merged_from = Some(right.hash.clone());
    let reconciled = testkit::sealed(reconciled);
    let mut cs = testkit::changeset(vec![root, left, right, reconciled], Vec::new());
    cs.format = crate::format::MERGE_FORMAT;
    let store = testkit::store(cs);
    assert_eq!(graph_findings(&store), Vec::new());
}

#[test]
fn g002_also_polices_a_dangling_merged_from() {
    let root = testkit::sealed(testkit::version());
    let mut merged = testkit::version();
    merged.parent = Some(root.hash.clone());
    merged.merged_from = Some(testkit::zero_hash());
    merged.statement = "a reconciliation closing nothing".into();
    let mut cs = testkit::changeset(vec![root, testkit::sealed(merged)], Vec::new());
    cs.format = crate::format::MERGE_FORMAT;
    let findings = graph_findings(&testkit::store(cs));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G002);
    assert!(findings[0].message.contains("closed nothing"), "{findings:?}");
}

#[test]
fn g005_two_live_claimants_superseding_one_decision() {
    let target = testkit::sealed(testkit::version());
    let mut first = testkit::version();
    first.decision = "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY5".parse().expect("id");
    first.statement = "the first claimant".into();
    first.supersedes = Some(testkit::decision_id());
    let mut second = testkit::version();
    second.decision = "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY6".parse().expect("id");
    second.statement = "the second claimant".into();
    second.supersedes = Some(testkit::decision_id());
    let store = testkit::store(testkit::changeset(
        vec![target, testkit::sealed(first), testkit::sealed(second)],
        Vec::new(),
    ));
    let findings = graph_findings(&store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].class, GraphClass::G005);
    assert_eq!(findings[0].subject, testkit::decision_id().to_string());
    assert!(findings[0].message.contains("merge --resolve"), "{findings:?}");
}

#[test]
fn a_withdrawn_supersession_claim_clears_g005() {
    // The arbitration: the losing claimant's next version drops the edge.
    let target = testkit::sealed(testkit::version());
    let mut first = testkit::version();
    first.decision = "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY5".parse().expect("id");
    first.statement = "the first claimant".into();
    first.supersedes = Some(testkit::decision_id());
    let first = testkit::sealed(first);
    let mut second = testkit::version();
    second.decision = "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY6".parse().expect("id");
    second.statement = "the second claimant".into();
    second.supersedes = Some(testkit::decision_id());
    let second = testkit::sealed(second);
    let mut withdrawn = second.clone();
    withdrawn.parent = Some(second.hash.clone());
    withdrawn.supersedes = None;
    let withdrawn = testkit::sealed(withdrawn);
    let store = testkit::store(testkit::changeset(
        vec![target, first, second, withdrawn],
        Vec::new(),
    ));
    assert_eq!(graph_findings(&store), Vec::new());
}

#[test]
fn the_graph_class_set_is_closed_at_five() {
    assert_eq!(super::ALL_GRAPH_CLASSES.len(), 5);
    let codes: Vec<&str> = super::ALL_GRAPH_CLASSES.iter().map(|c| c.code()).collect();
    assert_eq!(codes, ["G001", "G002", "G003", "G004", "G005"]);
    assert!(super::ALL_GRAPH_CLASSES.iter().all(|c| !c.title().is_empty()));
}
