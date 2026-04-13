//! TC-P001–P004: Parser robustness property tests (ADR-018)

use proptest::prelude::*;
use product_lib::parser;
use product_lib::types::*;
use std::path::PathBuf;

/// TC-P001: No input causes a panic
/// ∀s:String: parse_frontmatter(s) ≠ panic
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn tc_p001_no_panic_on_arbitrary_input(s in "\\PC{0,500}") {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.md");
        std::fs::write(&path, &s).expect("write");
        // Must not panic — error is fine, panic is not
        let _ = parser::parse_feature(&path);
        let _ = parser::parse_adr(&path);
        let _ = parser::parse_test(&path);
    }
}

/// TC-P002: Valid front-matter round-trips
/// ∀f:Feature: parse(serialise(f)) = f
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn tc_p002_feature_roundtrip(
        phase in 1u32..10,
        title in "[A-Za-z ]{3,30}",
    ) {
        let id = "FT-001".to_string();
        let front = FeatureFrontMatter {
            id: id.clone(),
            title: title.clone(),
            phase,
            status: FeatureStatus::Planned,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
        };
        let body = "Test body content.\n";
        let rendered = parser::render_feature(&front, body);

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("FT-001-test.md");
        std::fs::write(&path, &rendered).expect("write");

        let parsed = parser::parse_feature(&path).expect("parse");
        prop_assert_eq!(&parsed.front.id, &id);
        prop_assert_eq!(&parsed.front.title, &title);
        prop_assert_eq!(parsed.front.phase, phase);
    }
}

/// TC-P004: Malformed input returns structured error, not panic
/// ∀s:InvalidYAML: parse(s) = Err(E001)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn tc_p004_malformed_yaml_returns_error(
        garbage in "[^-]{1,100}",
    ) {
        let content = format!("---\n{}\n---\n\nbody\n", garbage);
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.md");
        std::fs::write(&path, &content).expect("write");

        let result = parser::parse_feature(&path);
        // Must be an error, not a panic
        prop_assert!(result.is_err());
    }
}
