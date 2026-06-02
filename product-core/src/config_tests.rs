//! Unit tests for `ProductConfig` (extracted from `config.rs` for
//! file-length hygiene).

#![cfg(test)]

use crate::config::ProductConfig;

#[test]
fn parse_minimal_config() {
    let config: ProductConfig = toml::from_str("name = \"test-project\"\n").unwrap();
    assert_eq!(config.name, "test-project");
    assert_eq!(config.schema_version, "1");
    assert_eq!(config.prefixes.feature, "FT");
    assert_eq!(config.paths.features, "docs/features");
    assert!(!config.tags.auto_push_tags);
    assert_eq!(config.tags.implementation_depth, 20);
}

#[test]
fn parse_tags_config_explicit() {
    let toml_str = "name = \"test\"\n[tags]\nauto-push-tags = false\nimplementation-depth = 30\n";
    let config: ProductConfig = toml::from_str(toml_str).unwrap();
    assert!(!config.tags.auto_push_tags);
    assert_eq!(config.tags.implementation_depth, 30);
}

#[test]
fn parse_full_config() {
    let toml_str = "name = \"picloud\"\nversion = \"0.1\"\nschema-version = \"1\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[phases]\n1 = \"Cluster Foundation\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n";
    let config: ProductConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.name, "picloud");
    assert_eq!(config.phases.get("1").unwrap(), "Cluster Foundation");
}

#[test]
fn schema_version_forward_error() {
    let cfg: ProductConfig =
        toml::from_str("name = \"test\"\nschema-version = \"99\"\n").unwrap();
    assert!(cfg.check_schema_version().is_err());
}

#[test]
fn parse_product_section_with_responsibility() {
    let cfg: ProductConfig = toml::from_str(
        "name = \"t\"\n[product]\nname = \"picloud\"\nresponsibility = \"A private cloud platform\"\n",
    )
    .unwrap();
    assert_eq!(cfg.product_name(), "picloud");
    assert_eq!(cfg.responsibility().unwrap(), "A private cloud platform");
}

#[test]
fn parse_config_without_product_section() {
    let cfg: ProductConfig = toml::from_str("name = \"test\"\n").unwrap();
    assert_eq!(cfg.product_name(), "test");
    assert!(cfg.responsibility().is_none());
}

#[test]
fn product_name_precedence_and_fallback() {
    let cfg: ProductConfig =
        toml::from_str("name = \"old\"\n[product]\nname = \"new\"\n").unwrap();
    assert_eq!(cfg.product_name(), "new");
    let cfg2: ProductConfig =
        toml::from_str("name = \"fb\"\n[product]\nresponsibility = \"X\"\n").unwrap();
    assert_eq!(cfg2.product_name(), "fb");
}

#[test]
fn validate_product_conjunction() {
    let cfg: ProductConfig = toml::from_str(
        "name = \"t\"\n[product]\nresponsibility = \"A platform and a monitor\"\n",
    )
    .unwrap();
    assert!(!cfg.validate_product_section().is_empty(), "top-level and");
    let cfg2: ProductConfig = toml::from_str(
        "name = \"t\"\n[product]\nresponsibility = \"A platform — no deps, no config\"\n",
    )
    .unwrap();
    assert!(cfg2.validate_product_section().is_empty(), "subordinate ok");
    let cfg3: ProductConfig = toml::from_str("name = \"t\"\n").unwrap();
    assert!(cfg3.validate_product_section().is_empty(), "absent ok");
}
