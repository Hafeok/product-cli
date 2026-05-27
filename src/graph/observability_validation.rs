//! TC observability validation for `product graph check` (FT-072, ADR-051).
//!
//! Two diagnostics:
//!
//! - **E032** — TC of a required type at phase ≥ threshold has missing or
//!   empty `observes:`. References ADR-051 in the hint.
//! - **W034** — TC declares `observes:` but its body never references any
//!   declared surface. Severity is promotable to error via
//!   `[tc-observability].body-reference-severity = "error"`; the diagnostic
//!   code string `"W034"` is stable across tiers.

use crate::config::{BodyReferenceSeverity, TcObservabilityConfig};
use crate::error::{CheckResult, Diagnostic, DiagnosticTier};
use crate::tc::observability::{
    body_references_surface, is_known_surface, requires_observes, surface_hint,
};
use crate::types::TestCriterion;

/// Run both observability gates against `tests` and append findings to
/// `result`.
pub fn check<'a, I>(tests: I, config: &TcObservabilityConfig, result: &mut CheckResult)
where
    I: IntoIterator<Item = &'a TestCriterion>,
{
    for t in tests {
        check_required(t, config, result);
        check_body_reference(t, config, result);
        check_unknown_surfaces(t, config, result);
    }
}

/// E032 — required-for gate.
fn check_required(t: &TestCriterion, config: &TcObservabilityConfig, result: &mut CheckResult) {
    let tc_type_str = t.front.test_type.as_str();
    if !requires_observes(tc_type_str, t.front.phase, config) {
        return;
    }
    if !t.front.observes.is_empty() {
        return;
    }
    let detail = format!(
        "{} (type: {}, phase: {}) declares no observable surface, but its type is in [tc-observability].required-for-types",
        t.front.id, tc_type_str, t.front.phase
    );
    let hint = format!(
        "add a non-empty `observes:` list to the front-matter (allowed: {})\n   = see ADR-051",
        surface_hint(config)
    );
    result.errors.push(
        Diagnostic::error("E032", "TC missing required observes field")
            .with_file(t.path.clone())
            .with_detail(&detail)
            .with_hint(&hint),
    );
}

/// W034 — body-reference gate.
fn check_body_reference(
    t: &TestCriterion,
    config: &TcObservabilityConfig,
    result: &mut CheckResult,
) {
    if t.front.observes.is_empty() {
        return;
    }
    let any_match = t
        .front
        .observes
        .iter()
        .any(|s| body_references_surface(&t.body, s));
    if any_match {
        return;
    }
    let detail = format!(
        "{} declares observes: {:?} but its body does not reference any of these surfaces",
        t.front.id, t.front.observes
    );
    let hint = "mention the observed surface in the TC body, or remove it from `observes:`";
    let mut diag = Diagnostic::warning("W034", "TC body lacks reference to declared surface")
        .with_file(t.path.clone())
        .with_detail(&detail)
        .with_hint(hint);
    if config.body_reference_severity == BodyReferenceSeverity::Error {
        diag.tier = DiagnosticTier::Error;
        result.errors.push(diag);
    } else {
        result.warnings.push(diag);
    }
}

/// E026 — any value in `observes:` that is not in the allowed vocabulary.
fn check_unknown_surfaces(
    t: &TestCriterion,
    config: &TcObservabilityConfig,
    result: &mut CheckResult,
) {
    for surface in &t.front.observes {
        if !is_known_surface(surface, config) {
            let detail = format!(
                "{} declares observes: [{}] — unknown surface",
                t.front.id, surface
            );
            let hint = format!(
                "allowed surfaces: {}\n   = add the value to [tc-observability].custom to accept it",
                surface_hint(config)
            );
            result.errors.push(
                Diagnostic::error("E026", "unknown observes surface value")
                    .with_file(t.path.clone())
                    .with_detail(&detail)
                    .with_hint(&hint),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TcObservabilityConfig;
    use crate::types::{TestCriterion, TestFrontMatter, TestStatus, TestType, ValidatesBlock};
    use std::path::PathBuf;

    fn mk(id: &str, phase: u32, ttype: TestType, observes: Vec<String>, body: &str) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: id.into(),
                title: "T".into(),
                test_type: ttype,
                status: TestStatus::Unimplemented,
                validates: ValidatesBlock::default(),
                phase,
                content_hash: None,
                runner: None,
                runner_args: None,
                runner_timeout: None,
                requires: vec![],
                observes,
                last_run: None,
                failure_message: None,
                last_run_duration: None,
            },
            body: body.into(),
            path: PathBuf::from(format!("docs/tests/{}.md", id)),
            formal_blocks: vec![],
        }
    }

    #[test]
    fn e032_fires_for_scenario_at_phase_5_without_observes() {
        let t = mk("TC-001", 5, TestType::Scenario, vec![], "");
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(result.errors.iter().any(|d| d.code == "E032"));
    }

    #[test]
    fn e032_silent_for_invariant_type() {
        let t = mk("TC-001", 5, TestType::Invariant, vec![], "");
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(!result.errors.iter().any(|d| d.code == "E032"));
    }

    #[test]
    fn e032_silent_below_phase_threshold() {
        let t = mk("TC-001", 4, TestType::Scenario, vec![], "");
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(!result.errors.iter().any(|d| d.code == "E032"));
    }

    #[test]
    fn w034_fires_when_body_lacks_reference() {
        let t = mk("TC-001", 5, TestType::Scenario, vec!["file".into()], "unrelated text");
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(result.warnings.iter().any(|d| d.code == "W034"));
    }

    #[test]
    fn w034_silent_when_body_mentions_surface() {
        let t = mk(
            "TC-001",
            5,
            TestType::Scenario,
            vec!["file".into()],
            "The file is written to disk",
        );
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(!result.warnings.iter().any(|d| d.code == "W034"));
    }

    #[test]
    fn w034_promoted_to_error_via_config() {
        let t = mk("TC-001", 5, TestType::Scenario, vec!["file".into()], "unrelated");
        let mut cfg = TcObservabilityConfig::default();
        cfg.body_reference_severity = BodyReferenceSeverity::Error;
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(result.errors.iter().any(|d| d.code == "W034"));
        assert!(!result.warnings.iter().any(|d| d.code == "W034"));
    }

    #[test]
    fn e026_fires_for_unknown_surface() {
        let t = mk(
            "TC-001",
            5,
            TestType::Scenario,
            vec!["bogus_surface".into()],
            "uses bogus_surface in body",
        );
        let cfg = TcObservabilityConfig::default();
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(result.errors.iter().any(|d| d.code == "E026"));
    }

    #[test]
    fn custom_surface_accepted() {
        let mut cfg = TcObservabilityConfig::default();
        cfg.custom.push("my-surface".into());
        let t = mk(
            "TC-001",
            5,
            TestType::Scenario,
            vec!["my-surface".into()],
            "exercises my-surface in body",
        );
        let mut result = CheckResult::new();
        check([&t], &cfg, &mut result);
        assert!(!result.errors.iter().any(|d| d.code == "E026"));
    }
}
