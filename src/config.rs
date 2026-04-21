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
    /// TC type vocabulary (ADR-042, FT-048).
    #[serde(rename = "tc-types", default)]
    pub tc_types: TcTypesConfig,
    /// Interactive request builder — `[request-builder]` (FT-052, ADR-044).
    #[serde(rename = "request-builder", default)]
    pub request_builder: RequestBuilderConfig,
}

/// `[request-builder]` section (FT-052, ADR-044).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBuilderConfig {
    /// Prompt for missing fields when stdin is a TTY.
    #[serde(default = "default_true")]
    pub interactive: bool,
    /// How to handle W-class findings at submit time: always | warn | block.
    #[serde(rename = "warn-on-warnings", default = "default_warn_policy")]
    pub warn_on_warnings: String,
    /// Optional `$EDITOR` override used by `product request edit`.
    #[serde(default)]
    pub editor: Option<String>,
}

impl Default for RequestBuilderConfig {
    fn default() -> Self {
        Self {
            interactive: true,
            warn_on_warnings: default_warn_policy(),
            editor: None,
        }
    }
}

fn default_warn_policy() -> String { "warn".into() }

/// `[tc-types]` section (ADR-042). Reserved structural names must not appear
/// in `custom`; that is enforced as E017 at startup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TcTypesConfig {
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
    /// Load product.toml from a path. Runs E017 immediately (ADR-042).
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ProductError::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let config: Self = toml::from_str(&content).map_err(|e| {
            ProductError::ConfigError(format!("Failed to parse {}: {}", path.display(), e))
        })?;
        config.check_tc_types_reserved()?;
        Ok(config)
    }

    /// E017 (ADR-042): reject reserved structural TC-type names in
    /// `[tc-types].custom`. Runs on every `load()` — before any subcommand.
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

    /// Configured custom TC types (`[tc-types].custom`, ADR-042).
    pub fn custom_tc_types(&self) -> &[String] {
        &self.tc_types.custom
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

    /// Is this TC-type value recognised? (ADR-042). See
    /// `crate::test_type::is_known_tc_type`.
    pub fn is_known_tc_type(&self, name: &str) -> bool {
        crate::test_type::is_known_tc_type(&self.tc_types.custom, name)
    }

    /// Hint string listing every recognised TC type (ADR-042).
    pub fn tc_type_hint(&self) -> String {
        crate::test_type::tc_type_hint(&self.tc_types.custom)
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

