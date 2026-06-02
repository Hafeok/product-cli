//! `[paths]` configuration — file/directory locations for product artifacts.
//!
//! Defaults preserve the pre-FT-057 layout (`docs/features`, `docs/adrs`, …)
//! so existing repositories continue to work without an explicit `[paths]`
//! section. New repositories created by `product init` and repositories
//! migrated by `product migrate consolidate` write canonical `.product/...`
//! paths explicitly per ADR-048.

use serde::{Deserialize, Serialize};

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
    /// Prompts directory (FT-057, ADR-048). When unset, path consumers fall
    /// back to `benchmarks/prompts` for backward compatibility.
    #[serde(default)]
    pub prompts: Option<String>,
    /// Gap baseline path (FT-057, ADR-048). When unset, path consumers fall
    /// back to `gaps.json` at the repo root.
    #[serde(default)]
    pub gaps: Option<String>,
    /// Pattern artifact directory (FT-070, ADR-050).
    /// Defaults to `docs/patterns`.
    #[serde(default = "default_patterns_path")]
    pub patterns: String,
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
            prompts: None,
            gaps: None,
            patterns: default_patterns_path(),
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
fn default_patterns_path() -> String { "docs/patterns".into() }

impl PathsConfig {
    /// Resolved prompts directory — `[paths].prompts` if set, else
    /// `benchmarks/prompts` (FT-057, ADR-048).
    pub fn prompts_resolved(&self) -> &str {
        self.prompts.as_deref().unwrap_or("benchmarks/prompts")
    }

    /// Resolved gaps baseline path — `[paths].gaps` if set, else `gaps.json`
    /// (FT-057, ADR-048).
    pub fn gaps_resolved(&self) -> &str {
        self.gaps.as_deref().unwrap_or("gaps.json")
    }
}
