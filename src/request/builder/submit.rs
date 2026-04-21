//! Submit — the atomic validate-then-apply-then-archive pipeline for a draft.

use super::add_helpers::validate_draft;
use super::draft::Draft;
use crate::config::ProductConfig;
use crate::request::{apply_request, parse_request_str, ApplyOptions, ApplyResult, Finding};
use std::path::Path;

/// How the builder should treat W-class findings at submit time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarnPolicy {
    /// Always submit through warnings.
    Always,
    /// Prompt interactively (default); when non-interactive, behave as `Block`.
    Warn,
    /// Treat W-class as E-class — refuse submit.
    Block,
}

impl WarnPolicy {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "always" => Some(Self::Always),
            "warn" => Some(Self::Warn),
            "block" => Some(Self::Block),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubmitOptions {
    /// `--force`: submit through warnings without prompting.
    pub force: bool,
}

pub struct SubmitOutcome {
    pub result: ApplyResult,
    pub archived_to: Option<std::path::PathBuf>,
    pub blocked_reason: Option<String>,
}

/// Submit pipeline:
///
/// 1. Structural validation on the draft. E-class → refuse, leave draft.
/// 2. Apply the request via the existing `apply_request` pipeline. If apply
///    reports `applied = false`, leave the draft untouched.
/// 3. On success, archive the draft (rename → `archive/<ts>-draft.yaml`).
pub fn submit(
    draft: &Draft,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
    root: &Path,
    options: SubmitOptions,
    warn_policy: WarnPolicy,
) -> SubmitOutcome {
    let pre_findings = validate_draft(draft, config, graph);
    if pre_findings.iter().any(|f| f.is_error()) {
        return SubmitOutcome {
            result: empty_result(pre_findings),
            archived_to: None,
            blocked_reason: Some("E-class finding in draft".into()),
        };
    }
    // Reason is required at submit time even though incremental builds tolerate empty.
    if draft.reason().trim().is_empty() {
        let finding = Finding::error(
            "E011",
            "request 'reason' is required and must not be empty",
            "$.reason",
        );
        return SubmitOutcome {
            result: empty_result(vec![finding]),
            archived_to: None,
            blocked_reason: Some("empty reason".into()),
        };
    }
    // W-class gate per `warn-on-warnings` policy.
    let has_warnings = pre_findings.iter().any(|f| !f.is_error());
    if has_warnings {
        match warn_policy {
            WarnPolicy::Always => {}
            WarnPolicy::Block => {
                return SubmitOutcome {
                    result: empty_result(pre_findings),
                    archived_to: None,
                    blocked_reason: Some(
                        "W-class findings (warn-on-warnings = block)".into(),
                    ),
                };
            }
            WarnPolicy::Warn => {
                if !options.force {
                    return SubmitOutcome {
                        result: empty_result(pre_findings),
                        archived_to: None,
                        blocked_reason: Some(
                            "W-class findings — rerun with --force to submit".into(),
                        ),
                    };
                }
            }
        }
    }

    // Apply via the canonical pipeline — identical to `product request apply`.
    let request = match parse_request_str(&draft.to_yaml()) {
        Ok(r) => r,
        Err(findings) => {
            return SubmitOutcome {
                result: empty_result(findings),
                archived_to: None,
                blocked_reason: Some("parse error".into()),
            };
        }
    };
    let result = apply_request(&request, config, root, ApplyOptions::default());
    if !result.applied {
        return SubmitOutcome {
            result,
            archived_to: None,
            blocked_reason: Some("apply rejected".into()),
        };
    }
    let archived_to = Draft::archive(root).unwrap_or_default();
    SubmitOutcome { result, archived_to, blocked_reason: None }
}

fn empty_result(findings: Vec<Finding>) -> ApplyResult {
    ApplyResult {
        applied: false,
        created: Vec::new(),
        changed: Vec::new(),
        findings,
        graph_check_clean: false,
    }
}
