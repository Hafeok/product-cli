//! Unified graph health check — single composition point for CLI + MCP (FT-069, ADR-020).
//!
//! `run(graph, config, root)` is the shared library function that both
//! `product graph check` (CLI) and `product_graph_check` (MCP) delegate to.
//! It composes every validation layer the user-facing CLI exposes:
//! structural (`check_with_config`), domain (`validate_domains`),
//! responsibility (`check_responsibility`), planning
//! (`planning_validation::check_due_dates`), and — when
//! `[log].verify-on-check` is true — request-log verification.
//!
//! Routing both adapters through this function guarantees the parity
//! invariant: a fixture that triggers any validation layer surfaces the
//! same finding regardless of whether the user invoked the CLI or the
//! MCP tool.

use std::path::Path;

use crate::config::ProductConfig;
use crate::error::{CheckResult, Diagnostic};
use crate::graph::KnowledgeGraph;

/// Execute every health-check layer the user-facing `product graph check`
/// exposes and return the consolidated `CheckResult`.
///
/// Layer order:
/// 1. Structural validation (`check_with_config`) — broken links,
///    cycles, formal blocks, content hashes, runner-required, etc.
/// 2. Domain validation (`domains::validate_domains`) — E011, E012,
///    W009, W010.
/// 3. Responsibility validation (`responsibility::check_responsibility`)
///    — W019.
/// 4. Planning validation (`planning_validation::check_due_dates`) —
///    W028, W029, evaluated against `chrono::Local::now().date_naive()`.
/// 5. Request-log verification — appended when
///    `config.log.verify_on_check` is true and the log file exists.
pub fn run(graph: &KnowledgeGraph, config: &ProductConfig, root: &Path) -> CheckResult {
    let mut result = graph.check_with_config(Some(config));

    crate::domains::validate_domains(
        graph,
        &config.domains,
        &mut result.errors,
        &mut result.warnings,
    );

    crate::graph::responsibility::check_responsibility(
        graph,
        config.responsibility(),
        &mut result,
    );

    // FT-053 / ADR-045 — W028 (due-date passed) and W029 (approaching).
    let today = chrono::Local::now().date_naive();
    crate::graph::planning_validation::check_due_dates(
        graph,
        &config.planning,
        today,
        &mut result,
    );

    // FT-042, ADR-039 decision 10: wire log verification into graph check.
    if config.log.verify_on_check {
        append_log_findings(config, root, &mut result);
    }

    result
}

/// Append request-log verification findings to the check result.
/// Mirrors the helper that used to live inside `commands/graph_cmd.rs`.
fn append_log_findings(config: &ProductConfig, root: &Path, result: &mut CheckResult) {
    use crate::request_log::{
        log_path,
        verify::{verify_log, Severity, VerifyOptions},
    };

    let lp = log_path(root, Some(&config.paths.requests));
    if !lp.exists() {
        return;
    }
    let outcome = verify_log(&lp, root, &VerifyOptions::default());
    for f in outcome.findings {
        let mut diag = match f.severity {
            Severity::Error => Diagnostic::error(&f.code, &f.message),
            Severity::Warning => Diagnostic::warning(&f.code, &f.message),
        };
        diag = diag.with_file(lp.clone());
        if let Some(line) = f.line {
            diag = diag.with_line(line);
        }
        if let Some(detail) = f.detail {
            diag = diag.with_detail(&detail);
        }
        match f.severity {
            Severity::Error => result.errors.push(diag),
            Severity::Warning => result.warnings.push(diag),
        }
    }
}
