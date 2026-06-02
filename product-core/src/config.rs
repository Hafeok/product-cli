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
    /// Planning annotations — `[planning]` (FT-053, ADR-045).
    #[serde(default)]
    pub planning: crate::config_planning::PlanningConfig,
    /// Cycle-time visibility — `[cycle-times]` (FT-054, ADR-046).
    #[serde(rename = "cycle-times", default)]
    pub cycle_times: CycleTimesConfig,
    #[serde(default)]
    pub author: AuthorConfig,
    /// Feature body completeness — `[features]` (FT-055, ADR-047).
    #[serde(default)]
    pub features: FeaturesConfig,
    /// Per-model context bundle template selection — `[context]`
    /// (FT-063, ADR-049).
    #[serde(default)]
    pub context: ContextConfig,
    /// Pattern artifact body checks — `[patterns]` (FT-070, ADR-050).
    #[serde(default)]
    pub patterns: PatternsConfig,
    /// TC observability requirement — `[tc-observability]` (FT-072, ADR-051).
    #[serde(rename = "tc-observability", default)]
    pub tc_observability: TcObservabilityConfig,
}

pub use crate::config_author::AuthorConfig;
pub use crate::config_cycle_times::CycleTimesConfig;
pub use crate::config_features::{CompletenessSeverity, FeaturesConfig, PatternsRequiredSeverity};
pub use crate::config_observability::{BodyReferenceSeverity, TcObservabilityConfig};
pub use crate::config_paths::PathsConfig;
pub use crate::config_patterns::{PatternBodySeverity, PatternsConfig};
pub use crate::config_planning::PlanningConfig;
pub use crate::config_request_builder::RequestBuilderConfig;
pub use crate::config_sections::{
    AgentContextConfig, ContextConfig, LogConfig, McpConfig, MetricsConfig, PrefixConfig,
    ProductSection, TagsConfig, TcTypesConfig, VerifyConfig,
};

fn default_version() -> String { "0.1".to_string() }
fn default_schema_version() -> String { "1".to_string() }
fn default_true() -> bool { true }

/// Current schema version supported by this binary
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Filenames searched in FT-057 / ADR-048 discovery order:
/// canonical, legacy alias inside `.product/`, then root legacy.
pub const CONFIG_CANDIDATES: [&str; 3] = [
    ".product/config.toml",
    ".product/product.toml",
    "product.toml",
];

/// Find a Product config file in `dir` per FT-057 / ADR-048 discovery order.
pub fn find_config_in_dir(dir: &Path) -> Option<PathBuf> {
    for c in CONFIG_CANDIDATES {
        let p = dir.join(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

impl ProductConfig {
    /// Load the Product config rooted at `root` per FT-057 / ADR-048
    /// discovery order. Returns [`ProductError::ConfigError`] enumerating
    /// the searched filenames when no candidate exists.
    pub fn load_from_root(root: &Path) -> Result<Self> {
        match find_config_in_dir(root) {
            Some(path) => Self::load(&path),
            None => Err(ProductError::ConfigError(format!(
                "No product config file at {}: searched {}",
                root.display(),
                CONFIG_CANDIDATES.join(", "),
            ))),
        }
    }

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

    /// Find a config file by walking up from cwd (FT-057, ADR-048).
    /// Discovery order at each level: `.product/config.toml`,
    /// `.product/product.toml` (legacy alias), `product.toml` (root).
    /// First match wins.
    ///
    /// Honours the `--root` flag and `PRODUCT_ROOT` env var: when either is
    /// set the explicit value short-circuits the walk-up, after validation
    /// (path exists, is a directory, contains `.product/`).
    pub fn discover() -> Result<(Self, PathBuf)> {
        if let Some(resolved) = crate::root::resolve_active()? {
            let candidate = find_config_in_dir(&resolved.path).ok_or_else(|| {
                ProductError::RootNotFound {
                    supplied: resolved.path.clone(),
                    source: resolved.source.as_str(),
                    reason: "no product config file (.product/config.toml, .product/product.toml, or product.toml) in supplied root".to_string(),
                }
            })?;
            let config = Self::load(&candidate)?;
            return Ok((config, resolved.path));
        }
        let mut dir = std::env::current_dir().map_err(|e| {
            ProductError::ConfigError(format!("Cannot determine working directory: {}", e))
        })?;
        loop {
            if let Some(candidate) = find_config_in_dir(&dir) {
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

