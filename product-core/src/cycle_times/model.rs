//! Cycle-time data model — pure types shared across compute/render.

use serde::{Deserialize, Serialize};

/// One row per completed feature (ADR-046).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimeRow {
    pub id: String,
    /// Feature phase (FT front-matter).
    pub phase: u32,
    /// ISO 8601 instant of the `product/FT-XXX/started` tag.
    pub started: String,
    /// ISO 8601 instant of the first `product/FT-XXX/complete` tag.
    pub completed: String,
    /// Elapsed days, rounded to one decimal.
    pub cycle_time_days: f64,
}

/// Row for `--in-progress` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InProgressRow {
    pub id: String,
    pub phase: u32,
    /// ISO 8601 instant of the `product/FT-XXX/started` tag.
    pub started: String,
    pub status: String,
    /// Elapsed days since the started tag (one decimal).
    pub elapsed_days: f64,
}

/// Descriptive stats over a sample (ADR-046 §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

/// Three-state trend classifier (ADR-046 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Accelerating,
    Stable,
    Slowing,
}

impl Trend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Trend::Accelerating => "accelerating",
            Trend::Stable => "stable",
            Trend::Slowing => "slowing",
        }
    }
}

/// Aggregate summary attached to a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_5: Option<Stats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<Stats>,
    /// `None` below 6 complete features (ADR-046 §4).
    pub trend: Option<Trend>,
}

/// Top-level report — what `product cycle-times` renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimeReport {
    pub features: Vec<CycleTimeRow>,
    pub summary: Summary,
}

/// Top-level report for `--in-progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InProgressReport {
    pub features: Vec<InProgressRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_median: Option<f64>,
}

/// Forecast output — single feature or phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaiveForecast {
    pub likely: String,
    pub optimistic: String,
    pub pessimistic: String,
}
