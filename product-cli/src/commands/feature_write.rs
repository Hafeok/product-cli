//! Feature write operations — creation, linking, status changes.
//!
//! Migrated handlers (`feature_new`, `feature_status`, `feature_domain`) are
//! thin adapters over pure planning functions in `product_core::feature`.
//! Legacy handlers (`feature_link`, `feature_acknowledge`) still use BoxResult
//! and print directly; migrate them when touching.

use product_core::{error::ProductError, feature as feat, fileops, graph, parser, types};
use std::io::{self, BufRead, IsTerminal, Write};

use super::{acquire_write_lock, acquire_write_lock_typed, load_graph, load_graph_typed, BoxResult, CmdResult, Output};

pub(crate) fn feature_new(title: &str, phase: u32) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, root, graph) = load_graph_typed()?;
    let existing: Vec<String> = graph.features.keys().cloned().collect();
    let plan = feat::plan_create(title, phase, &existing, &config.prefixes.feature)?;
    let target_dir = config.resolve_path(&root, &config.paths.features);
    let path = feat::apply_create(&plan, &target_dir)?;
    Ok(Output::text(format!(
        "Created: {} at {}",
        plan.id,
        path.display()
    )))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn feature_link(
    id: &str,
    adr: Option<String>,
    test: Option<String>,
    dep: Option<String>,
    pattern: Option<String>,
    assume_yes: bool,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_config, _root, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    let mut front = f.front.clone();
    let mut changed = false;

    let adr_linked = link_adr(&mut front, id, adr.clone());
    changed |= adr_linked;
    changed |= link_test(&mut front, id, test);
    if let Some(dep_id) = dep {
        changed |= link_dep(&mut front, id, &dep_id, f, &graph)?;
    }
    if let Some(ref pat_id) = pattern {
        changed |= link_pattern(&mut front, id, pat_id, &graph)?;
    }

    // Interactive TC inference when an ADR link was added (ADR-027).
    // FT-067: skip platform-wide ADRs (cross-cutting OR platform). For
    // cross-cutting we'd be linking TCs to every feature touching the ADR
    // (noise); for platform the TC is enforced once at the substrate, so
    // there's no per-feature link to infer either. The narrow predicate
    // (CrossCutting-only) became `is_platform_wide()` to cover both.
    if adr_linked {
        if let Some(ref adr_id) = adr {
            let is_platform_wide = graph
                .adrs
                .get(adr_id.as_str())
                .map(|a| a.front.scope.is_platform_wide())
                .unwrap_or(false);

            if !is_platform_wide {
                let inferred = compute_inferred_tc_links(&graph, id, adr_id);
                if !inferred.is_empty() {
                    println!();
                    println!("  Transitive TC links inferred:");
                    for (tc_id, tc_title) in &inferred {
                        println!(
                            "    {} {:<30} \u{2192} {}  (via {})",
                            tc_id, tc_title, id, adr_id
                        );
                    }
                    println!();

                    if assume_yes || prompt_confirm("  Add these TC links automatically? [Y/n] ") {
                        // Add TC IDs to the feature's tests list
                        for (tc_id, _) in &inferred {
                            if !front.tests.contains(tc_id) {
                                front.tests.push(tc_id.clone());
                            }
                        }
                        front.tests.sort();

                        // Prepare batch writes: feature file + TC files
                        let feature_content = parser::render_feature(&front, &f.body);
                        let mut writes: Vec<(&std::path::Path, String)> = Vec::new();
                        writes.push((&f.path, feature_content));

                        for (tc_id, _) in &inferred {
                            if let Some(tc) = graph.tests.get(tc_id.as_str()) {
                                let mut tc_front = tc.front.clone();
                                if !tc_front.validates.features.contains(&id.to_string()) {
                                    tc_front.validates.features.push(id.to_string());
                                }
                                tc_front.validates.features.sort();
                                let tc_content = parser::render_test(&tc_front, &tc.body);
                                writes.push((&tc.path, tc_content));
                            }
                        }

                        // Write atomically
                        let write_refs: Vec<(&std::path::Path, &str)> = writes
                            .iter()
                            .map(|(p, c)| (*p, c.as_str()))
                            .collect();
                        fileops::write_batch_atomic(&write_refs)?;
                        println!("  Applied {} TC links.", inferred.len());
                        return Ok(());
                    }
                    // User declined — fall through to write only the feature file
                }
            }
        }
    }

    if changed {
        // Pattern linking needs bidirectional reciprocation
        // (FT-073, ADR-050) — when --pattern PAT-YYY is set, write both
        // FT-XXX.patterns += PAT-YYY and PAT-YYY.examples += FT-XXX in one
        // atomic batch. Other field updates write the feature only.
        let feature_content = parser::render_feature(&front, &f.body);
        if let Some(ref pat_id) = pattern {
            if let Some(pat) = graph.patterns.get(pat_id.as_str()) {
                if !pat.front.examples.contains(&id.to_string()) {
                    let mut pat_front = pat.front.clone();
                    pat_front.examples.push(id.to_string());
                    let pat_content = parser::render_pattern(&pat_front, &pat.body);
                    let batch: Vec<(&std::path::Path, &str)> = vec![
                        (f.path.as_path(), feature_content.as_str()),
                        (pat.path.as_path(), pat_content.as_str()),
                    ];
                    fileops::write_batch_atomic(&batch)?;
                    return Ok(());
                }
            }
        }
        fileops::write_file_atomic(&f.path, &feature_content)?;
    }
    Ok(())
}

/// FT-073: link a feature to a pattern (`feature.patterns` ← PAT-YYY).
/// Validates the target pattern exists. The reciprocal write to
/// `pattern.examples` happens in the same atomic batch in the caller.
/// Emits a deprecation warning to stderr when the pattern is deprecated
/// (write still proceeds — author may intentionally cite during migration).
fn link_pattern(
    front: &mut types::FeatureFrontMatter,
    id: &str,
    pat_id: &str,
    graph: &graph::KnowledgeGraph,
) -> Result<bool, Box<dyn std::error::Error>> {
    let pat = graph
        .patterns
        .get(pat_id)
        .ok_or_else(|| ProductError::NotFound(format!("pattern {}", pat_id)))?;
    if pat.front.status == types::PatternStatus::Deprecated {
        let replacement = pat
            .front
            .deprecated_by
            .as_deref()
            .map(|r| format!(" (replaced by {})", r))
            .unwrap_or_default();
        eprintln!(
            "warning[W032]: {} cites deprecated pattern {}{}",
            id, pat_id, replacement
        );
    }
    if !front.patterns.contains(&pat_id.to_string()) {
        front.patterns.push(pat_id.to_string());
        println!("Linked {} -> {}", id, pat_id);
        return Ok(true);
    }
    println!("{} already cites {}", id, pat_id);
    Ok(false)
}

/// Compute TC links that would be inferred from linking a specific ADR to a feature
fn compute_inferred_tc_links(
    graph: &graph::KnowledgeGraph,
    feature_id: &str,
    adr_id: &str,
) -> Vec<(String, String)> {
    let mut inferred = Vec::new();
    for tc in graph.tests.values() {
        if tc.front.validates.adrs.contains(&adr_id.to_string())
            && !tc.front.validates.features.contains(&feature_id.to_string())
        {
            inferred.push((tc.front.id.clone(), tc.front.title.clone()));
        }
    }
    inferred.sort_by(|a, b| a.0.cmp(&b.0));
    inferred
}

/// Prompt user for y/n confirmation.
///
/// In a TTY, empty input (just Enter) defaults to yes.
/// In a non-TTY (piped/scripted), empty or EOF input defaults to no — callers
/// that want silent acceptance must either pipe "y" or pass `--yes` explicitly.
fn prompt_confirm(prompt: &str) -> bool {
    let stdin = io::stdin();
    let is_tty = stdin.is_terminal();

    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => false,
        Ok(_) => {
            let trimmed = line.trim().to_lowercase();
            if trimmed.is_empty() {
                is_tty
            } else {
                trimmed == "y" || trimmed == "yes"
            }
        }
        Err(_) => false,
    }
}

fn link_adr(front: &mut types::FeatureFrontMatter, id: &str, adr: Option<String>) -> bool {
    if let Some(adr_id) = adr {
        if !front.adrs.contains(&adr_id) {
            front.adrs.push(adr_id.clone());
            println!("Linked {} -> {}", id, adr_id);
            return true;
        }
        println!("{} already linked to {}", id, adr_id);
    }
    false
}

fn link_test(front: &mut types::FeatureFrontMatter, id: &str, test: Option<String>) -> bool {
    if let Some(test_id) = test {
        if !front.tests.contains(&test_id) {
            front.tests.push(test_id.clone());
            println!("Linked {} -> {}", id, test_id);
            return true;
        }
        println!("{} already linked to {}", id, test_id);
    }
    false
}

fn link_dep(
    front: &mut types::FeatureFrontMatter,
    id: &str,
    dep_id: &str,
    f: &types::Feature,
    graph: &graph::KnowledgeGraph,
) -> Result<bool, Box<dyn std::error::Error>> {
    if !graph.features.contains_key(dep_id) {
        return Err(Box::new(ProductError::NotFound(format!("feature {}", dep_id))));
    }
    if !front.depends_on.contains(&dep_id.to_string()) {
        // Validate no cycle would be introduced
        front.depends_on.push(dep_id.to_string());
        let mut test_features: Vec<types::Feature> = graph.features.values().cloned().collect();
        // Replace the feature with our modified version for cycle check
        test_features.retain(|tf| tf.front.id != id);
        test_features.push(types::Feature {
            front: front.clone(),
            body: f.body.clone(),
            path: f.path.clone(),
        });
        let test_graph = graph::KnowledgeGraph::build(test_features, vec![], vec![]);
        if let Err(ProductError::DependencyCycle { cycle }) = test_graph.topological_sort() {
            front.depends_on.retain(|d| d != dep_id);
            return Err(Box::new(ProductError::DependencyCycle { cycle }));
        }
        println!("Linked {} depends-on {}", id, dep_id);
        return Ok(true);
    }
    println!("{} already depends on {}", id, dep_id);
    Ok(false)
}

pub(crate) fn feature_status(id: &str, new_status: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, _, graph) = load_graph_typed()?;

    let status: types::FeatureStatus = new_status
        .parse()
        .map_err(ProductError::ConfigError)?;

    // FT-055 / ADR-047 — when severity is `error`, refuse `planned →
    // in-progress` transitions for features missing required sections.
    if status == types::FeatureStatus::InProgress
        && matches!(
            config.features.completeness_severity,
            product_core::config::CompletenessSeverity::Error
        )
    {
        if let Some(feature) = graph.features.get(id) {
            if let Some(diag) = product_core::graph::functional_spec_validation::check_feature(
                feature,
                &config.features,
            ) {
                let detail = diag.detail.unwrap_or_default();
                return Err(ProductError::ConfigError(format!(
                    "error[W030]: feature body missing required section\n  --> {}\n   |   {}\n   = hint: add the missing sections, or set [features].completeness-severity = \"warning\" in product.toml",
                    diag.file
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                    detail.replace('\n', "\n   |   ")
                )));
            }
        }
    }

    let plan = feat::plan_status_change(&graph, id, status)?;
    feat::apply_status_change(&plan)?;

    let mut lines = vec![format!("{} status -> {}", id, status)];
    if !plan.orphaned_tests.is_empty() {
        lines.push("Auto-orphaning test criteria linked to abandoned feature:".to_string());
        for upd in &plan.orphaned_tests {
            lines.push(format!(
                "  {} — removed {} from validates.features",
                upd.test_id, id
            ));
        }
    }
    Ok(Output::text(lines.join("\n")))
}

/// FT-104: reject a default-acknowledged cross-cutting ADR for a feature
pub(crate) fn feature_reject(adr_id: &str, feature_id: &str, reason: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_config, _root, graph) = load_graph()?;

    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "E011: rejection requires a non-empty reason".to_string(),
        )
        .into());
    }

    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    let mut front = feature.front.clone();

    // Check if already rejected
    if let Some(existing) = front.adrs_rejected.iter_mut().find(|r| r.id == adr_id) {
        // Update reason (idempotent)
        existing.reason = reason.to_string();
        println!("Updated rejection reason for {} on {}", adr_id, feature_id);
    } else {
        // Add new rejection
        front.adrs_rejected.push(types::AdrRejection {
            id: adr_id.to_string(),
            reason: reason.to_string(),
        });
        println!("Added rejection of {} to {}", adr_id, feature_id);
    }

    // Write the updated frontmatter
    let content = parser::render_feature(&front, &feature.body);
    fileops::write_file_atomic(&feature.path, &content)?;

    Ok(())
}

pub(crate) use super::feature_fields::{
    feature_acknowledge, feature_depends_on, feature_domain,
};
