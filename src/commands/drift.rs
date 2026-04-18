//! Spec-vs-code drift detection.

use clap::Subcommand;
use product_lib::drift;
use product_lib::tags;
use product_lib::types::FeatureStatus;
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum DriftCommands {
    /// Check for drift between ADRs/features and source code
    Check {
        /// ADR or Feature ID (optional — checks all ADRs if omitted)
        adr_id: Option<String>,
        /// Explicit source files to check
        #[arg(long)]
        files: Vec<String>,
        /// Check all complete features with completion tags
        #[arg(long)]
        all_complete: bool,
    },
    /// Produce an LLM-ready drift-diff bundle on stdout (ADR-040)
    Diff {
        /// Feature ID (mutually exclusive with --all-complete / --changed)
        feature_id: Option<String>,
        /// Emit a drift-diff for every complete feature with a completion tag
        #[arg(long)]
        all_complete: bool,
        /// Emit a drift-diff for every feature touched by recent commits
        #[arg(long)]
        changed: bool,
        /// Output format: markdown (default) or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    /// Scan a source file to find governing ADRs
    Scan {
        /// Source file path
        path: String,
    },
    /// Suppress a drift finding
    Suppress {
        drift_id: String,
        #[arg(long)]
        reason: String,
    },
    /// Unsuppress a drift finding
    Unsuppress {
        drift_id: String,
    },
}

pub(crate) fn handle_drift(cmd: DriftCommands, fmt: &str) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let baseline_path = root.join("drift.json");
    let mut baseline = drift::DriftBaseline::load(&baseline_path);

    let source_roots = vec!["src".to_string(), "crates".to_string()];
    let ignore = vec!["target".to_string(), ".git".to_string(), "node_modules".to_string()];

    match cmd {
        DriftCommands::Check { adr_id, files, all_complete } => {
            if all_complete {
                drift_check_all_complete(&graph, &root, &baseline, &source_roots, &ignore, &config, fmt)
            } else if let Some(ref id) = adr_id {
                if id.starts_with("FT-") || id.starts_with(&config.prefixes.feature) {
                    drift_check_feature(id, &graph, &root, &baseline, &source_roots, &ignore, &config, fmt)
                } else {
                    drift_check(Some(id.clone()), files, &graph, &root, &baseline, &source_roots, &ignore, fmt)
                }
            } else {
                drift_check(None, files, &graph, &root, &baseline, &source_roots, &ignore, fmt)
            }
        }
        DriftCommands::Diff { feature_id, all_complete, changed, format } => {
            super::drift_diff::drift_diff(feature_id, all_complete, changed, &format, &graph, &root, &config)
        }
        DriftCommands::Scan { path } => drift_scan(&path, &graph, fmt),
        DriftCommands::Suppress { drift_id, reason } => {
            drift_suppress(&mut baseline, &drift_id, &reason, &baseline_path)
        }
        DriftCommands::Unsuppress { drift_id } => {
            drift_unsuppress(&mut baseline, &drift_id, &baseline_path)
        }
    }
}


#[allow(clippy::too_many_arguments)]
fn drift_check(
    adr_id: Option<String>,
    files: Vec<String>,
    graph: &product_lib::graph::KnowledgeGraph,
    root: &std::path::Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    fmt: &str,
) -> BoxResult {
    let all_findings: Vec<drift::DriftFinding> = if let Some(ref id) = adr_id {
        drift::check_adr(id, graph, root, baseline, source_roots, ignore, &files)
    } else {
        let adr_ids: Vec<String> = graph.adrs.keys().cloned().collect();
        let mut combined = Vec::new();
        for id in &adr_ids {
            combined.extend(drift::check_adr(id, graph, root, baseline, source_roots, ignore, &files));
        }
        combined
    };

    print_findings(&all_findings, fmt);

    let has_high = all_findings.iter().any(|f| {
        f.severity == drift::DriftSeverity::High && !f.suppressed
    });
    if has_high {
        process::exit(1);
    }
    Ok(())
}

/// Structural drift check for a feature using completion tags (ADR-036,
/// ADR-040). No LLM call — reports changed files and exits 0 (no changes) or
/// 2 (changes detected). Falls back to pattern-based drift only when no tag
/// and no ADRs are linked.
#[allow(clippy::too_many_arguments)]
fn drift_check_feature(
    feature_id: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    root: &std::path::Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    config: &product_lib::config::ProductConfig,
    fmt: &str,
) -> BoxResult {
    let depth = config.tags.implementation_depth;
    let report = match drift::structural_for_feature(feature_id, graph, root, depth) {
        Some(r) => r,
        None => {
            eprintln!("error: feature {} not found", feature_id);
            process::exit(1);
        }
    };

    // Tag path — structural only
    if report.is_git {
        if let Some(ref tag_name) = report.tag {
            if report.changed_files.is_empty() {
                if fmt == "json" {
                    let v = serde_json::json!({
                        "feature": feature_id,
                        "tag": tag_name,
                        "tag_timestamp": report.tag_timestamp,
                        "changed_files": [],
                        "status": "clean",
                    });
                    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                } else {
                    let ts = report.tag_timestamp.as_deref().unwrap_or("(unknown)");
                    println!("{} ({})", tag_name, ts);
                    println!("No changes since completion.");
                }
                return Ok(());
            }

            if fmt == "json" {
                let v = serde_json::json!({
                    "feature": feature_id,
                    "tag": tag_name,
                    "tag_timestamp": report.tag_timestamp,
                    "changed_files": report.changed_files,
                    "status": "drift",
                });
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            } else {
                let ts = report.tag_timestamp.as_deref().unwrap_or("(unknown)");
                println!("Feature: {}", feature_id);
                println!("Completion tag: {} ({})", tag_name, ts);
                println!("Changed files since completion:");
                let counts = tags::diff_stats_since_tag(root, tag_name);
                for f in &report.changed_files {
                    match counts.get(f) {
                        Some((ins, del)) => println!("  {}  (+{}/-{})", f, ins, del),
                        None => println!("  {}", f),
                    }
                }
                println!();
                println!(
                    "Hint: run `product drift diff {}` | your-llm \"check for drift\"",
                    feature_id
                );
            }
            // Respect suppressions baked into drift.json
            let drift_id = format!("DRIFT-{}-TAG-drift", feature_id);
            if baseline.is_suppressed(&drift_id) {
                return Ok(());
            }
            process::exit(2);
        }
    }

    // No tag path — emit W020 and exit 2
    eprintln!(
        "warning[W020]: no completion tag for {} — structural drift check cannot bound changes",
        feature_id
    );

    // For features that have linked ADRs, run structural ADR-level drift as a
    // best-effort fallback; otherwise just report the missing tag.
    if let Some(feature) = graph.features.get(feature_id) {
        let mut all_findings = Vec::new();
        for adr_id in &feature.front.adrs {
            all_findings.extend(drift::check_adr(
                adr_id, graph, root, baseline, source_roots, ignore, &[],
            ));
        }
        if !all_findings.is_empty() {
            print_findings(&all_findings, fmt);
            let has_high = all_findings
                .iter()
                .any(|f| f.severity == drift::DriftSeverity::High && !f.suppressed);
            if has_high {
                process::exit(1);
            }
        } else if fmt == "json" {
            println!("[]");
        } else {
            println!("No drift findings (no completion tag for {}).", feature_id);
        }
    }

    Ok(())
}

/// Check drift for all complete features that have completion tags.
#[allow(clippy::too_many_arguments)]
fn drift_check_all_complete(
    graph: &product_lib::graph::KnowledgeGraph,
    root: &std::path::Path,
    baseline: &drift::DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    config: &product_lib::config::ProductConfig,
    fmt: &str,
) -> BoxResult {
    let is_git = tags::is_git_repo(root);
    let mut all_findings = Vec::new();
    let mut checked_count = 0;

    for feature in graph.features.values() {
        if feature.front.status != FeatureStatus::Complete {
            continue;
        }

        if is_git {
            if let Some(tag_name) = tags::find_completion_tag(root, &feature.front.id) {
                checked_count += 1;
                let depth = config.tags.implementation_depth;
                let (changed_files, _diff) = tags::check_drift_since_tag(root, &tag_name, depth);

                if !changed_files.is_empty() {
                    let id = format!("DRIFT-{}-TAG-drift", feature.front.id);
                    let suppressed = baseline.is_suppressed(&id);
                    all_findings.push(drift::DriftFinding {
                        id,
                        code: "D003".to_string(),
                        severity: drift::DriftSeverity::Medium,
                        description: format!(
                            "Implementation files changed since {} was completed ({})",
                            feature.front.id, tag_name
                        ),
                        adr_id: feature.front.id.clone(),
                        source_files: changed_files,
                        suggested_action: "Review changes to ensure they don't contradict governing ADRs".to_string(),
                        suppressed,
                    });
                }
                continue;
            }
        }

        // No tag — fallback to ADR-based drift for this feature's ADRs
        for adr_id in &feature.front.adrs {
            all_findings.extend(drift::check_adr(adr_id, graph, root, baseline, source_roots, ignore, &[]));
        }
    }

    if fmt != "json" && checked_count > 0 {
        println!("Checked {} complete feature(s) with completion tags.", checked_count);
    }

    print_findings(&all_findings, fmt);

    let has_high = all_findings.iter().any(|f| {
        f.severity == drift::DriftSeverity::High && !f.suppressed
    });
    if has_high {
        process::exit(1);
    }
    Ok(())
}

fn print_findings(findings: &[drift::DriftFinding], fmt: &str) {
    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(findings).unwrap_or_default());
    } else if findings.is_empty() {
        println!("No drift findings.");
    } else {
        for f in findings {
            let suppressed_tag = if f.suppressed { " [suppressed]" } else { "" };
            println!(
                "[{:>6}] {} ({}) \u{2014} {}{}",
                f.severity, f.id, f.code, f.description, suppressed_tag
            );
            println!("         Action: {}", f.suggested_action);
            if !f.source_files.is_empty() {
                println!("         Files: {}", f.source_files.join(", "));
            }
        }
    }
}

fn drift_scan(
    path: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    fmt: &str,
) -> BoxResult {
    let source_path = std::path::Path::new(path);
    let adrs = drift::scan_source(source_path, graph);
    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&adrs).unwrap_or_default());
    } else if adrs.is_empty() {
        println!("No governing ADRs found for {}", path);
    } else {
        println!("Governing ADRs for {}:", path);
        for adr_id in &adrs {
            let title = graph
                .adrs
                .get(adr_id)
                .map(|a| a.front.title.as_str())
                .unwrap_or("(unknown)");
            println!("  {} \u{2014} {}", adr_id, title);
        }
    }
    Ok(())
}

fn drift_suppress(
    baseline: &mut drift::DriftBaseline,
    drift_id: &str,
    reason: &str,
    baseline_path: &std::path::Path,
) -> BoxResult {
    baseline.suppress(drift_id, reason);
    baseline.save(baseline_path)?;
    println!("Suppressed: {}", drift_id);
    Ok(())
}

fn drift_unsuppress(
    baseline: &mut drift::DriftBaseline,
    drift_id: &str,
    baseline_path: &std::path::Path,
) -> BoxResult {
    baseline.unsuppress(drift_id);
    baseline.save(baseline_path)?;
    println!("Unsuppressed: {}", drift_id);
    Ok(())
}
