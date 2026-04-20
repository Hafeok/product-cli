//! Unit tests for drift detection (ADR-023)

use super::*;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::path::PathBuf;

#[test]
fn baseline_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("drift.json");
    let mut baseline = DriftBaseline::default();
    baseline.suppress("DRIFT-ADR001-D003-test", "known partial");
    baseline.save(&path).expect("save");

    let loaded = DriftBaseline::load(&path);
    assert_eq!(loaded.suppressions.len(), 1);
}

#[test]
fn extract_source_files() {
    let body = "Some text.\n\nsource-files:\n  - src/consensus/raft.rs\n  - src/consensus/leader.rs\n\nMore text.";
    let files = check::extract_source_files_from_content(body);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], "src/consensus/raft.rs");
}

#[test]
fn scan_finds_adr_references() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("test.rs");
    std::fs::write(&src, "// Implements ADR-002 consensus\nfn leader() {}").expect("write");

    let adr = Adr {
        front: AdrFrontMatter {
            id: "ADR-002".to_string(),
            title: "Consensus".to_string(),
            status: AdrStatus::Accepted,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::FeatureSpecific,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
            removes: vec![],
            deprecates: vec![],
        },
        body: String::new(),
        path: PathBuf::from("adr.md"),
    };
    let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
    let result = scan_source(&src, &graph);
    assert!(result.contains(&"ADR-002".to_string()));
}
