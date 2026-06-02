//! Parser unit tests — extracted from parser.rs to keep that file under 400 lines.

use super::*;

#[test]
fn test_split_front_matter() {
    let content = "---\nid: FT-001\ntitle: Test\n---\n\nBody content here.";
    let (yaml, body) = split_front_matter(content).unwrap();
    assert!(yaml.contains("id: FT-001"));
    assert_eq!(body, "Body content here.");
}

#[test]
fn test_split_no_front_matter() {
    let content = "No front matter here.";
    assert!(split_front_matter(content).is_none());
}

#[test]
fn test_next_id() {
    let existing = vec!["FT-001".to_string(), "FT-003".to_string()];
    assert_eq!(next_id("FT", &existing), "FT-004");
}

#[test]
fn test_next_id_empty() {
    let existing: Vec<String> = vec![];
    assert_eq!(next_id("ADR", &existing), "ADR-001");
}

#[test]
fn test_id_to_filename() {
    assert_eq!(
        id_to_filename("FT-001", "Cluster Foundation"),
        "FT-001-cluster-foundation.md"
    );
    assert_eq!(
        id_to_filename("ADR-002", "openraft for Consensus"),
        "ADR-002-openraft-for-consensus.md"
    );
}

#[test]
fn test_feature_parse_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("FT-001-test.md");
    let content = "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n";
    std::fs::write(&path, content).unwrap();
    let feature = parse_feature(&path).unwrap();
    assert_eq!(feature.front.id, "FT-001");
    assert_eq!(feature.front.title, "Test Feature");
    assert_eq!(feature.front.status, FeatureStatus::InProgress);
    assert_eq!(feature.front.adrs, vec!["ADR-001"]);
    assert_eq!(feature.body, "Feature body.\n");
}

#[test]
fn test_adr_parse() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ADR-001-test.md");
    let content = "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n";
    std::fs::write(&path, content).unwrap();
    let adr = parse_adr(&path).unwrap();
    assert_eq!(adr.front.id, "ADR-001");
    assert_eq!(adr.front.status, AdrStatus::Accepted);
}

#[test]
fn test_test_parse() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("TC-001-test.md");
    let content = "---\nid: TC-001\ntitle: Test Criterion\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nDescription.\n";
    std::fs::write(&path, content).unwrap();
    let tc = parse_test(&path).unwrap();
    assert_eq!(tc.front.id, "TC-001");
    assert_eq!(tc.front.test_type, TestType::Scenario);
    assert_eq!(tc.front.validates.features, vec!["FT-001"]);
}

#[test]
fn validate_id_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.md");
    assert!(validate_id("FT-001", &path).is_ok());
    assert!(validate_id("ADR-123", &path).is_ok());
    assert!(validate_id("TC-0001", &path).is_ok());
}

#[test]
fn validate_id_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.md");
    assert!(validate_id("bad-id", &path).is_err());
    assert!(validate_id("FT001", &path).is_err());
    assert!(validate_id("FT-1", &path).is_err()); // needs 3+ digits
    assert!(validate_id("", &path).is_err());
}
