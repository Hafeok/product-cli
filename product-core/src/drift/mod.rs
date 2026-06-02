//! Drift detection — spec vs. implementation verification (ADR-023)
//!
//! Structural checks for D003/D004 run locally.
//! Semantic checks for D001/D002 require LLM — stubbed for now.

pub mod check;
pub mod diff;

use crate::error::{ProductError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-export public API
pub use check::{check_adr, resolve_source_files, scan_source};
pub use diff::{diff_for_feature, structural_for_feature, DriftDiff, StructuralReport};

// ---------------------------------------------------------------------------
// Drift types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftSeverity {
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

impl std::fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFinding {
    pub id: String,
    pub code: String,
    pub severity: DriftSeverity,
    pub description: String,
    pub adr_id: String,
    pub source_files: Vec<String>,
    pub suggested_action: String,
    #[serde(default)]
    pub suppressed: bool,
}

// ---------------------------------------------------------------------------
// Drift baseline (drift.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftBaseline {
    #[serde(rename = "schema-version", default = "default_schema")]
    pub schema_version: String,
    #[serde(default)]
    pub suppressions: Vec<DriftSuppression>,
    #[serde(default)]
    pub resolved: Vec<DriftResolved>,
}

fn default_schema() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSuppression {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub suppressed_by: String,
    #[serde(default)]
    pub suppressed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResolved {
    pub id: String,
    #[serde(default)]
    pub resolved_at: String,
}

impl DriftBaseline {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ProductError::IoError(format!("failed to serialize drift.json: {}", e))
        })?;
        crate::fileops::write_file_atomic(path, &json)
    }

    pub fn is_suppressed(&self, id: &str) -> bool {
        self.suppressions.iter().any(|s| s.id == id)
    }

    pub fn suppress(&mut self, id: &str, reason: &str) {
        if !self.is_suppressed(id) {
            self.suppressions.push(DriftSuppression {
                id: id.to_string(),
                reason: reason.to_string(),
                suppressed_by: String::new(),
                suppressed_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub fn unsuppress(&mut self, id: &str) {
        self.suppressions.retain(|s| s.id != id);
    }
}

#[cfg(test)]
mod tests;
