//! Verification pipeline — product verify FT-XXX (ADR-021)

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::parser;
use crate::tags;
use crate::types::*;
use crate::fileops;
use std::path::Path;
use std::process::Command;

use super::runner::{self, TcResult, extract_yaml_field, extract_yaml_list, update_tc_status};

/// Verify all TCs linked to a feature by running their configured runners
pub fn run_verify(
    feature_id: &str,
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
    skip_adr_check: bool,
) -> Result<()> {
    let feature = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    // E016: Lifecycle gate — check that no linked ADR is still 'proposed' (ADR-034)
    if !skip_adr_check {
        let proposed_adrs: Vec<(&str, &str)> = feature
            .front
            .adrs
            .iter()
            .filter_map(|adr_id| {
                graph.adrs.get(adr_id.as_str()).and_then(|adr| {
                    if adr.front.status == AdrStatus::Proposed {
                        Some((adr.front.id.as_str(), adr.front.title.as_str()))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if !proposed_adrs.is_empty() {
            let detail_lines: Vec<String> = proposed_adrs
                .iter()
                .map(|(id, title)| format!("{} ({}) has status 'proposed'", id, title))
                .collect();
            eprintln!(
                "error[E016]: cannot verify — governing ADR not yet accepted\n  --> {}\n   = {}\n   = hint: accept the ADR first with `product adr status ADR-XXX accepted`\n           or remove the link if the ADR no longer governs this feature",
                feature.path.display(),
                detail_lines.join("\n   = "),
            );
            return Err(ProductError::LifecycleGate {
                feature_id: feature_id.to_string(),
                proposed_adrs: proposed_adrs.iter().map(|(id, _)| id.to_string()).collect(),
            });
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let tc_ids: Vec<String> = feature.front.tests.clone();
    let r = run_tc_list(&tc_ids, graph, root, config, &now)?;

    if r.unrunnable_count > 0 {
        eprintln!(
            "warning[W016]: {} TC(s) acknowledged as unrunnable for {}",
            r.unrunnable_count, feature_id
        );
    }

    let tag_created_opt = if r.any_runnable || r.has_unimplemented {
        update_feature_and_checklist(feature_id, config, root, r.all_pass, r.has_unimplemented, &tc_ids)?
    } else {
        eprintln!("warning[W001]: no runnable TCs found for {}", feature_id);
        None
    };

    // ADR-039 decision 6: verify writes a log entry.
    write_verify_log_entry(
        config,
        root,
        feature_id,
        &tc_ids,
        &r.passing,
        &r.failing,
        tag_created_opt.as_deref(),
    );

    Ok(())
}

fn write_verify_log_entry(
    config: &ProductConfig,
    root: &Path,
    feature_id: &str,
    tcs_run: &[String],
    passing: &[String],
    failing: &[String],
    tag_created: Option<&str>,
) {
    // Skip on dry-run / no-tc-run situations? No — verify always logs.
    let log_p = crate::request_log::log_path(root, Some(&config.paths.requests));
    let applied_by =
        crate::request_log::git_identity::resolve_applied_by(root)
            .unwrap_or_else(|_| "local:unknown".into());
    let commit = crate::request_log::git_identity::resolve_commit(root);
    let reason = format!("verify {}: {}/{} passing", feature_id, passing.len(), tcs_run.len());
    let _ = crate::request_log::append::append_verify_entry(
        &log_p,
        crate::request_log::append::VerifyEntryParams {
            applied_by: &applied_by,
            commit: &commit,
            reason: &reason,
            feature: feature_id,
            tcs_run: tcs_run.to_vec(),
            passing: passing.to_vec(),
            failing: failing.to_vec(),
            tag_created: tag_created.map(String::from),
        },
    );
}

/// Verify all TCs linked to cross-cutting ADRs, regardless of feature (--platform)
pub fn run_verify_platform(
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut platform_tc_ids: Vec<String> = Vec::new();
    for adr in graph.adrs.values() {
        if adr.front.scope == AdrScope::CrossCutting {
            for tc in graph.tests.values() {
                if tc.front.validates.adrs.contains(&adr.front.id) && !platform_tc_ids.contains(&tc.front.id) {
                    platform_tc_ids.push(tc.front.id.clone());
                }
            }
        }
    }
    if platform_tc_ids.is_empty() {
        eprintln!("warning[W001]: no TCs linked to cross-cutting ADRs found");
        return Ok(());
    }
    println!("  Running {} platform TC(s) linked to cross-cutting ADRs", platform_tc_ids.len());
    let _ = run_tc_list(&platform_tc_ids, graph, root, config, &now)?;

    // Regenerate checklist after status updates
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let loaded = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
    let new_graph = KnowledgeGraph::build(loaded.features, loaded.adrs, loaded.tests);
    let checklist_path = config.resolve_path(root, &config.paths.checklist);
    if let Some(parent) = checklist_path.parent() { let _ = std::fs::create_dir_all(parent); }
    fileops::write_file_atomic(&checklist_path, &crate::checklist::generate(&new_graph))
}

/// Result of running a list of TCs.
pub(crate) struct TcRunResult {
    pub all_pass: bool,
    pub any_runnable: bool,
    pub has_unimplemented: bool,
    pub unrunnable_count: usize,
    pub passing: Vec<String>,
    pub failing: Vec<String>,
}

/// Run a list of TCs.
fn run_tc_list(
    tc_ids: &[String], graph: &KnowledgeGraph, root: &Path,
    config: &ProductConfig, now: &str,
) -> Result<TcRunResult> {
    let mut all_pass = true;
    let mut any_runnable = false;
    let mut has_unimplemented = false;
    let mut unrunnable_count: usize = 0;
    let mut passing: Vec<String> = Vec::new();
    let mut failing: Vec<String> = Vec::new();

    for tc_id in tc_ids {
        let Some(tc) = graph.tests.get(tc_id.as_str()) else { continue };
        let content = std::fs::read_to_string(&tc.path).unwrap_or_default();
        let tc_runner = extract_yaml_field(&content, "runner");
        let runner_args = extract_yaml_field(&content, "runner-args");
        let requires = extract_yaml_list(&content, "requires");

        if tc.front.status == TestStatus::Unrunnable {
            println!("  {} {:<30} UNRUNNABLE (acknowledged)", tc.front.id, tc.front.title);
            unrunnable_count += 1;
            continue;
        }
        if tc_runner.is_empty() {
            println!("  {} {:<30} UNIMPLEMENTED (no runner configured)", tc.front.id, tc.front.title);
            has_unimplemented = true;
            continue;
        }

        // Check requires prerequisites (ADR-021)
        if !requires.is_empty() {
            match check_prerequisites(&requires, config, root) {
                PrereqResult::AllSatisfied => {}
                PrereqResult::NotSatisfied(name) => {
                    let msg = format!("prerequisite '{}' not satisfied", name);
                    println!("  {} {:<30} UNRUNNABLE ({})", tc.front.id, tc.front.title, msg);
                    update_tc_status(&tc.path, "unrunnable", now, Some(&msg), None)?;
                    unrunnable_count += 1;
                    continue;
                }
                PrereqResult::MissingDefinition(name) => {
                    eprintln!(
                        "error[E011]: prerequisite '{}' is not defined in [verify.prerequisites]\n   = hint: add '{}' to [verify.prerequisites] in product.toml",
                        name, name
                    );
                    return Err(ProductError::ConfigError(
                        format!("prerequisite '{}' is not defined in [verify.prerequisites]", name),
                    ));
                }
            }
        }

        any_runnable = true;
        match runner::run_tc(&tc_runner, &runner_args, root) {
            TcResult::Pass(d) => {
                println!("  {} {:<30} PASS ({:.1}s)", tc.front.id, tc.front.title, d);
                update_tc_status(&tc.path, "passing", now, None, Some(d))?;
                passing.push(tc.front.id.clone());
            }
            TcResult::Fail(d, msg) => {
                println!("  {} {:<30} FAIL ({:.1}s)", tc.front.id, tc.front.title, d);
                update_tc_status(&tc.path, "failing", now, Some(&msg), Some(d))?;
                all_pass = false;
                failing.push(tc.front.id.clone());
            }
        }
    }
    Ok(TcRunResult {
        all_pass,
        any_runnable,
        has_unimplemented,
        unrunnable_count,
        passing,
        failing,
    })
}

enum PrereqResult { AllSatisfied, NotSatisfied(String), MissingDefinition(String) }

fn check_prerequisites(requires: &[String], config: &ProductConfig, root: &Path) -> PrereqResult {
    for name in requires {
        match config.verify.prerequisites.get(name.as_str()) {
            None => return PrereqResult::MissingDefinition(name.clone()),
            Some(cmd) => {
                let ok = Command::new("bash")
                    .args(["-c", cmd])
                    .current_dir(root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !ok { return PrereqResult::NotSatisfied(name.clone()); }
            }
        }
    }
    PrereqResult::AllSatisfied
}

/// Reload the graph, update feature status, create completion tag, and regenerate the checklist.
/// Returns the created tag name, if one was created.
fn update_feature_and_checklist(
    feature_id: &str, config: &ProductConfig, root: &Path,
    all_pass: bool, has_unimplemented: bool, tc_ids: &[String],
) -> Result<Option<String>> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let loaded = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
    let new_graph = KnowledgeGraph::build(loaded.features, loaded.adrs, loaded.tests);
    let mut created_tag: Option<String> = None;

    if let Some(f) = new_graph.features.get(feature_id) {
        let new_status = if all_pass && !has_unimplemented {
            FeatureStatus::Complete
        } else {
            FeatureStatus::InProgress
        };
        if f.front.status != new_status {
            let mut front = f.front.clone();
            front.status = new_status;
            let content = parser::render_feature(&front, &f.body);
            fileops::write_file_atomic(&f.path, &content)?;
            println!();
            println!("  Feature {} status -> {}", feature_id, new_status);
        }

        // ADR-036: Create completion tag when transitioning to complete
        if new_status == FeatureStatus::Complete {
            if tags::is_git_repo(root) {
                match tags::create_completion_tag(root, feature_id, tc_ids, config) {
                    Ok(tag_name) => {
                        println!("  \u{2713} Tagged: {}", tag_name);
                        println!("    Run `git push --tags` to share.");
                        created_tag = Some(tag_name);
                    }
                    Err(e) => {
                        eprintln!(
                            "warning[W018]: failed to create completion tag: {}",
                            e
                        );
                    }
                }
            } else {
                eprintln!("warning[W018]: not a git repository \u{2014} skipping tag creation");
            }
        }
    }

    let checklist_path = config.resolve_path(root, &config.paths.checklist);
    if let Some(parent) = checklist_path.parent() { let _ = std::fs::create_dir_all(parent); }
    fileops::write_file_atomic(&checklist_path, &crate::checklist::generate(&new_graph))?;
    Ok(created_tag)
}
