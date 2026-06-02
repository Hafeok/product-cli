//! Auxiliary section types for `product.toml` — kept out of `config.rs` so
//! the main `ProductConfig` struct and the load / discover machinery stay
//! under the 400-line file cap (ADR-043 fitness gate).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `[context]` section — per-model bundle template configuration (FT-063).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Template name to use when `--target NAME` is omitted. Falls back
    /// to the legacy AISP-framed bundle when unset (ADR-049).
    #[serde(rename = "default-target", default)]
    pub default_target: Option<String>,
}

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

fn default_hash_algorithm() -> String {
    "sha256".to_string()
}

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
    fn default() -> Self {
        Self {
            auto_push_tags: false,
            implementation_depth: 20,
        }
    }
}

fn default_implementation_depth() -> usize {
    20
}

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

fn default_agent_output() -> String {
    "AGENTS.md".into()
}

fn default_true() -> bool {
    true
}

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
    /// Pattern artifact prefix (FT-070, ADR-050). Default `PAT`.
    #[serde(default = "default_pattern_prefix")]
    pub pattern: String,
}

impl Default for PrefixConfig {
    fn default() -> Self {
        Self {
            feature: default_feature_prefix(),
            adr: default_adr_prefix(),
            test: default_test_prefix(),
            dependency: default_dep_prefix(),
            pattern: default_pattern_prefix(),
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

fn default_mcp_port() -> u16 {
    7777
}

fn default_feature_prefix() -> String {
    "FT".into()
}

fn default_adr_prefix() -> String {
    "ADR".into()
}

fn default_test_prefix() -> String {
    "TC".into()
}

fn default_dep_prefix() -> String {
    "DEP".into()
}

fn default_pattern_prefix() -> String {
    "PAT".into()
}
