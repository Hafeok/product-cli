//! Architectural fitness functions: record, threshold, trend.

use clap::Subcommand;
use product_lib::{config::ProductConfig, graph::KnowledgeGraph, metrics};
use std::path::Path;
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum MetricsCommands {
    /// Record a metric snapshot to metrics.jsonl
    Record,
    /// Check current metrics against thresholds
    Threshold,
    /// Show metric trends
    Trend {
        /// Metric name (optional — shows all if omitted)
        #[arg(long)]
        metric: Option<String>,
    },
}

pub(crate) fn handle_metrics(cmd: MetricsCommands) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let jsonl_path = root.join("metrics.jsonl");

    match cmd {
        MetricsCommands::Record => metrics_record(&graph, &root, &jsonl_path),
        MetricsCommands::Threshold => metrics_threshold(&graph, &root, &config),
        MetricsCommands::Trend { metric } => metrics_trend(&jsonl_path, metric),
    }
}

fn metrics_record(
    graph: &KnowledgeGraph,
    root: &Path,
    jsonl_path: &Path,
) -> BoxResult {
    let snapshot = metrics::record(graph, root);
    metrics::append_snapshot(&snapshot, jsonl_path)?;
    print!("{}", metrics::render_summary(&snapshot));
    println!("Appended to {}", jsonl_path.display());
    Ok(())
}

fn metrics_threshold(
    graph: &KnowledgeGraph,
    root: &Path,
    config: &ProductConfig,
) -> BoxResult {
    let snapshot = metrics::record(graph, root);
    let thresholds = config
        .metrics
        .as_ref()
        .map(|m| &m.thresholds)
        .cloned()
        .unwrap_or_default();
    let (errors, warnings) = metrics::check_thresholds(&snapshot, &thresholds);

    for w in &warnings {
        eprintln!("warning: {}", w);
    }
    for e in &errors {
        eprintln!("error: {}", e);
    }

    if !errors.is_empty() {
        process::exit(1);
    } else if !warnings.is_empty() {
        process::exit(2);
    }
    Ok(())
}

fn metrics_trend(jsonl_path: &Path, metric: Option<String>) -> BoxResult {
    let (snapshots, warnings) = metrics::load_snapshots_with_warnings(jsonl_path);
    for w in &warnings {
        eprintln!("{}", w);
    }
    if snapshots.is_empty() {
        println!("No snapshots found. Run `product metrics record` first.");
        return Ok(());
    }
    match metric {
        Some(name) => {
            print!("{}", metrics::render_trend(&snapshots, &name));
        }
        None => {
            let last = snapshots.last();
            if let Some(s) = last {
                print!("{}", metrics::render_summary(s));
            }
            println!();
            for name in &[
                "spec_coverage",
                "test_coverage",
                "exit_criteria_coverage",
                "phi",
                "gap_density",
                "gap_resolution_rate",
                "drift_density",
                "centrality_stability",
            ] {
                print!("{}", metrics::render_trend(&snapshots, name));
            }
        }
    }
    Ok(())
}
