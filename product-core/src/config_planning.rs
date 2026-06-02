//! `[planning]` section (FT-053, ADR-045) — feature due-date advisory warnings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    /// Warning window for W029 "due-date-approaching" in days.
    /// Default 3. Setting to 0 disables W029 entirely.
    #[serde(rename = "due-date-warning-days", default = "default_due_date_warning_days")]
    pub due_date_warning_days: u32,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            due_date_warning_days: default_due_date_warning_days(),
        }
    }
}

fn default_due_date_warning_days() -> u32 { 3 }
