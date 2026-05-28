//! `[features]` section (FT-055, ADR-047) — feature body completeness check.

use serde::{Deserialize, Serialize};

/// `[features]` section in `product.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// Top-level H2 headings every non-stub feature body must contain.
    #[serde(rename = "required-sections", default = "default_required_sections")]
    pub required_sections: Vec<String>,
    /// H3 headings required under `## Functional Specification`.
    #[serde(rename = "functional-spec-subsections", default = "default_fs_subsections")]
    pub functional_spec_subsections: Vec<String>,
    /// Features with `phase < required-from-phase` are exempt from W030.
    #[serde(rename = "required-from-phase", default = "default_required_from_phase")]
    pub required_from_phase: u32,
    /// Severity of W030 — `warning` (default) or `error`.
    #[serde(rename = "completeness-severity", default)]
    pub completeness_severity: CompletenessSeverity,
    /// FT-073 / ADR-050: severity of the W035 advisory raised when a
    /// non-trivial feature transitions to `in-progress` with zero entries in
    /// `patterns:`. Defaults to `off`; teams that want to enforce pattern
    /// citation set this to `warning` or `error`.
    #[serde(rename = "patterns-required-severity", default)]
    pub patterns_required_severity: PatternsRequiredSeverity,
    /// FT-104: cross-cutting ADRs every feature acknowledges by default.
    /// Per-feature opt-out is via `adrs-rejected:` frontmatter.
    #[serde(rename = "default-acknowledged-cross-cutting", default)]
    pub default_acknowledged_cross_cutting: Vec<String>,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            required_sections: default_required_sections(),
            functional_spec_subsections: default_fs_subsections(),
            required_from_phase: default_required_from_phase(),
            completeness_severity: CompletenessSeverity::default(),
            patterns_required_severity: PatternsRequiredSeverity::default(),
            default_acknowledged_cross_cutting: Vec::new(),
        }
    }
}

fn default_required_sections() -> Vec<String> {
    vec![
        "Description".into(),
        "Functional Specification".into(),
        "Out of scope".into(),
    ]
}

fn default_fs_subsections() -> Vec<String> {
    vec![
        "Inputs".into(),
        "Outputs".into(),
        "State".into(),
        "Behaviour".into(),
        "Invariants".into(),
        "Error handling".into(),
        "Boundaries".into(),
    ]
}

fn default_required_from_phase() -> u32 { 1 }

/// `[features].completeness-severity` (FT-055, ADR-047) — `warning` by default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletenessSeverity {
    #[default]
    Warning,
    Error,
}

/// `[features].patterns-required-severity` (FT-073, ADR-050) — `off` by
/// default. Setting it to `warning` or `error` enables the W035 advisory
/// for in-progress features with empty `patterns:`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternsRequiredSeverity {
    #[default]
    Off,
    Warning,
    Error,
}
