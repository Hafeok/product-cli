//! Pattern-related graph diagnostics (FT-071, ADR-050).
//!
//! Three new findings live here:
//!
//! - **E031** — `requires:` cycle in the pattern DAG.
//! - **W032** — a live (planned / in-progress) feature cites a deprecated
//!   pattern.
//! - **W033** — a live pattern body is missing a required H2 section
//!   configured in `[patterns].body-sections`. Promotable to E033 when
//!   `[patterns].body-severity = "error"`.

use crate::config::{
    FeaturesConfig, PatternBodySeverity, PatternsConfig, PatternsRequiredSeverity,
};
use crate::error::{CheckResult, Diagnostic, DiagnosticTier};
use crate::graph::pattern_topo::detect_any_cycle;
use crate::graph::KnowledgeGraph;
use crate::types::{FeatureStatus, PatternStatus};

/// Run every pattern-related check and append findings to `result`.
pub fn check_all(graph: &KnowledgeGraph, config: &PatternsConfig, result: &mut CheckResult) {
    check_requires_cycle(graph, result);
    check_deprecated_pattern_cited(graph, result);
    check_pattern_body_sections(graph, config, result);
}

/// FT-073 / ADR-050 — W035 advisory raised when an `in-progress` feature has
/// no entries in `patterns:`. Off by default; enabled per
/// `[features].patterns-required-severity`.
pub fn check_patterns_required(
    graph: &KnowledgeGraph,
    config: &FeaturesConfig,
    result: &mut CheckResult,
) {
    if matches!(
        config.patterns_required_severity,
        PatternsRequiredSeverity::Off
    ) {
        return;
    }
    for feature in graph.features.values() {
        if feature.front.status != FeatureStatus::InProgress {
            continue;
        }
        if !feature.front.patterns.is_empty() {
            continue;
        }
        let mut diag = Diagnostic::warning(
            "W035",
            "in-progress feature cites no patterns",
        )
        .with_file(feature.path.clone())
        .with_detail(&format!(
            "{} — {}",
            feature.front.id, feature.front.title
        ))
        .with_hint("review `product pattern list` and cite applicable patterns via `product feature link FT-XXX --pattern PAT-YYY`, or set `[features].patterns-required-severity = \"off\"`");
        match config.patterns_required_severity {
            PatternsRequiredSeverity::Off => unreachable!(),
            PatternsRequiredSeverity::Warning => result.warnings.push(diag),
            PatternsRequiredSeverity::Error => {
                diag.tier = DiagnosticTier::Error;
                result.errors.push(diag);
            }
        }
    }
}

/// E031 — `requires:` cycle in the pattern DAG.
fn check_requires_cycle(graph: &KnowledgeGraph, result: &mut CheckResult) {
    if let Some(cycle) = detect_any_cycle(graph) {
        let path = cycle.join(" → ");
        result.errors.push(
            Diagnostic::error("E031", "pattern requires cycle")
                .with_detail(&format!("cycle: {}", path))
                .with_hint("break the cycle by removing one of the `requires:` entries"),
        );
    }
}

/// W032 — a live (planned / in-progress) feature cites a deprecated pattern.
fn check_deprecated_pattern_cited(graph: &KnowledgeGraph, result: &mut CheckResult) {
    for feature in graph.features.values() {
        // Complete / abandoned features are exempt — complete features
        // already shipped against the pattern; abandoned ones are out.
        if !matches!(
            feature.front.status,
            FeatureStatus::Planned | FeatureStatus::InProgress
        ) {
            continue;
        }
        for pat_id in &feature.front.patterns {
            if let Some(p) = graph.patterns.get(pat_id) {
                if p.front.status == PatternStatus::Deprecated {
                    let replacement = p
                        .front
                        .deprecated_by
                        .as_deref()
                        .map(|r| format!(" (replaced by {})", r))
                        .unwrap_or_default();
                    result.warnings.push(
                        Diagnostic::warning(
                            "W032",
                            "deprecated pattern cited by live feature",
                        )
                        .with_file(feature.path.clone())
                        .with_detail(&format!(
                            "{} (status {}) cites deprecated pattern {}{}",
                            feature.front.id, feature.front.status, pat_id, replacement
                        ))
                        .with_hint(
                            "migrate the feature to the replacement pattern or remove the citation",
                        ),
                    );
                }
            }
        }
    }
}

/// W033 / E033 — pattern body missing a required H2 section. Severity is
/// controlled by `[patterns].body-severity`.
fn check_pattern_body_sections(
    graph: &KnowledgeGraph,
    config: &PatternsConfig,
    result: &mut CheckResult,
) {
    for pat in graph.patterns.values() {
        // Only live patterns are required to carry the body convention —
        // deprecated patterns are accreted history.
        if pat.front.status != PatternStatus::Live {
            continue;
        }
        let mut missing: Vec<String> = Vec::new();
        for required in &config.body_sections {
            if !body_contains_h2(&pat.body, required) {
                missing.push(required.clone());
            }
        }
        if missing.is_empty() {
            continue;
        }
        let detail_lines: Vec<String> = std::iter::once(format!(
            "{} — {}",
            pat.front.id, pat.front.title
        ))
        .chain(std::iter::once("Missing sections:".to_string()))
        .chain(missing.iter().map(|n| format!("  - {}", n)))
        .collect();
        let mut diag = Diagnostic::warning("W033", "pattern body missing required section")
            .with_file(pat.path.clone())
            .with_detail(&detail_lines.join("\n"))
            .with_hint("add the missing H2 sections or override `[patterns].body-sections`");
        match config.body_severity {
            PatternBodySeverity::Warning => result.warnings.push(diag),
            PatternBodySeverity::Error => {
                diag.tier = DiagnosticTier::Error;
                result.errors.push(diag);
            }
        }
    }
}

/// True when `body` contains an ATX H2 with exactly `name` (trimmed). Skips
/// fenced code blocks so heading-shaped lines inside samples do not count.
fn body_contains_h2(body: &str, name: &str) -> bool {
    let needle = name.trim();
    let mut in_fence = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if rest.trim() == needle {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Feature, FeatureFrontMatter, Pattern, PatternFrontMatter};
    use std::path::PathBuf;

    fn mk_pat(id: &str, status: PatternStatus, requires: Vec<&str>, body: &str) -> Pattern {
        Pattern {
            front: PatternFrontMatter {
                id: id.into(),
                title: id.into(),
                status,
                domains: vec![],
                adrs: vec![],
                requires: requires.into_iter().map(String::from).collect(),
                examples: vec![],
                deprecated_by: None,
            },
            body: body.into(),
            path: PathBuf::from(format!("docs/patterns/{}.md", id)),
        }
    }

    fn mk_feat(id: &str, status: FeatureStatus, patterns: Vec<&str>) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.into(),
                title: id.into(),
                phase: 1,
                status,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: Default::default(),
                patterns: patterns.into_iter().map(String::from).collect(),
                due_date: None,
                bundle: None,
                adrs_rejected: vec![],
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/features/{}.md", id)),
        }
    }

    fn complete_body() -> String {
        "## When to use\n\nx\n\n## Prerequisites\n\nx\n\n## The pattern\n\nx\n\n## Anti-patterns\n\nx\n\n## Worked example\n\nx\n".into()
    }

    #[test]
    fn e031_fires_on_pattern_requires_cycle() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![
                mk_pat("PAT-001", PatternStatus::Live, vec!["PAT-002"], &complete_body()),
                mk_pat("PAT-002", PatternStatus::Live, vec!["PAT-001"], &complete_body()),
            ],
        );
        let mut result = CheckResult::new();
        check_all(&g, &PatternsConfig::default(), &mut result);
        assert!(result.errors.iter().any(|d| d.code == "E031"));
    }

    #[test]
    fn w032_fires_when_live_feature_cites_deprecated() {
        let g = KnowledgeGraph::build_full(
            vec![mk_feat("FT-100", FeatureStatus::Planned, vec!["PAT-001"])],
            vec![],
            vec![],
            vec![],
            vec![mk_pat(
                "PAT-001",
                PatternStatus::Deprecated,
                vec![],
                &complete_body(),
            )],
        );
        let mut result = CheckResult::new();
        check_all(&g, &PatternsConfig::default(), &mut result);
        assert!(result.warnings.iter().any(|d| d.code == "W032"));
    }

    #[test]
    fn w032_suppressed_for_complete_feature() {
        let g = KnowledgeGraph::build_full(
            vec![mk_feat("FT-100", FeatureStatus::Complete, vec!["PAT-001"])],
            vec![],
            vec![],
            vec![],
            vec![mk_pat(
                "PAT-001",
                PatternStatus::Deprecated,
                vec![],
                &complete_body(),
            )],
        );
        let mut result = CheckResult::new();
        check_all(&g, &PatternsConfig::default(), &mut result);
        assert!(!result.warnings.iter().any(|d| d.code == "W032"));
    }

    #[test]
    fn w033_fires_on_missing_h2_section() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mk_pat(
                "PAT-001",
                PatternStatus::Live,
                vec![],
                "## When to use\n\nx\n",
            )],
        );
        let mut result = CheckResult::new();
        check_all(&g, &PatternsConfig::default(), &mut result);
        let w033 = result
            .warnings
            .iter()
            .find(|d| d.code == "W033")
            .expect("expected W033");
        let detail = w033.detail.as_deref().unwrap_or("");
        assert!(detail.contains("Anti-patterns"));
    }

    #[test]
    fn w033_clear_when_all_sections_present() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mk_pat(
                "PAT-001",
                PatternStatus::Live,
                vec![],
                &complete_body(),
            )],
        );
        let mut result = CheckResult::new();
        check_all(&g, &PatternsConfig::default(), &mut result);
        assert!(!result.warnings.iter().any(|d| d.code == "W033"));
    }

    #[test]
    fn w033_promotes_to_error_under_error_severity() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mk_pat(
                "PAT-001",
                PatternStatus::Live,
                vec![],
                "## When to use\n\nx\n",
            )],
        );
        let mut cfg = PatternsConfig::default();
        cfg.body_severity = PatternBodySeverity::Error;
        let mut result = CheckResult::new();
        check_all(&g, &cfg, &mut result);
        // Same code "W033" but error tier
        assert!(result.errors.iter().any(|d| d.code == "W033"));
        assert!(!result.warnings.iter().any(|d| d.code == "W033"));
    }
}
