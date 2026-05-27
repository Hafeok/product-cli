//! Step 0a — auto-fill missing TC runner config from filename convention.
//!
//! FT-068: `product implement FT-XXX` runs this step immediately before the
//! Step 0 preflight gate. For every TC linked to the target feature that
//! lacks `runner` or `runner-args`, derive a `tc_<NNN>_<slug>` value from
//! the TC's markdown filename and write `runner: cargo-test`,
//! `runner-args: <derived>`, `runner-timeout: 120s` to the TC's
//! front-matter.
//!
//! The auto-fill is local to the implement command — the five enforcement
//! gates from FT-058 / ADR-021 remain strict. Once the auto-fill writes,
//! Step 0's preflight check passes because runner config is present on
//! every linked TC.
//!
//! Pure functions:
//!
//! - [`derive_runner_args`] — slug derivation from the TC filename.
//! - [`plan_autofill`] — enumerate the writes Step 0a would perform.
//!
//! I/O wrapper:
//!
//! - [`apply_autofill`] — call the existing `tc::runner_config` slice for
//!   each plan to commit the writes through the same code path
//!   `product test runner` uses (preserving FT-042 request-log hash chain
//!   semantics).

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::tc::runner_config::{apply_runner_config, plan_runner_config};
use std::path::{Path, PathBuf};

/// Default values Step 0a writes when auto-filling.
pub const AUTOFILL_RUNNER: &str = "cargo-test";
pub const AUTOFILL_TIMEOUT_SECS: u64 = 120;

/// One TC the auto-fill step would (or did) configure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutofillPlan {
    pub tc_id: String,
    pub tc_path: PathBuf,
    pub derived_args: String,
}

/// Derive the canonical `runner-args` value from a TC's markdown filename.
///
/// Matches the convention used by harness scripts and the E022 error
/// renderer's hint: take the basename, strip `.md`, strip the leading
/// `TC-NNN-` prefix, replace `-` with `_`, prepend `tc_<NNN>_`.
///
/// Falls back to a sanitised form of the TC id (`tc_xxx`) when the
/// filename does not match the convention.
pub fn derive_runner_args(tc_id: &str, tc_path: &Path) -> String {
    let stem = tc_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Strip the TC-NNN- prefix (case-insensitive on TC; numbers vary in
    // width). Find the first '-' after the leading TC- segment.
    if let Some(rest) = strip_tc_prefix(stem) {
        let slug: String = rest.replace('-', "_");
        let num = extract_tc_number(tc_id);
        if !slug.is_empty() && !num.is_empty() {
            return format!("tc_{}_{}", num, slug);
        }
    }

    // Fallback: use the TC id with the same shape but no slug.
    let num = extract_tc_number(tc_id);
    if !num.is_empty() {
        format!("tc_{}", num)
    } else {
        // Last resort: a sanitised id.
        tc_id.replace(['-', ' '], "_").to_lowercase()
    }
}

/// Strip a `TC-NNN-` (or `TC-NNNN-`) prefix from a filename stem. Returns
/// the remainder when the prefix matched, or `None` when it did not.
fn strip_tc_prefix(stem: &str) -> Option<&str> {
    // Expect "TC-" prefix (case-sensitive — the convention writes upper).
    let after_tc = stem.strip_prefix("TC-")?;
    // Find the next '-' after the number block.
    let dash_idx = after_tc.find('-')?;
    // Verify the leading segment is all digits.
    let num_part = &after_tc[..dash_idx];
    if num_part.is_empty() || !num_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(&after_tc[dash_idx + 1..])
}

/// Extract the numeric portion of a TC id (`TC-705` → `705`).
fn extract_tc_number(tc_id: &str) -> String {
    tc_id
        .trim_start_matches("TC-")
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect()
}

/// Enumerate every TC linked to `feature_id` that lacks runner config,
/// pairing each with the runner-args slug Step 0a would write.
///
/// Returns an empty vec when the feature is not in the graph, when it has
/// no linked TCs, or when every linked TC is already fully configured.
pub fn plan_autofill(graph: &KnowledgeGraph, feature_id: &str) -> Vec<AutofillPlan> {
    let Some(feature) = graph.features.get(feature_id) else {
        return Vec::new();
    };

    let mut plans = Vec::new();
    for tc_id in &feature.front.tests {
        let Some(tc) = graph.tests.get(tc_id.as_str()) else {
            // Missing on disk falls through to Step 0's broken-link check.
            continue;
        };
        if has_runner_config(&tc.front.runner, &tc.front.runner_args) {
            continue;
        }
        let derived = derive_runner_args(&tc.front.id, &tc.path);
        plans.push(AutofillPlan {
            tc_id: tc.front.id.clone(),
            tc_path: tc.path.clone(),
            derived_args: derived,
        });
    }
    plans
}

/// Apply a batch of auto-fill plans by delegating to the `tc::runner_config`
/// slice. Writes go through the same code path `product test runner` uses,
/// preserving the FT-042 hash-chain semantics in the request log.
pub fn apply_autofill(
    plans: &[AutofillPlan],
    config: &ProductConfig,
    graph: &KnowledgeGraph,
) -> Result<()> {
    for plan in plans {
        let timeout_str = format!("{}s", AUTOFILL_TIMEOUT_SECS);
        let p = plan_runner_config(
            config,
            graph,
            &plan.tc_id,
            Some(AUTOFILL_RUNNER),
            Some(&plan.derived_args),
            Some(&timeout_str),
            &[],
            &[],
        )
        .map_err(|e| {
            ProductError::ConfigError(format!(
                "step 0a: failed to plan runner config for {}: {}",
                plan.tc_id, e
            ))
        })?;
        apply_runner_config(&p)?;
    }
    Ok(())
}

fn has_runner_config(runner: &Option<String>, args: &Option<String>) -> bool {
    let runner_ok = runner.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
    let args_ok = args.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
    runner_ok && args_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    #[test]
    fn derive_runner_args_strips_tc_prefix_and_replaces_hyphens() {
        let path = PathBuf::from(
            "docs/tests/TC-705-verify-hard-fails-when-in-progress-tc-missing-runner.md",
        );
        let got = derive_runner_args("TC-705", &path);
        assert_eq!(
            got,
            "tc_705_verify_hard_fails_when_in_progress_tc_missing_runner"
        );
    }

    #[test]
    fn derive_runner_args_supports_four_digit_tcs() {
        let path = PathBuf::from("docs/tests/TC-1234-some-thing.md");
        assert_eq!(derive_runner_args("TC-1234", &path), "tc_1234_some_thing");
    }

    #[test]
    fn derive_runner_args_handles_unconventional_filename() {
        let path = PathBuf::from("docs/tests/oddfile.md");
        // Falls back to the TC id since the filename does not match.
        assert_eq!(derive_runner_args("TC-007", &path), "tc_007");
    }

    #[test]
    fn derive_runner_args_handles_missing_extension() {
        let path = PathBuf::from("docs/tests/TC-001-short");
        assert_eq!(derive_runner_args("TC-001", &path), "tc_001_short");
    }

    fn mk_tc(id: &str, runner: Option<&str>, args: Option<&str>, path: &str) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: id.to_string(),
                title: format!("title for {}", id),
                test_type: TestType::Scenario,
                status: TestStatus::Unimplemented,
                validates: ValidatesBlock::default(),
                phase: 1,
                content_hash: None,
                runner: runner.map(String::from),
                runner_args: args.map(String::from),
                runner_timeout: None,
                requires: vec![],
                observes: vec![],
                last_run: None,
                failure_message: None,
                last_run_duration: None,
            },
            body: String::new(),
            path: PathBuf::from(path),
            formal_blocks: vec![],
        }
    }

    fn mk_feature(id: &str, tests: Vec<&str>) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("title for {}", id),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![],
                tests: tests.into_iter().map(String::from).collect(),
                domains: vec![],
                domains_acknowledged: Default::default(),
                patterns: vec![],
                due_date: None,
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/features/{}-x.md", id)),
        }
    }

    #[test]
    fn plan_autofill_skips_fully_configured_tcs() {
        let f = mk_feature("FT-001", vec!["TC-001"]);
        let tcs = vec![mk_tc(
            "TC-001",
            Some("cargo-test"),
            Some("tc_001_x"),
            "docs/tests/TC-001-x.md",
        )];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        assert!(plan_autofill(&graph, "FT-001").is_empty());
    }

    #[test]
    fn plan_autofill_includes_tcs_missing_runner() {
        let f = mk_feature("FT-001", vec!["TC-001", "TC-002"]);
        let tcs = vec![
            mk_tc(
                "TC-001",
                Some("cargo-test"),
                Some("tc_001_x"),
                "docs/tests/TC-001-x.md",
            ),
            mk_tc("TC-002", None, None, "docs/tests/TC-002-missing.md"),
        ];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        let plans = plan_autofill(&graph, "FT-001");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].tc_id, "TC-002");
        assert_eq!(plans[0].derived_args, "tc_002_missing");
    }

    #[test]
    fn plan_autofill_includes_tcs_with_partial_config() {
        let f = mk_feature("FT-001", vec!["TC-003", "TC-004"]);
        let tcs = vec![
            mk_tc(
                "TC-003",
                Some("cargo-test"),
                None,
                "docs/tests/TC-003-args-missing.md",
            ),
            mk_tc(
                "TC-004",
                None,
                Some("tc_004_x"),
                "docs/tests/TC-004-runner-missing.md",
            ),
        ];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        let plans = plan_autofill(&graph, "FT-001");
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].tc_id, "TC-003");
        assert_eq!(plans[1].tc_id, "TC-004");
    }

    #[test]
    fn plan_autofill_empty_for_unknown_feature() {
        let graph = KnowledgeGraph::build(vec![], vec![], vec![]);
        assert!(plan_autofill(&graph, "FT-999").is_empty());
    }

    #[test]
    fn plan_autofill_skips_missing_tc_files() {
        let f = mk_feature("FT-001", vec!["TC-001", "TC-missing"]);
        let tcs = vec![mk_tc("TC-001", None, None, "docs/tests/TC-001-x.md")];
        let graph = KnowledgeGraph::build(vec![f], vec![], tcs);
        let plans = plan_autofill(&graph, "FT-001");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].tc_id, "TC-001");
    }
}
