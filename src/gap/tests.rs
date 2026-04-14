//! Unit tests for gap analysis (ADR-019)

use super::*;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::path::PathBuf;

fn make_adr(id: &str, body: &str) -> Adr {
    Adr {
        front: AdrFrontMatter {
            id: id.to_string(),
            title: format!("ADR {}", id),
            status: AdrStatus::Accepted,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::FeatureSpecific,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
        },
        body: body.to_string(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

#[test]
fn gap_id_deterministic() {
    let id1 = gap_id("ADR-001", "G003", &["ADR-001"], "test description");
    let id2 = gap_id("ADR-001", "G003", &["ADR-001"], "test description");
    assert_eq!(id1, id2);
}

#[test]
fn gap_id_format() {
    let id = gap_id("ADR-002", "G001", &["ADR-002"], "missing test");
    assert!(id.starts_with("GAP-ADR-002-G001-"));
    assert!(id.len() > 20); // GAP-ADR-002-G001-XXXX
}

#[test]
fn g003_detected_missing_rejected_alternatives() {
    let adr = make_adr("ADR-001", "**Decision:** Use Rust.\n\n**Rationale:** Fast.\n");
    let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
    let baseline = GapBaseline::default();
    let findings = check_adr(&graph, "ADR-001", &baseline);
    assert!(findings.iter().any(|f| f.code == "G003"), "should detect G003");
}

#[test]
fn g003_not_detected_when_present() {
    let adr = make_adr("ADR-001", "**Rejected alternatives:**\n- Go\n- Python\n");
    let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
    let baseline = GapBaseline::default();
    let findings = check_adr(&graph, "ADR-001", &baseline);
    assert!(!findings.iter().any(|f| f.code == "G003"));
}

#[test]
fn suppression_works() {
    let adr = make_adr("ADR-001", "Just a decision with no other section.\n");
    let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
    let mut baseline = GapBaseline::default();

    let findings = check_adr(&graph, "ADR-001", &baseline);
    assert!(!findings.is_empty());
    let gap_id = &findings[0].id;

    baseline.suppress(gap_id, "known issue");
    let findings2 = check_adr(&graph, "ADR-001", &baseline);
    assert!(findings2.iter().all(|f| f.suppressed || f.id != *gap_id));
}

#[test]
fn baseline_save_load_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gaps.json");

    let mut baseline = GapBaseline::default();
    baseline.suppress("GAP-TEST-001", "test reason");
    baseline.save(&path).expect("save");

    let loaded = GapBaseline::load(&path);
    assert_eq!(loaded.suppressions.len(), 1);
    assert_eq!(loaded.suppressions[0].id, "GAP-TEST-001");
}
