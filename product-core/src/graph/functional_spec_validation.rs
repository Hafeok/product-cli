//! W030 — feature body completeness check (FT-055, ADR-047).
//!
//! Pure module: given a knowledge graph and a `FeaturesConfig`, returns the
//! set of W030 diagnostics — one per feature with at least one missing
//! required section. Severity is controlled by
//! `[features].completeness-severity`; the diagnostic code string `"W030"`
//! is stable across tiers (only the tier flips).

use crate::config::{CompletenessSeverity, FeaturesConfig};
use crate::error::Diagnostic;
use crate::feature::body_sections::parse_body_sections;
use crate::types::{Feature, FeatureStatus};

/// Findings split by tier.
#[derive(Debug, Default)]
pub struct FunctionalSpecFindings {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

/// Check every feature in the graph for W030 — missing required sections.
///
/// Behaviour follows the FT-055 spec:
/// - Features with `phase < required_from_phase` are exempt.
/// - Abandoned features are exempt.
/// - Top-level missing sections are reported by exact name.
/// - When `## Functional Specification` is present, missing required H3
///   subsections are reported as `Functional Specification > <name>`. If
///   the parent H2 is absent, subsection findings are skipped — the
///   parent is already covered by the top-level finding.
/// - A section is "present" only when it has at least one non-whitespace
///   content line before the next same-or-higher-level heading. Empty
///   meaning ("Stateless. ...") satisfies; whitespace-only does not.
pub fn check_features<'a, I>(
    features: I,
    config: &FeaturesConfig,
) -> FunctionalSpecFindings
where
    I: IntoIterator<Item = &'a Feature>,
{
    let mut out = FunctionalSpecFindings::default();
    for f in features {
        if let Some(diag) = check_feature(f, config) {
            match config.completeness_severity {
                CompletenessSeverity::Warning => out.warnings.push(diag),
                CompletenessSeverity::Error => {
                    // Promote the same diagnostic to error tier; keep code
                    // string "W030" stable.
                    let mut e = diag;
                    e.tier = crate::error::DiagnosticTier::Error;
                    out.errors.push(e);
                }
            }
        }
    }
    out
}

/// Compute a single W030 diagnostic for one feature, or `None` when the
/// feature is exempt or fully complete.
pub fn check_feature(feature: &Feature, config: &FeaturesConfig) -> Option<Diagnostic> {
    if feature.front.status == FeatureStatus::Abandoned {
        return None;
    }
    if feature.front.phase < config.required_from_phase {
        return None;
    }

    let sections = parse_body_sections(&feature.body);

    // Top-level missing — by exact name with content present.
    let mut missing_top: Vec<String> = Vec::new();
    for required in &config.required_sections {
        if !sections.h2_has_content(required) {
            missing_top.push(required.clone());
        }
    }

    // Subsections under `## Functional Specification` (when present).
    let fs_name = "Functional Specification";
    let mut missing_sub: Vec<String> = Vec::new();
    let fs_present = sections.h2.iter().any(|h| h == fs_name)
        && sections.h2_has_content(fs_name);
    if fs_present {
        for required in &config.functional_spec_subsections {
            if !sections.h3_has_content(fs_name, required) {
                missing_sub.push(format!("{} > {}", fs_name, required));
            }
        }
    }

    if missing_top.is_empty() && missing_sub.is_empty() {
        return None;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    detail_lines.push(format!("{} — {}", feature.front.id, feature.front.title));
    detail_lines.push("Missing sections:".to_string());
    for name in &missing_top {
        detail_lines.push(format!("  - {}", name));
    }
    for name in &missing_sub {
        detail_lines.push(format!("  - {}", name));
    }

    let detail = detail_lines.join("\n");
    Some(
        Diagnostic::warning("W030", "feature body missing required section")
            .with_file(feature.path.clone())
            .with_detail(&detail)
            .with_hint("add with `product request change`, op: set, field: body"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Feature, FeatureFrontMatter, FeatureStatus};
    use std::path::PathBuf;

    fn mk(id: &str, phase: u32, status: FeatureStatus, body: &str) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.into(),
                title: "Test".into(),
                phase,
                status,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: Default::default(),
                patterns: vec![],
                due_date: None,
                bundle: None,
                adrs_rejected: vec![],
            },
            body: body.into(),
            path: PathBuf::from(format!("docs/features/{}.md", id)),
        }
    }

    #[test]
    fn empty_body_emits_w030_for_all_top_level() {
        let f = mk("FT-001", 1, FeatureStatus::Planned, "");
        let cfg = FeaturesConfig::default();
        let diag = check_feature(&f, &cfg).expect("expected W030");
        assert_eq!(diag.code, "W030");
        let detail = diag.detail.expect("detail");
        assert!(detail.contains("- Description"));
        assert!(detail.contains("- Functional Specification"));
        assert!(detail.contains("- Out of scope"));
    }

    #[test]
    fn exempt_below_required_from_phase() {
        let mut cfg = FeaturesConfig::default();
        cfg.required_from_phase = 2;
        let f = mk("FT-001", 1, FeatureStatus::Planned, "");
        assert!(check_feature(&f, &cfg).is_none());
    }

    #[test]
    fn abandoned_feature_exempt() {
        let f = mk("FT-001", 1, FeatureStatus::Abandoned, "");
        assert!(check_feature(&f, &FeaturesConfig::default()).is_none());
    }

    #[test]
    fn complete_body_clears_w030() {
        let body = "\
## Description

prose

## Functional Specification

### Inputs

x

### Outputs

x

### State

x

### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x

## Out of scope

- nothing
";
        let f = mk("FT-001", 1, FeatureStatus::Planned, body);
        assert!(check_feature(&f, &FeaturesConfig::default()).is_none());
    }

    #[test]
    fn missing_subsection_reported_with_parent_prefix() {
        let body = "\
## Description

x

## Functional Specification

### Inputs

x

### Outputs

x

## Out of scope

x
";
        let f = mk("FT-001", 1, FeatureStatus::Planned, body);
        let diag = check_feature(&f, &FeaturesConfig::default()).expect("expected W030");
        let detail = diag.detail.expect("detail");
        assert!(detail.contains("Functional Specification > State"));
        assert!(detail.contains("Functional Specification > Behaviour"));
        // Functional Specification itself should not appear as missing top-level.
        assert!(!detail.contains("- Functional Specification\n"));
    }
}
