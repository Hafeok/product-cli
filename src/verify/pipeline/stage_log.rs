//! Stage 1 — log integrity (FT-044 step 1, ADR-039).
//!
//! Wraps `product request log verify`. Errors: E017/E018 (per-entry hash
//! mismatch / chain break). Warnings: W021 (tag without log entry).

use super::types::{Finding, StageResult, StageStatus};
use crate::config::ProductConfig;
use crate::request_log::{log_path, verify::{verify_log, Severity, VerifyOptions}};
use std::path::Path;

pub(super) fn run(config: &ProductConfig, root: &Path) -> StageResult {
    let lp = log_path(root, Some(&config.paths.requests));
    if !lp.exists() {
        return StageResult {
            stage: 1,
            name: "log-integrity",
            status: StageStatus::Pass,
            findings: vec![],
            summary: "no requests.jsonl — clean".into(),
        };
    }
    let outcome = verify_log(&lp, root, &VerifyOptions { against_tags: true });
    let mut findings: Vec<Finding> = Vec::new();
    let mut status = StageStatus::Pass;
    for f in &outcome.findings {
        findings.push(Finding::Code(f.code.clone()));
        status = match f.severity {
            Severity::Error => status.merge(StageStatus::Fail),
            Severity::Warning => status.merge(StageStatus::Warning),
        };
    }
    let summary = if outcome.findings.is_empty() {
        format!("clean ({} entries, chain intact)", outcome.entry_count)
    } else {
        let n_err = outcome.findings.iter().filter(|f| f.is_error()).count();
        let n_warn = outcome.findings.len() - n_err;
        format!("{} error(s), {} warning(s)", n_err, n_warn)
    };
    StageResult {
        stage: 1,
        name: "log-integrity",
        status,
        findings,
        summary,
    }
}
