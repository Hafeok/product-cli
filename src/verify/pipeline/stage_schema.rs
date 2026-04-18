//! Stage 3 — schema validation (FT-044 step 3).
//!
//! Compares `schema-version` in `product.toml` against the binary's supported
//! schema version. Errors: E008. Warnings: W007.

use super::types::{Finding, StageResult, StageStatus};
use crate::config::ProductConfig;

pub(super) fn run(config: &ProductConfig) -> StageResult {
    match config.check_schema_version() {
        Ok(warnings) if warnings.is_empty() => StageResult {
            stage: 3,
            name: "schema-validation",
            status: StageStatus::Pass,
            findings: vec![],
            summary: "clean".into(),
        },
        Ok(warnings) => StageResult {
            stage: 3,
            name: "schema-validation",
            status: StageStatus::Warning,
            findings: vec![Finding::Code("W007".into())],
            summary: format!("{} upgrade available", warnings.len()),
        },
        Err(_) => StageResult {
            stage: 3,
            name: "schema-validation",
            status: StageStatus::Fail,
            findings: vec![Finding::Code("E008".into())],
            summary: "schema ahead of binary".into(),
        },
    }
}
