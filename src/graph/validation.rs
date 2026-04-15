//! Graph validation — structural checks, diagnostics.

use super::model::{find_reference_line, KnowledgeGraph};
use crate::error::{CheckResult, Diagnostic, ProductError};
use crate::types::*;
use std::collections::HashSet;

impl KnowledgeGraph {
    pub fn check(&self) -> CheckResult {
        let mut result = CheckResult::new();
        let all_ids = self.all_ids();

        self.check_parse_errors(&mut result);
        self.check_duplicate_ids(&mut result);
        self.check_feature_broken_links(&mut result, &all_ids);
        self.check_adr_broken_links(&mut result, &all_ids);
        self.check_test_broken_links(&mut result, &all_ids);
        self.check_dependency_cycles(&mut result);
        self.check_supersession_cycles(&mut result);
        self.check_orphaned_adrs(&mut result);
        self.check_orphaned_tests(&mut result);
        self.check_features_no_tests(&mut result);
        self.check_features_no_exit_criteria(&mut result);
        self.check_complete_features_blocking_tcs(&mut result);
        self.check_formal_block_coverage(&mut result);
        self.check_phase_dependency_order(&mut result);
        self.check_evidence_delta(&mut result);
        self.check_formal_block_diagnostics(&mut result);
        self.check_content_hashes(&mut result);
        self.check_proposed_adr_lifecycle(&mut result);
        self.check_dep_has_adr(&mut result);
        self.check_dep_deprecated_usage(&mut result);
        self.check_dep_broken_links(&mut result, &all_ids);

        result
    }

    fn check_parse_errors(&self, result: &mut CheckResult) {
        for pe in &self.parse_errors {
            match pe {
                ProductError::ParseError { file, line, message } => {
                    let mut diag = Diagnostic::error("E001", "malformed front-matter")
                        .with_file(file.clone())
                        .with_detail(message);
                    if let Some(l) = line {
                        diag = diag.with_line(*l);
                    }
                    result.errors.push(diag);
                }
                ProductError::InvalidId { file, id } => {
                    result.errors.push(
                        Diagnostic::error("E005", "invalid artifact ID")
                            .with_file(file.clone())
                            .with_detail(&format!("'{}' does not match PREFIX-NNN format", id)),
                    );
                }
                ProductError::MissingField { file, field } => {
                    result.errors.push(
                        Diagnostic::error("E006", "missing required field")
                            .with_file(file.clone())
                            .with_detail(&format!("required field '{}' not found", field)),
                    );
                }
                other => {
                    result.errors.push(
                        Diagnostic::error("E001", "parse error")
                            .with_detail(&format!("{}", other)),
                    );
                }
            }
        }
    }

    fn check_duplicate_ids(&self, result: &mut CheckResult) {
        for (id, paths) in &self.duplicates {
            let path_strs: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            result.errors.push(
                Diagnostic::error("E011", "duplicate artifact ID")
                    .with_detail(&format!("{} is declared in multiple files: {}", id, path_strs.join(", ")))
                    .with_hint("each artifact ID must be unique — rename or remove the duplicate"),
            );
        }
    }

    fn check_feature_broken_links(&self, result: &mut CheckResult, all_ids: &HashSet<String>) {
        for f in self.features.values() {
            for adr_id in &f.front.adrs {
                if !all_ids.contains(adr_id) {
                    push_broken_link(result, &f.path, &f.front.id, adr_id,
                        "references", "create the file with `product adr new` or remove the reference");
                }
            }
            for test_id in &f.front.tests {
                if !all_ids.contains(test_id) {
                    push_broken_link(result, &f.path, &f.front.id, test_id,
                        "references", "create the file with `product test new` or remove the reference");
                }
            }
            for dep_id in &f.front.depends_on {
                if !self.features.contains_key(dep_id) {
                    push_broken_link(result, &f.path, &f.front.id, dep_id,
                        "depends-on", "create the feature or remove the dependency");
                }
            }
        }
    }

    fn check_adr_broken_links(&self, result: &mut CheckResult, all_ids: &HashSet<String>) {
        for a in self.adrs.values() {
            for sup_id in &a.front.supersedes {
                if !all_ids.contains(sup_id) {
                    push_broken_link(result, &a.path, &a.front.id, sup_id,
                        "supersedes", "");
                }
            }
        }
    }

    fn check_test_broken_links(&self, result: &mut CheckResult, all_ids: &HashSet<String>) {
        for t in self.tests.values() {
            for f_id in &t.front.validates.features {
                if !all_ids.contains(f_id) {
                    push_broken_link(result, &t.path, &t.front.id, f_id,
                        "validates feature", "");
                }
            }
            for a_id in &t.front.validates.adrs {
                if !all_ids.contains(a_id) {
                    push_broken_link(result, &t.path, &t.front.id, a_id,
                        "validates ADR", "");
                }
            }
        }
    }

    fn check_dependency_cycles(&self, result: &mut CheckResult) {
        if let Err(ProductError::DependencyCycle { cycle }) = self.topological_sort() {
            result.errors.push(
                Diagnostic::error("E003", "dependency cycle in depends-on DAG")
                    .with_detail(&format!("cycle: {}", cycle.join(" -> "))),
            );
        }
    }

    fn check_supersession_cycles(&self, result: &mut CheckResult) {
        if let Some(cycle) = self.detect_supersession_cycle() {
            result.errors.push(
                Diagnostic::error("E004", "supersession cycle in ADR supersedes chain")
                    .with_detail(&format!("cycle: {}", cycle.join(" -> "))),
            );
        }
    }

    fn check_orphaned_adrs(&self, result: &mut CheckResult) {
        for a in self.adrs.values() {
            let has_incoming = self.features.values().any(|f| f.front.adrs.contains(&a.front.id));
            if !has_incoming {
                result.warnings.push(
                    Diagnostic::warning("W001", "orphaned artifact")
                        .with_file(a.path.clone())
                        .with_detail(&format!("{} has no feature linking to it", a.front.id))
                        .with_hint("link it to a feature with `product feature link`"),
                );
            }
        }
    }

    fn check_orphaned_tests(&self, result: &mut CheckResult) {
        for t in self.tests.values() {
            // ADR-010: Exclude abandoned features from incoming check
            let has_incoming = self.features.values().any(|f| {
                f.front.status != FeatureStatus::Abandoned && f.front.tests.contains(&t.front.id)
            });
            if !has_incoming && t.front.validates.features.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W001", "orphaned artifact")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{} has no feature linking to it", t.front.id)),
                );
            }
        }
    }

    /// W002: Features with no linked tests
    fn check_features_no_tests(&self, result: &mut CheckResult) {
        for f in self.features.values() {
            if f.front.status != FeatureStatus::Abandoned && f.front.tests.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W002", "feature has no linked test criteria")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} — {}", f.front.id, f.front.title))
                        .with_hint("add test criteria with `product test new`"),
                );
            }
        }
    }

    /// W003: Features with no exit-criteria test
    fn check_features_no_exit_criteria(&self, result: &mut CheckResult) {
        for f in self.features.values() {
            if f.front.status == FeatureStatus::Abandoned {
                continue;
            }
            let has_exit = f.front.tests.iter().any(|t_id| {
                self.tests
                    .get(t_id)
                    .map(|t| t.front.test_type == TestType::ExitCriteria)
                    .unwrap_or(false)
            });
            if !has_exit && !f.front.tests.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W003", "missing exit criteria")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} has no test of type `exit-criteria`", f.front.id))
                        .with_hint("add one with `product test new --type exit-criteria`"),
                );
            }
        }
    }

    /// W016: Feature marked complete but has unimplemented or failing TCs
    fn check_complete_features_blocking_tcs(&self, result: &mut CheckResult) {
        for f in self.features.values() {
            if f.front.status != FeatureStatus::Complete {
                continue;
            }
            let blocking_tcs: Vec<&str> = f
                .front
                .tests
                .iter()
                .filter_map(|t_id| {
                    self.tests.get(t_id.as_str()).and_then(|t| {
                        if t.front.status == TestStatus::Unimplemented
                            || t.front.status == TestStatus::Failing
                        {
                            Some(t.front.id.as_str())
                        } else {
                            None
                        }
                    })
                })
                .collect();
            if !blocking_tcs.is_empty() {
                push_blocking_tc_warning(result, f, &blocking_tcs);
            }
        }
    }

    /// W004: Invariant/chaos tests missing formal blocks
    fn check_formal_block_coverage(&self, result: &mut CheckResult) {
        for t in self.tests.values() {
            if (t.front.test_type == TestType::Invariant || t.front.test_type == TestType::Chaos)
                && t.formal_blocks.is_empty()
            {
                result.warnings.push(
                    Diagnostic::warning("W004", "missing formal specification blocks")
                        .with_file(t.path.clone())
                        .with_detail(&format!(
                            "{} is type {} but has no formal blocks",
                            t.front.id, t.front.test_type
                        )),
                );
            }
        }
    }

    /// W005: Phase label disagrees with dependency order
    fn check_phase_dependency_order(&self, result: &mut CheckResult) {
        for f in self.features.values() {
            for dep_id in &f.front.depends_on {
                if let Some(dep) = self.features.get(dep_id) {
                    if dep.front.phase > f.front.phase {
                        result.warnings.push(
                            Diagnostic::warning("W005", "phase label disagrees with dependency order")
                                .with_file(f.path.clone())
                                .with_detail(&format!(
                                    "{} (phase {}) depends-on {} (phase {})",
                                    f.front.id, f.front.phase, dep_id, dep.front.phase
                                )),
                        );
                    }
                }
            }
        }
    }

    /// W006: Evidence block delta below 0.7
    fn check_evidence_delta(&self, result: &mut CheckResult) {
        for t in self.tests.values() {
            for block in &t.formal_blocks {
                if let crate::formal::FormalBlock::Evidence(e) = block {
                    if e.delta < 0.7 {
                        result.warnings.push(
                            Diagnostic::warning("W006", "low-confidence specification")
                                .with_file(t.path.clone())
                                .with_detail(&format!(
                                    "{} evidence block delta={:.2} (below 0.7 threshold)",
                                    t.front.id, e.delta
                                )),
                        );
                    }
                }
            }
        }
    }

    /// Formal block diagnostics: E001 errors and W004 warnings from formal block parsing
    fn check_formal_block_diagnostics(&self, result: &mut CheckResult) {
        for t in self.tests.values() {
            let diag = crate::formal::parse_formal_blocks_with_diagnostics(&t.body);
            for err in &diag.errors {
                result.errors.push(
                    Diagnostic::error("E001", "formal block parse error")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{}: {}", t.front.id, err)),
                );
            }
            for warn in &diag.warnings {
                result.warnings.push(
                    Diagnostic::warning("W004", "formal block warning")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{}: {}", t.front.id, warn)),
                );
            }
        }
    }

    /// E014/W016/E015: Content hash checks (ADR-032)
    fn check_content_hashes(&self, result: &mut CheckResult) {
        let adrs_vec: Vec<&crate::types::Adr> = self.adrs.values().collect();
        let tests_vec: Vec<&crate::types::TestCriterion> = self.tests.values().collect();
        let hash_result = crate::hash::verify_all(&adrs_vec, &tests_vec);
        result.errors.extend(hash_result.errors);
        result.warnings.extend(hash_result.warnings);
    }

    pub fn detect_supersession_cycle(&self) -> Option<Vec<String>> {
        for adr in self.adrs.values() {
            let mut visited = std::collections::HashSet::new();
            let mut current = adr.front.id.clone();
            visited.insert(current.clone());
            while let Some(a) = self.adrs.get(&current) {
                if let Some(next) = a.front.supersedes.first() {
                    if visited.contains(next) {
                        return Some(visited.into_iter().collect());
                    }
                    visited.insert(next.clone());
                    current = next.clone();
                } else {
                    break;
                }
            }
        }
        None
    }
}

/// Helper: push a broken-link E002 diagnostic with optional line info
pub(crate) fn push_broken_link(
    result: &mut CheckResult,
    path: &std::path::Path,
    from_id: &str,
    target_id: &str,
    verb: &str,
    hint: &str,
) {
    let mut diag = Diagnostic::error("E002", "broken link")
        .with_file(path.to_path_buf())
        .with_detail(&format!("{} {} {} which does not exist", from_id, verb, target_id));
    if !hint.is_empty() {
        diag = diag.with_hint(hint);
    }
    if let Some((line, content)) = find_reference_line(path, target_id) {
        diag = diag.with_line(line).with_context(&content);
    }
    result.errors.push(diag);
}

/// Helper: push W016 warning for complete features with blocking TCs
fn push_blocking_tc_warning(result: &mut CheckResult, f: &Feature, blocking_tcs: &[&str]) {
    let preview: Vec<&str> = blocking_tcs.iter().take(5).copied().collect();
    let suffix = if blocking_tcs.len() > 5 {
        format!(", ... ({} total)", blocking_tcs.len())
    } else {
        String::new()
    };
    result.warnings.push(
        Diagnostic::warning("W016", "complete feature has unimplemented tests")
            .with_file(f.path.clone())
            .with_detail(&format!(
                "{} is complete but has {} unimplemented/failing TC(s): {}{}",
                f.front.id,
                blocking_tcs.len(),
                preview.join(", "),
                suffix,
            ))
            .with_hint("run `product verify` to re-evaluate, or set blocking TCs to `unrunnable`"),
    );
}
