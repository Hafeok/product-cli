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
}

/// Verify prerequisites — named shell conditions (ADR-021)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyConfig {
    #[serde(default)]
    pub prerequisites: HashMap<String, String>,
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
    "AGENTS.md".to_string()
}

fn default_version() -> String {
    "0.1".to_string()
}
fn default_schema_version() -> String {
    "1".to_string()
}
fn default_true() -> bool {
    true
}

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
        }
    }
}

fn default_features_path() -> String { "docs/features".to_string() }
fn default_adrs_path() -> String { "docs/adrs".to_string() }
fn default_tests_path() -> String { "docs/tests".to_string() }
fn default_graph_path() -> String { "docs/graph".to_string() }
fn default_checklist_path() -> String { "docs/checklist.md".to_string() }
fn default_dependencies_path() -> String { "docs/dependencies".to_string() }

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

fn default_mcp_port() -> u16 {
    7777
}

fn default_feature_prefix() -> String {
    "FT".to_string()
}
fn default_adr_prefix() -> String {
    "ADR".to_string()
}
fn default_test_prefix() -> String {
    "TC".to_string()
}
fn default_dep_prefix() -> String {
    "DEP".to_string()
}

/// Current schema version supported by this binary
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

impl ProductConfig {
    /// Load product.toml from a path
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ProductError::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let config: Self = toml::from_str(&content).map_err(|e| {
            ProductError::ConfigError(format!("Failed to parse {}: {}", path.display(), e))
        })?;
        Ok(config)
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
}

/// Run schema migration from current version to CURRENT_SCHEMA_VERSION.
/// Returns (files_updated, files_unchanged).
pub fn migrate_schema(
    root: &Path,
    config: &ProductConfig,
    dry_run: bool,
) -> crate::error::Result<(usize, usize)> {
    let version: u32 = config.schema_version.parse().unwrap_or(0);
    if version >= CURRENT_SCHEMA_VERSION {
        return Ok((0, 0));
    }

    let mut updated = 0;
    let mut unchanged = 0;

    // v0 → v1: add depends-on field to feature files missing it
    if version < 1 {
        let features_dir = config.resolve_path(root, &config.paths.features);
        if features_dir.exists() {
            for entry in std::fs::read_dir(&features_dir)
                .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?
                .flatten()
            {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    let content = std::fs::read_to_string(entry.path())
                        .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;
                    if !content.contains("depends-on:") {
                        if dry_run {
                            println!("  would add depends-on: [] to {}", entry.path().display());
                            updated += 1;
                        } else {
                            // Insert depends-on: [] after status line
                            let new_content = content.replace(
                                "\nstatus:",
                                "\ndepends-on: []\nstatus:",
                            );
                            if new_content != content {
                                crate::fileops::write_file_atomic(&entry.path(), &new_content)?;
                                updated += 1;
                                println!("  updated: {}", entry.path().display());
                            } else {
                                // Try inserting after phase line
                                let new_content2 = content.replace(
                                    "\nphase:",
                                    "\nphase: \ndepends-on: []",
                                );
                                if new_content2 != content {
                                    crate::fileops::write_file_atomic(&entry.path(), &new_content2)?;
                                    updated += 1;
                                } else {
                                    unchanged += 1;
                                }
                            }
                        }
                    } else {
                        unchanged += 1;
                    }
                }
            }
        }

        // Update product.toml schema-version
        if !dry_run {
            let toml_path = root.join("product.toml");
            if toml_path.exists() {
                let content = std::fs::read_to_string(&toml_path)
                    .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;
                let new_content = content.replace(
                    &format!("schema-version = \"{}\"", version),
                    &format!("schema-version = \"{}\"", CURRENT_SCHEMA_VERSION),
                );
                crate::fileops::write_file_atomic(&toml_path, &new_content)?;
                println!("  updated product.toml schema-version to {}", CURRENT_SCHEMA_VERSION);
            }
        }
    }

    Ok((updated, unchanged))
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
    }
    #[test]
    fn parse_full_config() {
        let toml_str = "name = \"picloud\"\nversion = \"0.1\"\nschema-version = \"1\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[phases]\n1 = \"Cluster Foundation\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\n";
        let config: ProductConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.name, "picloud");
        assert_eq!(config.phases.get("1").unwrap(), "Cluster Foundation");
        assert_eq!(config.prefixes.test, "TC");
    }
    #[test]
    fn schema_version_forward_error() {
        let cfg: ProductConfig = toml::from_str("name = \"test\"\nschema-version = \"99\"\n").unwrap();
        assert!(cfg.check_schema_version().is_err());
    }
    fn migrate_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let tp = dir.path().join("product.toml");
        std::fs::write(&tp, "name = \"test\"\nschema-version = \"0\"\n\n[paths]\nfeatures = \"f\"\n").unwrap();
        std::fs::create_dir_all(dir.path().join("f")).unwrap();
        std::fs::write(dir.path().join("f/FT-001-test.md"),
            "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\n\nBody.\n").unwrap();
        (dir, tp)
    }
    #[test]
    fn schema_migrate_v0_dry_run() {
        let (dir, tp) = migrate_fixture();
        let cfg = ProductConfig::load(&tp).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, true).unwrap();
        assert!(updated > 0, "dry-run should report files to update");
        let c = std::fs::read_to_string(dir.path().join("f/FT-001-test.md")).unwrap();
        assert!(!c.contains("depends-on"), "dry-run should not modify files");
    }
    #[test]
    fn schema_migrate_v0_execute() {
        let (dir, tp) = migrate_fixture();
        let cfg = ProductConfig::load(&tp).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, false).unwrap();
        assert!(updated > 0);
        let c = std::fs::read_to_string(dir.path().join("f/FT-001-test.md")).unwrap();
        assert!(c.contains("depends-on"), "file should now have depends-on");
        let t = std::fs::read_to_string(&tp).unwrap();
        assert!(t.contains("schema-version = \"1\""), "product.toml should be bumped to v1");
    }
    #[test]
    fn schema_migrate_already_current() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\nschema-version = \"1\"\n").unwrap();
        let cfg = ProductConfig::load(&dir.path().join("product.toml")).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, false).unwrap();
        assert_eq!(updated, 0, "no migration needed for current version");
    }
}
