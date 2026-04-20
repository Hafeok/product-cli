//! product.toml parsing, repository discovery (ADR-014)

use crate::error::{ProductError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(rename = "schema-version", default = "default_schema_version")]
    pub schema_version: String,
    #[serde(rename = "schema-version-warning", default = "default_true")]
    pub schema_version_warning: bool,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub phases: HashMap<String, String>,
    #[serde(default)]
    pub prefixes: PrefixConfig,
    #[serde(default)]
    pub mcp: Option<McpConfig>,
    #[serde(default)]
    pub metrics: Option<MetricsConfig>,
    /// Concern domain vocabulary (ADR-025)
    #[serde(default)]
    pub domains: HashMap<String, String>,
    /// Whether checklist.md is added to .gitignore by `product init` (ADR-007)
    #[serde(rename = "checklist-in-gitignore", default = "default_true")]
    pub checklist_in_gitignore: bool,
    /// Agent context generation configuration (ADR-031)
    #[serde(rename = "agent-context", default)]
    pub agent_context: AgentContextConfig,
    /// Verify prerequisites — declarative shell conditions (ADR-021)
    #[serde(default)]
    pub verify: VerifyConfig,
    /// Tag-based implementation tracking configuration (ADR-036)
    #[serde(default)]
    pub tags: TagsConfig,
    /// Product identity and responsibility (FT-039)
    #[serde(default)]
    pub product: Option<ProductSection>,
    /// Request log configuration (FT-042, ADR-039)
    #[serde(default)]
    pub log: LogConfig,
    /// TC type vocabulary — custom descriptive types (ADR-042)
    #[serde(rename = "tc-types", default)]
    pub tc_types: TcTypesConfig,
}

/// `[tc-types]` section — custom descriptive TC types (ADR-042).
///
/// Reserved structural names (`exit-criteria`, `invariant`, `chaos`,
/// `absence`) must never appear in `custom`. That check is E017.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TcTypesConfig {
    /// Custom descriptive type names declared by this project.
    #[serde(default)]
    pub custom: Vec<String>,
}

/// Hash-chained request log configuration — `[log]` in product.toml (FT-042).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// When true, `product graph check` also verifies the log chain (default: true).
    #[serde(rename = "verify-on-check", default = "default_true")]
    pub verify_on_check: bool,
    /// Hash algorithm — `sha256` only for v1.
    #[serde(rename = "hash-algorithm", default = "default_hash_algorithm")]
    pub hash_algorithm: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            verify_on_check: true,
            hash_algorithm: default_hash_algorithm(),
        }
    }
}

fn default_hash_algorithm() -> String { "sha256".to_string() }

/// Product identity section — `[product]` in product.toml (FT-039)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSection {
    /// Product name (overrides top-level `name` if present)
    #[serde(default)]
    pub name: Option<String>,
    /// Single-statement responsibility — what the product is and is not
    #[serde(default)]
    pub responsibility: Option<String>,
}

/// Verify prerequisites — named shell conditions (ADR-021)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyConfig {
    #[serde(default)]
    pub prerequisites: HashMap<String, String>,
}
/// Tag-based implementation tracking configuration (ADR-036)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsConfig {
    #[serde(rename = "auto-push-tags", default)]
    pub auto_push_tags: bool,
    #[serde(rename = "implementation-depth", default = "default_implementation_depth")]
    pub implementation_depth: usize,
}

impl Default for TagsConfig {
    fn default() -> Self { Self { auto_push_tags: false, implementation_depth: 20 } }
}
fn default_implementation_depth() -> usize { 20 }

/// Configuration for AGENTS.md generation (ADR-031)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContextConfig {
    #[serde(rename = "include-repo-state", default = "default_true")]
    pub include_repo_state: bool,
    #[serde(rename = "include-schemas", default = "default_true")]
    pub include_schemas: bool,
    #[serde(rename = "include-domains", default = "default_true")]
    pub include_domains: bool,
    #[serde(rename = "include-tool-guide", default = "default_true")]
    pub include_tool_guide: bool,
    #[serde(rename = "output-file", default = "default_agent_output")]
    pub output_file: String,
}

impl Default for AgentContextConfig {
    fn default() -> Self {
        Self {
            include_repo_state: true,
            include_schemas: true,
            include_domains: true,
            include_tool_guide: true,
            output_file: default_agent_output(),
        }
    }
}

fn default_agent_output() -> String { "AGENTS.md".into() }
fn default_version() -> String { "0.1".to_string() }
fn default_schema_version() -> String { "1".to_string() }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_features_path")]
    pub features: String,
    #[serde(default = "default_adrs_path")]
    pub adrs: String,
    #[serde(default = "default_tests_path")]
    pub tests: String,
    #[serde(default = "default_graph_path")]
    pub graph: String,
    #[serde(default = "default_checklist_path")]
    pub checklist: String,
    #[serde(default = "default_dependencies_path")]
    pub dependencies: String,
    /// Committed request log path (FT-042, ADR-039) — default `requests.jsonl`.
    #[serde(default = "default_requests_path")]
    pub requests: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            features: default_features_path(),
            adrs: default_adrs_path(),
            tests: default_tests_path(),
            graph: default_graph_path(),
            checklist: default_checklist_path(),
            dependencies: default_dependencies_path(),
            requests: default_requests_path(),
        }
    }
}

fn default_features_path() -> String { "docs/features".into() }
fn default_adrs_path() -> String { "docs/adrs".into() }
fn default_tests_path() -> String { "docs/tests".into() }
fn default_graph_path() -> String { "docs/graph".into() }
fn default_checklist_path() -> String { "docs/checklist.md".into() }
fn default_dependencies_path() -> String { "docs/dependencies".into() }
fn default_requests_path() -> String { "requests.jsonl".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixConfig {
    #[serde(default = "default_feature_prefix")]
    pub feature: String,
    #[serde(default = "default_adr_prefix")]
    pub adr: String,
    #[serde(default = "default_test_prefix")]
    pub test: String,
    #[serde(default = "default_dep_prefix")]
    pub dependency: String,
}

impl Default for PrefixConfig {
    fn default() -> Self {
        Self {
            feature: default_feature_prefix(),
            adr: default_adr_prefix(),
            test: default_test_prefix(),
            dependency: default_dep_prefix(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub thresholds: HashMap<String, crate::metrics::ThresholdConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Allow MCP write tools (default false)
    #[serde(default)]
    pub write: bool,
    /// Bearer token for HTTP transport
    #[serde(default)]
    pub token: Option<String>,
    /// Default HTTP port
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    /// Allowed CORS origins for HTTP transport
    #[serde(rename = "cors-origins", default)]
    pub cors_origins: Vec<String>,
}

fn default_mcp_port() -> u16 { 7777 }

fn default_feature_prefix() -> String { "FT".into() }
fn default_adr_prefix() -> String { "ADR".into() }
fn default_test_prefix() -> String { "TC".into() }
fn default_dep_prefix() -> String { "DEP".into() }

/// Current schema version supported by this binary
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

impl ProductConfig {
    /// Load product.toml from a path. Runs E017 validation immediately.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ProductError::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let config: Self = toml::from_str(&content).map_err(|e| {
            ProductError::ConfigError(format!("Failed to parse {}: {}", path.display(), e))
        })?;
        // E017: reserved structural names must never appear in [tc-types].custom.
        config.check_tc_types_reserved()?;
        Ok(config)
    }

    /// E017 — reject configs whose `[tc-types].custom` contains a reserved
    /// structural TC type name. Runs at startup before any command.
    pub fn check_tc_types_reserved(&self) -> Result<()> {
        let reserved = crate::types::TestType::RESERVED;
        let offenders: Vec<String> = self
            .tc_types
            .custom
            .iter()
            .filter(|name| reserved.contains(&name.as_str()))
            .cloned()
            .collect();
        if !offenders.is_empty() {
            return Err(ProductError::ConfigError(format!(
                "error[E017]: reserved TC type name(s) in [tc-types].custom: {}\n   = reserved names: {}\n   = hint: remove the offending entries from product.toml — reserved names drive Product mechanics (phase gate, W004, G002, G009) and cannot be redeclared as custom types",
                offenders.join(", "),
                reserved.join(", "),
            )));
        }
        Ok(())
    }

    /// Find product.toml by walking up from cwd
    pub fn discover() -> Result<(Self, PathBuf)> {
        let mut dir = std::env::current_dir().map_err(|e| {
            ProductError::ConfigError(format!("Cannot determine working directory: {}", e))
        })?;
        loop {
            let candidate = dir.join("product.toml");
            if candidate.exists() {
                let config = Self::load(&candidate)?;
                return Ok((config, dir));
            }
            if !dir.pop() {
                return Err(ProductError::ConfigError(
                    "No product.toml found in current directory or any parent".to_string(),
                ));
            }
        }
    }

    /// Validate schema version compatibility
    pub fn check_schema_version(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        let version: u32 = self.schema_version.parse().unwrap_or(0);

        if version > CURRENT_SCHEMA_VERSION {
            return Err(ProductError::SchemaVersionMismatch {
                declared: version,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }

        if version < CURRENT_SCHEMA_VERSION && self.schema_version_warning {
            warnings.push(format!(
                "warning[W007]: schema upgrade available\n  schema version {} is supported but version {} is current\n  run `product migrate schema` to upgrade (dry-run with --dry-run)",
                version, CURRENT_SCHEMA_VERSION
            ));
        }

        Ok(warnings)
    }

    /// Resolve a relative path from the config against the repo root
    pub fn resolve_path(&self, root: &Path, config_path: &str) -> PathBuf {
        root.join(config_path)
    }

    /// Effective product name: `[product].name` takes precedence over top-level `name`
    pub fn product_name(&self) -> &str {
        self.product
            .as_ref()
            .and_then(|p| p.name.as_deref())
            .unwrap_or(&self.name)
    }

    /// Product responsibility statement, if configured
    pub fn responsibility(&self) -> Option<&str> {
        self.product
            .as_ref()
            .and_then(|p| p.responsibility.as_deref())
            .filter(|s| !s.trim().is_empty())
    }

    /// Return a reference to the configured custom TC type list
    /// (`[tc-types].custom`, ADR-042).
    pub fn custom_tc_types(&self) -> &[String] {
        &self.tc_types.custom
    }

    /// Is this TC-type value recognised — either a built-in (structural or
    /// descriptive) or present in `[tc-types].custom`?
    pub fn is_known_tc_type(&self, name: &str) -> bool {
        use crate::types::TestType;
        TestType::RESERVED.contains(&name)
            || TestType::BUILTIN_DESCRIPTIVE.contains(&name)
            || self.tc_types.custom.iter().any(|s| s == name)
    }

    /// Hint string listing every recognised TC type plus the configured
    /// custom list. Used in E006 diagnostics for unknown type values.
    pub fn tc_type_hint(&self) -> String {
        use crate::types::TestType;
        let builtin = TestType::RESERVED
            .iter()
            .chain(TestType::BUILTIN_DESCRIPTIVE.iter())
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        let custom = if self.tc_types.custom.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                self.tc_types
                    .custom
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "valid TC types:\n     built-in: {}\n     [tc-types].custom in product.toml: {}\n   to accept a new type, run:\n     product request change --type create-and-change --reason \"accept new TC type\" \\\n       --mutation 'set tc-types.custom += [\"<name>\"]'",
            builtin, custom
        )
    }

    /// Validate `[product]` section — warns on top-level conjunction (TC-478)
    pub fn validate_product_section(&self) -> Vec<String> {
        let mut w = Vec::new();
        if let Some(r) = self.responsibility() {
            if crate::graph::responsibility::contains_top_level_conjunction(r) {
                w.push("warning[W019]: product responsibility may describe multiple products\n  = hint: single statement only — top-level \" and \" suggests two products".into());
            }
        }
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let cfg: ProductConfig = toml::from_str("name = \"test\"\nschema-version = \"99\"\n").unwrap();
        assert!(cfg.check_schema_version().is_err());
    }
    #[test]
    fn parse_product_section_with_responsibility() {
        let cfg: ProductConfig = toml::from_str("name = \"t\"\n[product]\nname = \"picloud\"\nresponsibility = \"A private cloud platform\"\n").unwrap();
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
        let cfg: ProductConfig = toml::from_str("name = \"old\"\n[product]\nname = \"new\"\n").unwrap();
        assert_eq!(cfg.product_name(), "new");
        let cfg2: ProductConfig = toml::from_str("name = \"fb\"\n[product]\nresponsibility = \"X\"\n").unwrap();
        assert_eq!(cfg2.product_name(), "fb");
    }
    #[test]
    fn validate_product_conjunction() {
        let cfg: ProductConfig = toml::from_str("name = \"t\"\n[product]\nresponsibility = \"A platform and a monitor\"\n").unwrap();
        assert!(!cfg.validate_product_section().is_empty(), "top-level and");
        let cfg2: ProductConfig = toml::from_str("name = \"t\"\n[product]\nresponsibility = \"A platform — no deps, no config\"\n").unwrap();
        assert!(cfg2.validate_product_section().is_empty(), "subordinate ok");
        let cfg3: ProductConfig = toml::from_str("name = \"t\"\n").unwrap();
        assert!(cfg3.validate_product_section().is_empty(), "absent ok");
    }
}
