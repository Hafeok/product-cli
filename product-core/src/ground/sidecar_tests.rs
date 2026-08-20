//! Fixtures for the per-run evidence layer.

use super::*;
use crate::ground::derivation::Derivation;
use crate::ground::facts::{DeclKind, MemberKind, OperationStatus};
use crate::ground::propose::Proposer;
use crate::ground::rows::row;
use crate::ground::triple::{Term, Triple};

const DOC: &[&str] = &["documentSymbol"];

/// A probe where one operation answered and one timed out — the pair the
/// full-solution runs actually produced, minutes apart.
fn probe() -> Vec<OperationStatus> {
    vec![
        OperationStatus {
            operation: "documentSymbol".into(),
            answered: true,
            advertised: Some(true),
            divergence: None,
            detail: "3 symbol(s)".into(),
        },
        OperationStatus {
            operation: "workspaceSymbol".into(),
            answered: false,
            advertised: Some(true),
            divergence: Some("over-advertised".into()),
            detail: "error: language server timeout: workspace/symbol unanswered after 15s".into(),
        },
    ]
}

fn ctx() -> WriteContext {
    WriteContext {
        base_iri: "tag:x,2026:ground/".into(),
        as_of: "2026-08-20T00:00:00Z".into(),
        corpus: "corpus-backend".into(),
        git_ref: "3b0a56b33b2282e45e53c73d3df2026ac506bd01".into(),
        corpus_iri: "tag:x,2026:ground/source-corpus-backend-at-3b0a56b33b22".into(),
        run_iri: "tag:x,2026:ground/extraction-corpus-backend-3b0a56b33b22".into(),
    }
}

fn field_assertion(subject: &str) -> ProposedAssertion {
    let p = Proposer::new("roslyn 5.11.0");
    p.propose(
        row("properties").expect("row"),
        Triple::new(format!("tag:x,2026:ground/{subject}"), "tag:x,2026:ground/p", Term::plain("v")),
        vec![],
        Derivation::by("declared-member", DOC)
            .in_container(DeclKind::Class, "src/Converters/Foo.cs")
            .of_member(MemberKind::Field),
    )
}

/// The forcing argument as a file: the fact that made a whole rejected class
/// unfindable is written down, on the assertion's own node, keyed by the id.
#[test]
fn the_run_file_records_the_member_kind_that_was_missing() {
    let ttl = run_file(&[field_assertion("A")], &probe(), &ctx());
    assert!(ttl.contains("reg:memberKind"), "{ttl}");
    assert!(ttl.contains("reg:field"), "{ttl}");
    assert!(ttl.contains("reg:containerKind"), "{ttl}");
    assert!(ttl.contains("reg:derivedByRule    \"declared-member\""), "{ttl}");
    assert!(ttl.contains("reg:standsOn"), "{ttl}");
    assert!(ttl.contains("reg:sourcePath"), "{ttl}");
}

/// It says what it is. Evidence read as ratified content is the whole reason
/// the two layers are separate files.
#[test]
fn the_run_file_states_that_it_is_not_ratified_content() {
    let ttl = run_file(&[field_assertion("A")], &probe(), &ctx());
    assert!(ttl.contains("NOT ratified content"), "{ttl}");
    assert!(ttl.contains("reg:ExtractionRun"), "{ttl}");
}

/// A row's own fact is not repeated across layers: `reg:mappingRow` is on the
/// canonical assertion, and duplicating it here is how the two drift apart.
#[test]
fn the_run_file_does_not_restate_what_the_assertion_carries() {
    let ttl = run_file(&[field_assertion("A")], &probe(), &ctx());
    assert!(!ttl.contains("reg:mappingRow"), "{ttl}");
    assert!(!ttl.contains("reg:domainRelevance"), "{ttl}");
}

/// Byte-stable: entries sort by id, so a re-run at the same ref writes an
/// identical file rather than a reordered one.
#[test]
fn entries_are_written_in_a_stable_order() {
    let forward = [field_assertion("A"), field_assertion("B"), field_assertion("C")];
    let mut backward = forward.to_vec();
    backward.reverse();
    assert_eq!(run_file(&forward, &probe(), &ctx()), run_file(&backward, &probe(), &ctx()));
}

/// The run file names one run, at one ref, so evidence never floats free of
/// what produced it.
#[test]
fn the_file_name_pins_the_corpus_at_its_ref() {
    let name = file_name("corpus-backend", "3b0a56b33b2282e45e53c73d3df2026ac506bd01");
    assert!(name.starts_with("runs/corpus-backend-at-"), "{name}");
    assert!(name.ends_with(".ttl"), "{name}");
}

/// The join condition, mechanised: an entry naming an assertion the graph does
/// not hold is reported, not assumed away.
#[test]
fn an_entry_with_no_assertion_behind_it_is_an_orphan() {
    let graph: std::collections::BTreeSet<String> =
        ["aaaa".to_string(), "bbbb".to_string()].into_iter().collect();
    let entries = vec!["aaaa".to_string(), "cccc".to_string()];
    let found = orphans("runs/x.ttl", &entries, &graph);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].assertion, "cccc");
    assert_eq!(found[0].run, "runs/x.ttl");
}

/// The ids are read back out of the file the writer produced — the check
/// stands on the artefact, not on an in-memory copy of it.
#[test]
fn entry_ids_round_trip_through_the_written_file() {
    let written = [field_assertion("A"), field_assertion("B")];
    let ttl = run_file(&written, &probe(), &ctx());
    let mut expected: Vec<String> = written.iter().map(|a| a.id.clone()).collect();
    expected.sort();
    assert_eq!(entry_ids(&ttl), expected);
}

/// An instance whose evidence matches its graph reports no orphan; the same
/// instance missing one assertion reports exactly that one.
#[test]
fn orphans_are_counted_per_run_over_an_instance_tree() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path();
    let written = [field_assertion("A"), field_assertion("B")];
    std::fs::create_dir_all(root.join("graphs/canonical")).expect("graphs");
    std::fs::create_dir_all(root.join(RUNS_DIR)).expect("runs");
    for a in &written {
        std::fs::write(root.join("graphs/canonical").join(a.file_name()), "# x\n").expect("write");
    }
    std::fs::write(root.join(RUNS_DIR).join("r.ttl"), run_file(&written, &probe(), &ctx()))
        .expect("run");
    assert!(orphans_in(root).is_empty());

    std::fs::remove_file(root.join("graphs/canonical").join(written[0].file_name()))
        .expect("remove");
    let found = orphans_in(root);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].assertion, written[0].id);
}

/// A probe verdict is a Reading, so it carries the whole §4.1 tuple — value,
/// as-of, provenance, assurance — and not merely a boolean.
#[test]
fn a_probe_verdict_is_written_as_a_reading() {
    let ttl = run_file(&[field_assertion("A")], &probe(), &ctx());
    assert!(ttl.contains("reg:ProbeVerdict , reg:Reading"), "{ttl}");
    assert!(ttl.contains("reg:operation"), "{ttl}");
    assert!(ttl.contains("reg:answered         true"), "{ttl}");
    assert!(ttl.contains("reg:provenance       reg:observed"), "{ttl}");
    assert!(ttl.contains("reg:asOf"), "{ttl}");
    assert!(ttl.contains("not a standing fact about the server"), "{ttl}");
}

/// Both directions on the record: the verdict, and the advertisement it
/// disagreed with. An operation recorded unavailable is legible as such
/// rather than simply missing.
#[test]
fn an_unavailable_verdict_records_what_was_advertised_beside_it() {
    let ttl = run_file(&[field_assertion("A")], &probe(), &ctx());
    assert!(ttl.contains("reg:answered         false"), "{ttl}");
    assert!(ttl.contains("reg:advertised       true"), "{ttl}");
    assert!(ttl.contains("reg:divergence"), "{ttl}");
    assert!(ttl.contains("over-advertised"), "{ttl}");
    assert!(ttl.contains("unavailable — error: language server timeout"), "{ttl}");
}

/// The point of recording verdicts at all: two runs that disagree about one
/// operation differ *visibly*, because each verdict is its own node with its
/// own as-of. A reviewer can tell "this row found nothing" from "this row was
/// gated off that day".
#[test]
fn two_runs_disagreeing_about_an_operation_differ_visibly() {
    let answered = vec![OperationStatus {
        operation: "workspaceSymbol".into(),
        answered: true,
        advertised: Some(true),
        divergence: None,
        detail: "7 symbol(s)".into(),
    }];
    let timed_out: Vec<OperationStatus> =
        probe().into_iter().filter(|o| o.operation == "workspaceSymbol").collect();
    let one = run_file(&[], &answered, &ctx());
    let other = run_file(&[], &timed_out, &ctx());
    assert_ne!(one, other);
    assert!(one.contains("available — 7 symbol(s)"), "{one}");
    assert!(other.contains("unavailable — error"), "{other}");
    // The verdict node is the same IRI in both, so the two readings are
    // comparable rather than merely different.
    assert!(one.contains("reg:probe-corpus-backend-3b0a56b33b22-workspaceSymbol"), "{one}");
    assert!(other.contains("reg:probe-corpus-backend-3b0a56b33b22-workspaceSymbol"), "{other}");
}

/// Verdicts sort by operation, so the file is byte-stable however the probe
/// happened to order them.
#[test]
fn verdicts_are_written_in_a_stable_order() {
    let mut reversed = probe();
    reversed.reverse();
    assert_eq!(run_file(&[], &probe(), &ctx()), run_file(&[], &reversed, &ctx()));
}
