//! Unit tests for store loading over a temp `.ddd/` tree.

use std::path::Path;

use super::*;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

fn demo_store(root: &Path) {
    write(
        root,
        ".ddd/predicates/shape.yaml",
        "predicate:\n  id: pred/data/shape\n  format: 1\n  name: Shape\n  statement: accept c iff it conforms\n  artifact_class: data\n  ground:\n    - fact: schema\n      provenance: controlled\n  tolerance: exact\n  rejection_witness: field path\n",
    );
    write(
        root,
        ".ddd/claims/DDD-cat-01.yaml",
        "format: 1\nid: DDD-cat-01\nstatement: strict mode closes shape\nstatus: reported\nevidence: exercised\nfalsifier: a rejected conforming payload\nowner: none\nchanged: 2026-07-30\n",
    );
    write(
        root,
        ".ddd/decisions/adopt.yaml",
        "format: 1\nid: dec/x/adopt\ntitle: Adopt\nrationale: because\nprincipal: Emil\nbased_on: [DDD-cat-01]\n",
    );
    write(
        root,
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/x/adopt\n",
    );
    write(root, ".ddd/config.yaml", "format: 1\nintercept: warn\n");
}

#[test]
fn loads_every_entry_kind() {
    let tmp = tempfile::tempdir().expect("tempdir");
    demo_store(tmp.path());
    let store = load(&tmp.path().join(".ddd"));
    assert!(store.schema_violations.is_empty(), "{:?}", store.schema_violations);
    assert_eq!(store.predicates.len(), 1);
    assert_eq!(store.claims.len(), 1);
    assert_eq!(store.decisions.len(), 1);
    assert_eq!(store.manifests.len(), 1);
    assert_eq!(store.entry_count(), 4);
    assert_eq!(store.config.as_ref().map(|c| c.intercept.as_str()), Some("warn"));
}

#[test]
fn a_statused_predicate_faults_with_the_rule_message() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(
        tmp.path(),
        ".ddd/predicates/bad.yaml",
        "predicate:\n  id: pred/x/bad\n  format: 1\n  name: Bad\n  statement: s\n  artifact_class: code\n  ground: []\n  tolerance: t\n  rejection_witness: w\n  status: closed\n",
    );
    let store = load(&tmp.path().join(".ddd"));
    assert_eq!(store.schema_violations.len(), 1);
    let v = &store.schema_violations[0];
    assert_eq!(v.focus, "bad.yaml");
    assert_eq!(v.path, "status");
    assert!(v.message.contains("ontology rule 1"), "{}", v.message);
}

#[test]
fn an_unparseable_claim_faults_by_file_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), ".ddd/claims/broken.yaml", "format: 1\nid: [not a string\n");
    let store = load(&tmp.path().join(".ddd"));
    assert_eq!(store.schema_violations.len(), 1);
    assert_eq!(store.schema_violations[0].focus, "broken.yaml");
    assert!(store.claims.is_empty());
}

#[test]
fn find_root_walks_upward() {
    let tmp = tempfile::tempdir().expect("tempdir");
    demo_store(tmp.path());
    let nested = tmp.path().join("a/b");
    std::fs::create_dir_all(&nested).expect("mkdir");
    let found = find_root(&nested).expect("root");
    // Canonicalize both sides: macOS tempdirs traverse /var -> /private/var.
    assert_eq!(
        found.canonicalize().expect("canon"),
        tmp.path().canonicalize().expect("canon")
    );
}

#[test]
fn missing_store_dirs_load_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".ddd")).expect("mkdir");
    let store = load(&tmp.path().join(".ddd"));
    assert_eq!(store.entry_count(), 0);
    assert!(store.schema_violations.is_empty());
}
