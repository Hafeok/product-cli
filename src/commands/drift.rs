//! Spec-vs-code drift detection.

use clap::Subcommand;
use product_lib::drift;
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum DriftCommands {
    /// Check for drift between ADRs and source code
    Check {
        /// ADR ID (optional — checks all if omitted)
        adr_id: Option<String>,
        /// Explicit source files to check
        #[arg(long)]
        files: Vec<String>,
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
    let (_config, root, graph) = load_graph()?;
    let baseline_path = root.join("drift.json");
    let mut baseline = drift::DriftBaseline::load(&baseline_path);

    let source_roots = vec!["src".to_string(), "crates".to_string()];
    let ignore = vec!["target".to_string(), ".git".to_string(), "node_modules".to_string()];

    match cmd {
        DriftCommands::Check { adr_id, files } => {
            drift_check(adr_id, files, &graph, &root, &baseline, &source_roots, &ignore, fmt)
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

    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&all_findings).unwrap_or_default());
    } else if all_findings.is_empty() {
        println!("No drift findings.");
    } else {
        for f in &all_findings {
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

    let has_high = all_findings.iter().any(|f| {
        f.severity == drift::DriftSeverity::High && !f.suppressed
    });
    if has_high {
        process::exit(1);
    }
    Ok(())
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
