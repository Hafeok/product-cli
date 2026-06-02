//! `[tc-observability]` section (FT-072, ADR-051) — TC observability requirement.
//!
//! Operationalises ADR-051 by configuring the `observes:` front-matter
//! field validation behaviour: which TC types require it, the
//! grandfathering threshold, the body-reference severity, and the
//! custom-surface vocabulary extension.

use serde::{Deserialize, Serialize};

/// `[tc-observability]` section in `product.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcObservabilityConfig {
    /// TCs at `phase >= required_from_phase` must declare `observes:`.
    /// Default 5 — grandfathers every pre-FT-072 TC at phase < 5.
    #[serde(rename = "required-from-phase", default = "default_required_from_phase")]
    pub required_from_phase: u32,
    /// TC types that require a non-empty `observes:` field.
    /// Default `["scenario", "session", "smoke", "contract"]`.
    #[serde(rename = "required-for-types", default = "default_required_for_types")]
    pub required_for_types: Vec<String>,
    /// Extra surface vocabulary, appended to the built-in set
    /// (mirrors `[tc-types].custom`).
    #[serde(default)]
    pub custom: Vec<String>,
    /// Severity of the body-reference check — `warning` (default) or `error`.
    #[serde(rename = "body-reference-severity", default)]
    pub body_reference_severity: BodyReferenceSeverity,
}

impl Default for TcObservabilityConfig {
    fn default() -> Self {
        Self {
            required_from_phase: default_required_from_phase(),
            required_for_types: default_required_for_types(),
            custom: Vec::new(),
            body_reference_severity: BodyReferenceSeverity::default(),
        }
    }
}

fn default_required_from_phase() -> u32 {
    5
}

fn default_required_for_types() -> Vec<String> {
    vec![
        "scenario".into(),
        "session".into(),
        "smoke".into(),
        "contract".into(),
    ]
}

/// `[tc-observability].body-reference-severity` — `warning` by default (FT-072).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyReferenceSeverity {
    #[default]
    Warning,
    Error,
}
