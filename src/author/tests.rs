//! Unit tests for authoring sessions (ADR-022)

use super::*;
use crate::types::*;

#[test]
fn agent_cli_parse_accepts_claude_and_copilot() {
    assert!(matches!(AgentCli::parse("claude"), Ok(AgentCli::Claude)));
    assert!(matches!(AgentCli::parse("CLAUDE"), Ok(AgentCli::Claude)));
    assert!(matches!(AgentCli::parse("copilot"), Ok(AgentCli::Copilot)));
    assert!(matches!(AgentCli::parse(" Copilot "), Ok(AgentCli::Copilot)));
}

#[test]
fn agent_cli_parse_rejects_unknown() {
    let err = AgentCli::parse("cursor").expect_err("should reject unknown CLI");
    assert!(err.to_string().contains("unknown author.cli value"));
}

#[test]
fn author_config_default_is_claude() {
    let cfg = crate::config::AuthorConfig::default();
    assert_eq!(cfg.cli, "claude");
}

#[test]
fn default_prompts_not_empty() {
    assert!(!prompts::default_content("author-feature").is_empty());
    assert!(!prompts::default_content("author-adr").is_empty());
    assert!(!prompts::default_content("author-review").is_empty());
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
        due_date: None,
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
        removes: vec![],
        deprecates: vec![],
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

#[test]
fn prompts_init_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    let created = prompts::init(dir.path()).unwrap();
    // FT-045 added gap-analysis, drift-analysis, conflict-check.
    assert_eq!(created.len(), 7, "should create all 7 default prompts");
    assert!(dir.path().join("benchmarks/prompts/author-feature-v1.md").exists());
    assert!(dir.path().join("benchmarks/prompts/implement-v1.md").exists());
    assert!(dir.path().join("benchmarks/prompts/gap-analysis-v1.md").exists());
    assert!(dir.path().join("benchmarks/prompts/drift-analysis-v1.md").exists());
    assert!(dir.path().join("benchmarks/prompts/conflict-check-v1.md").exists());
}

#[test]
fn prompts_list_returns_all() {
    let dir = tempfile::tempdir().unwrap();
    let list = prompts::list(dir.path());
    assert_eq!(list.len(), 7);
    assert!(list.iter().any(|p| p.name == "author-feature"));
    assert!(list.iter().any(|p| p.name == "implement"));
    assert!(list.iter().any(|p| p.name == "gap-analysis"));
    assert!(list.iter().any(|p| p.name == "drift-analysis"));
    assert!(list.iter().any(|p| p.name == "conflict-check"));
}

#[test]
fn prompts_get_returns_content() {
    let dir = tempfile::tempdir().unwrap();
    let content = prompts::get(dir.path(), "author-feature").unwrap();
    assert!(content.contains("product_feature_list"));
}

#[test]
fn prompts_get_unknown_errors() {
    let dir = tempfile::tempdir().unwrap();
    assert!(prompts::get(dir.path(), "nonexistent").is_err());
}

#[test]
fn review_adr_content_catches_missing_section() {
    let mut findings = Vec::new();
    let content = "---\nid: ADR-001\nstatus: proposed\nfeatures: [FT-001]\n---\n\n**Context:** c\n**Decision:** d\n**Rationale:** r\n**Test coverage:** t\n";
    review_adr_content("test.md", content, &mut findings);
    assert!(
        findings.iter().any(|f| f.contains("Rejected alternatives")),
        "Should catch missing Rejected alternatives: {:?}",
        findings
    );
}

#[test]
fn review_adr_content_catches_empty_features() {
    let mut findings = Vec::new();
    let content = "---\nid: ADR-001\nstatus: proposed\nfeatures: []\n---\n\n**Context:** c\n**Decision:** d\n**Rationale:** r\n**Rejected alternatives:** n\n**Test coverage:** t\n";
    review_adr_content("test.md", content, &mut findings);
    assert!(
        findings.iter().any(|f| f.contains("W001")),
        "Should emit W001 for empty features: {:?}",
        findings
    );
}
