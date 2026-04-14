//! Gap analysis — LLM-driven specification review (ADR-019)
//!
//! Structural gap checks (G003, G006, G007) run locally.
//! Semantic checks (G001, G002, G004, G005) require LLM — stubbed for now.

pub mod baseline;
pub mod check;
pub mod model;

use serde::{Deserialize, Serialize};

// Re-export public API
pub use baseline::{GapBaseline, Resolved, Suppression};
pub use check::{check_adr, check_all, check_changed, gap_id, gap_stats};
pub use model::{parse_model_findings, try_model_analysis, ModelError};

// ---------------------------------------------------------------------------
// Gap types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapSeverity {
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

impl std::fmt::Display for GapSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFinding {
    pub id: String,
    pub code: String,
    pub severity: GapSeverity,
    pub description: String,
    pub affected_artifacts: Vec<String>,
    pub suggested_action: String,
    #[serde(default)]
    pub suppressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReport {
    pub adr: String,
    pub run_date: String,
    pub product_version: String,
    pub findings: Vec<GapFinding>,
    pub summary: GapSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapSummary {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub suppressed: usize,
}

// hex module (avoid adding a dep just for this)
pub(crate) mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests;
