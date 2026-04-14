//! Unit tests for authoring sessions (ADR-022)

use super::*;
use crate::types::*;

#[test]
fn default_prompts_not_empty() {
    assert!(!default_prompt(&SessionType::Feature).is_empty());
    assert!(!default_prompt(&SessionType::Adr).is_empty());
    assert!(!default_prompt(&SessionType::Review).is_empty());
}

fn assert_yaml_keys_in_doc(yaml: &str, doc: &str, label: &str) {
    for line in yaml.lines() {
        if let Some(key) = line.split(':').next() {
            let key = key.trim();
            if !key.is_empty() && key != "---" {
                assert!(doc.contains(key), "schema_prompt missing {} field: {}", label, key);
            }
        }
    }
}

#[test]
fn schema_prompt_covers_feature_fields() {
    let doc = schema_prompt();
    let feature = FeatureFrontMatter {
        id: "FT-000".into(),
        title: "t".into(),
        phase: 1,
        status: FeatureStatus::Planned,
        depends_on: vec![],
        adrs: vec![],
        tests: vec![],
        domains: vec![],
        domains_acknowledged: Default::default(),
        bundle: None,
    };
    let yaml = serde_yaml::to_string(&feature).unwrap();
    assert_yaml_keys_in_doc(&yaml, &doc, "feature");
}

#[test]
fn schema_prompt_covers_adr_fields() {
    let doc = schema_prompt();
    let adr = AdrFrontMatter {
        id: "ADR-000".into(),
        title: "t".into(),
        status: AdrStatus::Proposed,
        features: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        domains: vec![],
        scope: AdrScope::FeatureSpecific,
        content_hash: None,
        amendments: vec![],
        source_files: vec![],
    };
    let yaml = serde_yaml::to_string(&adr).unwrap();
    assert_yaml_keys_in_doc(&yaml, &doc, "ADR");
}

#[test]
fn schema_prompt_covers_tc_fields() {
    let doc = schema_prompt();
    let tc = TestFrontMatter {
        id: "TC-000".into(),
        title: "t".into(),
        test_type: TestType::Scenario,
        status: TestStatus::Unimplemented,
        validates: ValidatesBlock { features: vec![], adrs: vec![] },
        phase: 1,
        content_hash: None,
        runner: None,
        runner_args: None,
        runner_timeout: None,
        requires: vec![],
        last_run: None,
        failure_message: None,
        last_run_duration: None,
    };
    let yaml = serde_yaml::to_string(&tc).unwrap();
    assert_yaml_keys_in_doc(&yaml, &doc, "TC");
}
