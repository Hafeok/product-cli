//! Gap baseline file (gaps.json) — suppression tracking (ADR-019)

use crate::error::{ProductError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapBaseline {
    #[serde(rename = "schema-version", default = "default_schema")]
    pub schema_version: String,
    #[serde(default)]
    pub suppressions: Vec<Suppression>,
    #[serde(default)]
    pub resolved: Vec<Resolved>,
}

fn default_schema() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suppression {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub suppressed_by: String,
    #[serde(default)]
    pub suppressed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolved {
    pub id: String,
    #[serde(default)]
    pub resolved_at: String,
    #[serde(default)]
    pub resolving_commit: String,
}

impl GapBaseline {
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
            ProductError::IoError(format!("failed to serialize gaps.json: {}", e))
        })?;
        crate::fileops::write_file_atomic(path, &json)
    }

    pub fn is_suppressed(&self, gap_id: &str) -> bool {
        self.suppressions.iter().any(|s| s.id == gap_id)
    }

    pub fn suppress(&mut self, gap_id: &str, reason: &str) {
        if !self.is_suppressed(gap_id) {
            self.suppressions.push(Suppression {
                id: gap_id.to_string(),
                reason: reason.to_string(),
                suppressed_by: current_git_commit().unwrap_or_default(),
                suppressed_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub fn unsuppress(&mut self, gap_id: &str) {
        self.suppressions.retain(|s| s.id != gap_id);
    }

    /// Move gaps that were suppressed but are no longer detected to the resolved list
    pub fn update_resolved(&mut self, all_finding_ids: &[String]) {
        let mut newly_resolved = Vec::new();
        self.suppressions.retain(|s| {
            if all_finding_ids.contains(&s.id) {
                true // still detected, keep suppression
            } else {
                newly_resolved.push(Resolved {
                    id: s.id.clone(),
                    resolved_at: chrono::Utc::now().to_rfc3339(),
                    resolving_commit: current_git_commit().unwrap_or_default(),
                });
                false // no longer detected, remove suppression
            }
        });
        self.resolved.extend(newly_resolved);
    }
}

pub(crate) fn current_git_commit() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(format!("git:{}", String::from_utf8_lossy(&output.stdout).trim()))
    } else {
        None
    }
}
