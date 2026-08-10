//! Repo-level `.ddd/config.yaml` — interception mode, ignore globs, plus
//! the format-2 `diff`/`detect` sections, plus the format-3 adapter keys.
//!
//! M1 reads the file for validity; the `intercept` mode is consumed by M4's
//! edit interceptor (`enforce | warn | off`, PRD §8) and defaults to `warn`
//! per PRD §11's adoption mitigation. Format 2 adds `diff` (per-finding
//! severity thresholds for `ddd diff`) and `detect` (default SARIF inputs).
//! Format 3 adds `intercept_by_class` (per-artifact-class interception
//! modes over the global default) and `adapter` (per-language behavior
//! switches keyed by language name — the keys are routing data; language
//! *meaning* stays in the ddd-lsp adapters).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The `.ddd/config.yaml` contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DddConfig {
    /// Entry format version: 1, or 2 when `diff`/`detect` are used.
    pub format: u32,
    /// Interception mode: `enforce | warn | off`.
    #[serde(default = "default_intercept")]
    pub intercept: String,
    /// Globs the interceptor plus detection skip.
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Per-finding severity thresholds for `ddd diff` (format 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<DiffConfig>,
    /// Detection inputs for `ddd diff` / `ddd report` (format 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detect: Option<DetectConfig>,
    /// Per-artifact-class interception modes (format 3); classes not
    /// listed fall back to the global `intercept`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intercept_by_class: Option<BTreeMap<String, String>>,
    /// Per-language adapter switches, keyed by adapter language name
    /// (format 3), e.g. `adapter.csharp.internal_is_surface`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<BTreeMap<String, AdapterEntry>>,
    /// The HTML+CSS pair map plus its class-contract thresholds (format 4).
    /// The keys are routing data; what a pair *means* stays in the
    /// `htmlcss` adapter (M7, same rule as `adapter.*`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair: Option<PairConfig>,
}

/// The governed HTML+CSS pairs plus the thresholds the class-contract
/// check reads (format 4). Thresholds are data, never code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairConfig {
    /// The pair units: each is one governed HTML+owned-CSS seam.
    #[serde(default)]
    pub units: Vec<PairUnit>,
    /// Decorative one-off class globs, exempt from the class contract.
    #[serde(default)]
    pub ignore_classes: Vec<String>,
}

/// One governed pair: the HTML documents and the CSS they own, as
/// repo-relative globs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairUnit {
    #[serde(default)]
    pub html: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
}

/// One language's adapter switches (format 3).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterEntry {
    /// Host command override (program + args); defaults live in the adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// Treat `internal` visibility as contract surface (library-repo
    /// posture; default off per `dec/ddd/internal-not-surface`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_is_surface: Option<bool>,
    /// Attribute markers that make a symbol an exported endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exported_attributes: Option<Vec<String>>,
}

/// How each `ddd diff` finding kind is treated (default: `error`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ungoverned: Option<FindingSeverity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale: Option<FindingSeverity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncited_suppression: Option<FindingSeverity>,
}

/// A finding's configured treatment: block, report, or drop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    /// Reported and non-zero exit (the default).
    #[default]
    Error,
    /// Reported, exit stays zero.
    Warn,
    /// Dropped from the report entirely.
    Off,
}

/// Where detection reads emitted diagnostics from.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectConfig {
    /// SARIF files (paths relative to the repo root) ingested by default;
    /// `--sarif` on the CLI appends to this list.
    #[serde(default)]
    pub sarif: Vec<String>,
    /// stylelint JSON output files — the emitted source for the
    /// `stylelint` namespace (format 4).
    #[serde(default)]
    pub stylelint: Vec<String>,
    /// html-validate JSON output files — the emitted source for the
    /// `htmlvalidate` namespace (format 4).
    #[serde(default)]
    pub htmlvalidate: Vec<String>,
    /// Design-token stylesheets: each registered file's custom properties
    /// become the configured source for the `tokens` namespace, so a token
    /// resolves through `why` to its decision (format 4).
    #[serde(default)]
    pub tokens: Vec<String>,
}

impl DiffConfig {
    /// The effective severity for a finding kind key
    /// (`ungoverned | stale | uncited_suppression`).
    pub fn severity_of(&self, kind: &str) -> FindingSeverity {
        let chosen = match kind {
            "ungoverned" => self.ungoverned,
            "stale" => self.stale,
            "uncited_suppression" => self.uncited_suppression,
            _ => None,
        };
        chosen.unwrap_or_default()
    }
}

impl DddConfig {
    /// The effective interception mode for an artifact class: the
    /// per-class entry when present, else the global `intercept`.
    pub fn intercept_for(&self, artifact_class: &str) -> &str {
        self.intercept_by_class
            .as_ref()
            .and_then(|m| m.get(artifact_class))
            .map(String::as_str)
            .unwrap_or(&self.intercept)
    }

    /// The adapter switches for a language, when configured.
    pub fn adapter_entry(&self, language: &str) -> Option<&AdapterEntry> {
        self.adapter.as_ref().and_then(|m| m.get(language))
    }
}

/// Read `.ddd/config.yaml` under `repo_root`, when present and parseable.
pub fn load(repo_root: &std::path::Path) -> Option<DddConfig> {
    let path = repo_root.join(crate::store::STORE_DIR).join("config.yaml");
    let text = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}

fn default_intercept() -> String {
    "warn".to_string()
}

impl Default for DddConfig {
    fn default() -> Self {
        Self {
            format: 1,
            intercept: default_intercept(),
            ignore: Vec::new(),
            diff: None,
            detect: None,
            intercept_by_class: None,
            adapter: None,
            pair: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intercept_defaults_to_warn() {
        let c: DddConfig = serde_yaml::from_str("format: 1\n").expect("parse");
        assert_eq!(c.intercept, "warn");
        assert!(c.ignore.is_empty());
        assert!(c.diff.is_none());
    }

    #[test]
    fn format_two_diff_and_detect_sections_parse() {
        let c: DddConfig = serde_yaml::from_str(
            "format: 2\ndiff:\n  stale: warn\ndetect:\n  sarif:\n    - artifacts/cs.sarif\n",
        )
        .expect("parse");
        let diff = c.diff.expect("diff section");
        assert_eq!(diff.severity_of("stale"), FindingSeverity::Warn);
        assert_eq!(diff.severity_of("ungoverned"), FindingSeverity::Error);
        assert_eq!(c.detect.expect("detect").sarif, vec!["artifacts/cs.sarif"]);
    }
}
