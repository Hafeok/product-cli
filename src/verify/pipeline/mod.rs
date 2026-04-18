//! Unified six-stage verify pipeline (FT-044, ADR-040).
//!
//! Runs all six stages in order:
//!
//! 1. Log integrity   — wraps `product request log verify`
//! 2. Graph structure — wraps `product graph check`
//! 3. Schema          — `schema-version` compatibility check
//! 4. Metrics         — wraps `product metrics threshold`
//! 5. Feature TCs     — iterates features, runs TCs per feature
//! 6. Platform TCs    — wraps `product verify --platform`
//!
//! The pipeline never short-circuits: every stage runs regardless of earlier
//! failures, so one invocation produces one complete report. The exit code is
//! the worst across all stages: 0 pass, 1 error, 2 warning.

mod render;
mod stage_features;
mod stage_graph;
mod stage_log;
mod stage_metrics;
mod stage_platform;
mod stage_schema;
mod types;

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use std::path::Path;

pub use render::{render_json, render_pretty};
pub use types::{Finding, PipelineResult, PipelineScope, StageResult, StageStatus};

/// Run the unified pipeline. Never short-circuits — every stage runs.
pub fn run_all(
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
    scope: &PipelineScope,
) -> PipelineResult {
    let stages = vec![
        run_stage_safe(1, "log-integrity", || stage_log::run(config, root)),
        run_stage_safe(2, "graph-structure", || stage_graph::run(config, graph)),
        run_stage_safe(3, "schema-validation", || stage_schema::run(config)),
        run_stage_safe(4, "metrics", || stage_metrics::run(config, graph, root)),
        run_stage_safe(5, "feature-tcs", || {
            stage_features::run(config, root, graph, scope)
        }),
        run_stage_safe(6, "platform-tcs", || stage_platform::run(config, root, graph)),
    ];

    let mut result = PipelineResult {
        passed: false,
        exit: 0,
        stages,
    };
    result.exit = result.exit_code();
    result.passed = result.exit == 0;
    result
}

/// Wrap a stage runner with panic handling — a panicking stage is marked
/// `Fail` with a diagnostic and the pipeline continues.
fn run_stage_safe<F>(stage_num: u8, name: &'static str, f: F) -> StageResult
where
    F: FnOnce() -> StageResult + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(r) => r,
        Err(_) => StageResult {
            stage: stage_num,
            name,
            status: StageStatus::Fail,
            findings: vec![Finding::Code("PANIC".into())],
            summary: "stage panicked".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_status_merge_fail_wins() {
        assert_eq!(
            StageStatus::Pass.merge(StageStatus::Fail),
            StageStatus::Fail
        );
        assert_eq!(
            StageStatus::Warning.merge(StageStatus::Fail),
            StageStatus::Fail
        );
    }

    #[test]
    fn stage_status_merge_warning_beats_pass() {
        assert_eq!(
            StageStatus::Pass.merge(StageStatus::Warning),
            StageStatus::Warning
        );
    }

    #[test]
    fn exit_code_worst_across_stages() {
        let r = PipelineResult {
            passed: false,
            exit: 0,
            stages: vec![
                StageResult {
                    stage: 1,
                    name: "log-integrity",
                    status: StageStatus::Pass,
                    findings: vec![],
                    summary: "".into(),
                },
                StageResult {
                    stage: 2,
                    name: "graph-structure",
                    status: StageStatus::Warning,
                    findings: vec![],
                    summary: "".into(),
                },
            ],
        };
        assert_eq!(r.exit_code(), 2);
    }
}
