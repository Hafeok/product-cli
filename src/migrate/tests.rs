//! Unit tests for migration (ADR-017)

use super::*;

#[test]
fn strip_leading_number_works() {
    assert_eq!(helpers::strip_leading_number("5. Products and IAM"), "Products and IAM");
    assert_eq!(helpers::strip_leading_number("12) Storage"), "Storage");
    assert_eq!(helpers::strip_leading_number("No Number"), "No Number");
}

#[test]
fn excluded_headings_detected() {
    assert!(extract::is_excluded_heading("Vision"));
    assert!(extract::is_excluded_heading("Non-Goals"));
    assert!(extract::is_excluded_heading("Core Architecture"));
    assert!(!extract::is_excluded_heading("Cluster Foundation"));
}

#[test]
fn detect_phase() {
    assert_eq!(helpers::detect_phase_heading("### Phase 1 — MVP"), Some(1));
    assert_eq!(helpers::detect_phase_heading("## Phase 3 — RDF"), Some(3));
    assert_eq!(helpers::detect_phase_heading("Some other line"), None);
}

#[test]
fn infer_status() {
    assert_eq!(
        helpers::infer_status_from_body("- [x] done\n- [x] also done\n"),
        crate::types::FeatureStatus::Complete
    );
    assert_eq!(
        helpers::infer_status_from_body("- [x] done\n- [ ] not done\n"),
        crate::types::FeatureStatus::InProgress
    );
    assert_eq!(
        helpers::infer_status_from_body("no checklist here"),
        crate::types::FeatureStatus::Planned
    );
}

#[test]
fn extract_adr_status_works() {
    assert_eq!(
        helpers::extract_adr_status("**Status:** Accepted\n"),
        Some(crate::types::AdrStatus::Accepted)
    );
    assert_eq!(
        helpers::extract_adr_status("**Status:** Proposed\n"),
        Some(crate::types::AdrStatus::Proposed)
    );
    assert_eq!(helpers::extract_adr_status("no status\n"), None);
}

#[test]
fn migrate_prd_detects_features() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("test-prd.md");
    std::fs::write(&source, "# PRD\n\n## Vision\n\nHello.\n\n## Resource Model\n\nStuff.\n\n## Storage Model\n\nMore stuff.\n").unwrap();
    let features_dir = dir.path().join("features");
    let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
    assert_eq!(plan.features.len(), 2, "should detect 2 features (Vision excluded)");
    assert_eq!(plan.features[0].title, "Resource Model");
    assert_eq!(plan.features[1].title, "Storage Model");
}

#[test]
fn migrate_adrs_extracts_tests() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("test-adrs.md");
    std::fs::write(&source, r#"# ADRs

## ADR-001: Rust Language

**Status:** Accepted

Some context.

**Test coverage:**

Scenario tests:
- `binary_compiles.rs` — compiles on ARM64
- `binary_no_deps.rs` — no dynamic deps

Exit criteria:
- Binary size < 20 MB.

---

## ADR-002: YAML Front-Matter

**Status:** Accepted

More context.
"#).unwrap();
    let adrs_dir = dir.path().join("adrs");
    let tests_dir = dir.path().join("tests");
    let plan = migrate_from_adrs(&source, &adrs_dir, &tests_dir, "ADR", "TC").unwrap();
    assert_eq!(plan.adrs.len(), 2, "should extract 2 ADRs");
    assert!(plan.tests.len() >= 2, "should extract test criteria from ADR-001");
    assert_eq!(plan.adrs[0].status, crate::types::AdrStatus::Accepted);
}

#[test]
fn migrate_validate_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("test.md");
    std::fs::write(&source, "# PRD\n\n## Feature One\n\nContent.\n").unwrap();
    let features_dir = dir.path().join("features");
    let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
    // Don't call execute_plan — just verify plan exists and no files were created
    assert_eq!(plan.features.len(), 1);
    assert!(!features_dir.exists(), "features dir should not exist (validate only)");
}

#[test]
fn migrate_execute_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("test.md");
    std::fs::write(&source, "# PRD\n\n## Feature One\n\nContent.\n").unwrap();
    let features_dir = dir.path().join("features");
    let adrs_dir = dir.path().join("adrs");
    let tests_dir = dir.path().join("tests");
    let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
    std::fs::create_dir_all(&features_dir).unwrap();
    let (written, _skipped) = execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, false, false).unwrap();
    assert_eq!(written, 1);
    assert!(features_dir.read_dir().unwrap().count() > 0, "should have created files");
}
