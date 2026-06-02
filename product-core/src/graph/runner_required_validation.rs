//! E022: TC runner config required for active features (FT-058 / ADR-021).
//!
//! Surfaces the `tc::runner_required::find_offenders` predicate as a
//! `graph check` finding so manual YAML edits that strip a `runner` line
//! are caught in CI even if no developer ran `verify` locally.

use super::model::KnowledgeGraph;
use crate::error::{CheckResult, Diagnostic};

/// Iterate every feature whose status requires runner config; emit one
/// E022 diagnostic per offending TC. The diagnostic carries the TC's
/// file path, names the linked feature, and includes the canonical
/// fix-snippet hint.
pub(crate) fn check(graph: &KnowledgeGraph, result: &mut CheckResult) {
    for f in graph.features.values() {
        if !crate::tc::runner_required::status_requires_runner(f.front.status) {
            continue;
        }
        let offenders =
            crate::tc::runner_required::find_offenders(graph, &f.front.id, f.front.status);
        for tc_id in offenders {
            let path = graph.tests.get(tc_id.as_str()).map(|t| t.path.clone());
            let mut diag = Diagnostic::error("E022", "TC runner configuration missing")
                .with_detail(&format!(
                    "{} (linked to {}) lacks `runner` and/or `runner-args`",
                    tc_id, f.front.id
                ))
                .with_hint(
                    "add the following to the TC's front-matter:\n            runner: cargo-test\n            runner-args: \"tc_XXX_<snake_case_title>\"",
                );
            if let Some(p) = path {
                diag = diag.with_file(p);
            }
            result.errors.push(diag);
        }
    }
}
