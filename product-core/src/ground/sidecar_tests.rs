//! Fixtures for the per-run evidence layer.

use super::*;
use crate::ground::derivation::Derivation;
use crate::ground::facts::{DeclKind, MemberKind};
use crate::ground::propose::Proposer;
use crate::ground::rows::row;
use crate::ground::triple::{Term, Triple};

const DOC: &[&str] = &["documentSymbol"];

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
    let ttl = run_file(&[field_assertion("A")], &ctx());
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
    let ttl = run_file(&[field_assertion("A")], &ctx());
    assert!(ttl.contains("NOT ratified content"), "{ttl}");
    assert!(ttl.contains("reg:ExtractionRun"), "{ttl}");
}

/// A row's own fact is not repeated across layers: `reg:mappingRow` is on the
/// canonical assertion, and duplicating it here is how the two drift apart.
#[test]
fn the_run_file_does_not_restate_what_the_assertion_carries() {
    let ttl = run_file(&[field_assertion("A")], &ctx());
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
    assert_eq!(run_file(&forward, &ctx()), run_file(&backward, &ctx()));
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
    let ttl = run_file(&written, &ctx());
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
    std::fs::write(root.join(RUNS_DIR).join("r.ttl"), run_file(&written, &ctx())).expect("run");
    assert!(orphans_in(root).is_empty());

    std::fs::remove_file(root.join("graphs/canonical").join(written[0].file_name()))
        .expect("remove");
    let found = orphans_in(root);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].assertion, written[0].id);
}
