//! `[cycle-times]` section (FT-054, ADR-046) — historical cycle-time config.

use serde::{Deserialize, Serialize};

/// `[cycle-times]` section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimesConfig {
    /// Size of the "recent" sample window. Default 5.
    #[serde(rename = "recent-window", default = "default_recent_window")]
    pub recent_window: usize,
    /// Minimum complete features required to render cycle-time output and
    /// to allow `forecast --naive`. Default 3.
    #[serde(rename = "min-features", default = "default_min_features")]
    pub min_features: usize,
    /// Ratio threshold for the trend classifier. Default 0.25.
    #[serde(rename = "trend-threshold", default = "default_trend_threshold")]
    pub trend_threshold: f64,
}

impl Default for CycleTimesConfig {
    fn default() -> Self {
        Self {
            recent_window: default_recent_window(),
            min_features: default_min_features(),
            trend_threshold: default_trend_threshold(),
        }
    }
}

fn default_recent_window() -> usize { 5 }
fn default_min_features() -> usize { 3 }
fn default_trend_threshold() -> f64 { 0.25 }
