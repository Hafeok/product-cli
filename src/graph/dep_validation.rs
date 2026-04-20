//! Dependency validation checks — E013, W013 (ADR-030)

use super::model::KnowledgeGraph;
use super::validation_helpers::push_broken_link;
use crate::error::{CheckResult, Diagnostic};
use std::collections::HashSet;

impl KnowledgeGraph {
    /// E013: Dependency has no linked ADR — every dependency requires a governing decision (ADR-030)
    pub(crate) fn check_dep_has_adr(&self, result: &mut CheckResult) {
        for d in self.dependencies.values() {
            if d.front.adrs.is_empty() {
                result.errors.push(
                    Diagnostic::error("E013", "dependency has no governing ADR")
                        .with_file(d.path.clone())
                        .with_detail(&format!(
                            "{} has no ADR links — every dependency requires a governing decision",
                            d.front.id
                        ))
                        .with_hint("add an `adrs: [ADR-XXX]` field linking to the decision that chose this dependency"),
                );
            }
        }
    }

    /// W013: Feature uses a deprecated or migrating dependency (ADR-030)
    pub(crate) fn check_dep_deprecated_usage(&self, result: &mut CheckResult) {
        use crate::types::DependencyStatus;
        for d in self.dependencies.values() {
            if d.front.status != DependencyStatus::Deprecated && d.front.status != DependencyStatus::Migrating {
                continue;
            }
            for feat_id in &d.front.features {
                if let Some(f) = self.features.get(feat_id) {
                    result.warnings.push(
                        Diagnostic::warning("W013", "feature uses a deprecated dependency")
                            .with_file(f.path.clone())
                            .with_detail(&format!(
                                "{} uses {} which has status '{}'",
                                feat_id, d.front.id, d.front.status
                            ))
                            .with_hint("migrate to the successor dependency or update the dependency status"),
                    );
                }
            }
        }
    }

    /// Broken links in dependency front-matter (ADR-030)
    pub(crate) fn check_dep_broken_links(&self, result: &mut CheckResult, all_ids: &HashSet<String>) {
        for d in self.dependencies.values() {
            for feat_id in &d.front.features {
                if !all_ids.contains(feat_id) {
                    push_broken_link(result, &d.path, &d.front.id, feat_id,
                        "references feature", "create the feature or remove the reference");
                }
            }
            for adr_id in &d.front.adrs {
                if !all_ids.contains(adr_id) {
                    push_broken_link(result, &d.path, &d.front.id, adr_id,
                        "references ADR", "create the ADR or remove the reference");
                }
            }
            for sup_id in &d.front.supersedes {
                if !all_ids.contains(sup_id) {
                    push_broken_link(result, &d.path, &d.front.id, sup_id,
                        "supersedes", "create the dependency or remove the reference");
                }
            }
        }
    }
}
