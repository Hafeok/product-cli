//! Repo-level `.ddd/config.yaml` — interception mode, ignore globs, plus
//! the format-2 `diff`/`detect` sections.
//!
//! M1 reads the file for validity; the `intercept` mode is consumed by M4's
//! edit interceptor (`enforce | warn | off`, PRD §8) and defaults to `warn`
//! per PRD §11's adoption mitigation. Format 2 adds `diff` (per-finding
//! severity thresholds for `ddd diff`) and `detect` (default SARIF inputs).

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
