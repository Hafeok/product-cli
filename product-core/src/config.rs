//! product.toml parsing, repository discovery (ADR-014)

use crate::error::{ProductError, Result};
use serde::{Deserialize, Serialize};
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
    /// MCP server settings — `[mcp]`.
    #[serde(default)]
    pub mcp: Option<McpConfig>,
    /// Product identity and responsibility — `[product]`.
    #[serde(default)]
    pub product: Option<ProductSection>,
    /// Authoring-session defaults — `[author]`.
    #[serde(default)]
    pub author: Option<AuthorSection>,
}

pub use crate::config_sections::{AuthorSection, McpConfig, ProductSection};

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
        Ok(config)
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

    /// Configured default agent CLI from `[author].cli`, if any (trimmed,
    /// empty treated as unset). Validation happens at `AgentCli::parse`.
    pub fn author_cli(&self) -> Option<String> {
        self.author
            .as_ref()
            .and_then(|a| a.cli.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}

/// Path to the global user config holding cross-repo authoring defaults:
/// `$XDG_CONFIG_HOME/product/config.toml`, else `$HOME/.config/product/config.toml`.
/// Returns `None` when neither env var is set.
pub fn global_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("product").join("config.toml"));
        }
    }
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("product")
                .join("config.toml")
        })
}

/// Default agent CLI from the global user config's `[author].cli`, if present.
/// Tolerant: a missing or unparseable file yields `None`.
pub fn load_global_author_cli() -> Option<String> {
    let path = global_config_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    parse_global_author_cli(&content)
}

/// Parse `[author].cli` from a global config's TOML text. Uses a minimal schema
/// since the global config omits the repo-only required `name` field. Returns
/// `None` for unparseable input or an absent/empty value.
fn parse_global_author_cli(content: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct GlobalConfig {
        #[serde(default)]
        author: Option<AuthorSection>,
    }
    let global: GlobalConfig = toml::from_str(content).ok()?;
    global
        .author
        .and_then(|a| a.cli)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod author_cli_tests {
    use super::*;

    fn config(toml: &str) -> ProductConfig {
        toml::from_str(toml).expect("parse config")
    }

    #[test]
    fn author_cli_reads_section() {
        let c = config("name = \"app\"\n[author]\ncli = \"copilot\"\n");
        assert_eq!(c.author_cli().as_deref(), Some("copilot"));
    }

    #[test]
    fn author_cli_absent_section_is_none() {
        let c = config("name = \"app\"\n");
        assert_eq!(c.author_cli(), None);
    }

    #[test]
    fn author_cli_empty_value_is_none() {
        let c = config("name = \"app\"\n[author]\ncli = \"  \"\n");
        assert_eq!(c.author_cli(), None);
    }

    #[test]
    fn global_parse_reads_author_without_name() {
        // The global config omits the repo-only required `name` field.
        assert_eq!(
            parse_global_author_cli("[author]\ncli = \"copilot\"\n").as_deref(),
            Some("copilot")
        );
    }

    #[test]
    fn global_parse_tolerates_empty_and_garbage() {
        assert_eq!(parse_global_author_cli(""), None);
        assert_eq!(parse_global_author_cli("not = valid = toml"), None);
        assert_eq!(parse_global_author_cli("[author]\n"), None);
    }
}

