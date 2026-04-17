//! Graph operations: check, rebuild, query, stats, centrality, autolink, coverage, infer.

use clap::Subcommand;
use product_lib::{context::summary as bundle_summary, domains, graph::{inference, responsibility}, rdf};
use std::process;

use super::graph_autolink::graph_autolink;
use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum GraphCommands {
    /// Validate all links and report errors/warnings
    Check {
        /// Output as JSON (for CI)
        #[arg(long)]
        format: Option<String>,
    },
    /// Regenerate index.ttl from all front-matter
    Rebuild,
    /// Execute a SPARQL query over the graph
    Query {
        /// SPARQL query string
        query: String,
    },
    /// Show graph statistics
    Stats,
    /// Show top ADRs by betweenness centrality
    Central {
        /// Number of results
        #[arg(long, default_value = "10")]
        top: usize,
        /// Show all ADRs
        #[arg(long)]
        all: bool,
    },
    /// Auto-link TCs to features via shared ADRs
    Autolink {
        /// Only show what would be linked (don't write)
        #[arg(long)]
        dry_run: bool,
    },
    /// Infer transitive TC→Feature links from shared ADRs (ADR-027)
    Infer {
        /// Only show what would be linked (don't write)
        #[arg(long)]
        dry_run: bool,
        /// Scope to a specific ADR
        #[arg(long)]
        adr: Option<String>,
        /// Scope to a specific feature
        #[arg(long)]
        feature: Option<String>,
    },
    /// Show feature x domain coverage matrix
    Coverage {
        /// Filter to a specific domain
        #[arg(long)]
        domain: Option<String>,
        /// Output as JSON
        #[arg(long)]
        format: Option<String>,
    },
}

pub(crate) fn handle_graph(cmd: GraphCommands, global_format: &str) -> BoxResult {
    match cmd {
        GraphCommands::Check { format } => graph_check(format, global_format),
        GraphCommands::Rebuild => graph_rebuild(),
        GraphCommands::Query { query } => graph_query(&query),
        GraphCommands::Stats => graph_stats(),
        GraphCommands::Central { top, all } => graph_central(top, all),
        GraphCommands::Autolink { dry_run } => graph_autolink(dry_run),
        GraphCommands::Infer { dry_run, adr, feature } => graph_infer(dry_run, adr, feature),
        GraphCommands::Coverage { domain, format } => graph_coverage(domain, format, global_format),
    }
}

fn graph_check(format: Option<String>, global_format: &str) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let mut result = graph.check();
    domains::validate_domains(&graph, &config.domains, &mut result.errors, &mut result.warnings);
    responsibility::check_responsibility(&graph, config.responsibility(), &mut result);
    for w in config.validate_product_section() { eprintln!("{}", w); }

    // FT-042, ADR-039 decision 10: wire log verification into graph check.
    if config.log.verify_on_check {
        append_log_findings_to_check(&config, &root, &mut result);
    }

    let fmt = format.as_deref().unwrap_or(global_format);

    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&result.to_json())?);
        let code = result.exit_code();
        if code != 0 { process::exit(code); }
    } else {
        result.print_stderr();
        let code = result.exit_code();
        match code {
            0 => eprintln!("Graph check: clean (no errors, no warnings)"),
            1 => eprintln!("Graph check: {} error(s)", result.errors.len()),
            2 => eprintln!("Graph check: {} warning(s)", result.warnings.len()),
            _ => {}
        }
        process::exit(code);
    }
    Ok(())
}

/// FT-042: append log-verification findings to the graph check result.
fn append_log_findings_to_check(
    config: &product_lib::config::ProductConfig,
    root: &std::path::Path,
    result: &mut product_lib::error::CheckResult,
) {
    use product_lib::error::Diagnostic;
    use product_lib::request_log::{log_path, verify::{verify_log, Severity, VerifyOptions}};

    let lp = log_path(root, Some(&config.paths.requests));
    if !lp.exists() {
        return;
    }
    let outcome = verify_log(&lp, root, &VerifyOptions::default());
    for f in outcome.findings {
        let mut diag = match f.severity {
            Severity::Error => Diagnostic::error(&f.code, &f.message),
            Severity::Warning => Diagnostic::warning(&f.code, &f.message),
        };
        diag = diag.with_file(lp.clone());
        if let Some(line) = f.line {
            diag = diag.with_line(line);
        }
        if let Some(detail) = f.detail {
            diag = diag.with_detail(&detail);
        }
        match f.severity {
            Severity::Error => result.errors.push(diag),
            Severity::Warning => result.warnings.push(diag),
        }
    }
}

fn graph_rebuild() -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    let graph_dir = config.resolve_path(&root, &config.paths.graph);
    std::fs::create_dir_all(&graph_dir)?;
    let path = graph_dir.join("index.ttl");
    rdf::write_index_ttl(&graph, &path)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn graph_query(query: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let result = rdf::sparql_query(&graph, query)?;
    print!("{}", result);
    Ok(())
}

fn graph_stats() -> BoxResult {
    let start = std::time::Instant::now();
    let (config, _, graph) = load_graph()?;
    let parse_time = start.elapsed();

    let centrality_start = std::time::Instant::now();
    let stats = graph.stats();
    let centrality_time = centrality_start.elapsed();

    let total_time = start.elapsed();

    // Link density: edges / (nodes * (nodes - 1)), 0 if < 2 nodes
    let link_density = if stats.total_nodes > 1 {
        stats.total_edges as f64 / (stats.total_nodes * (stats.total_nodes - 1)) as f64
    } else {
        0.0
    };

    print_stats_summary(&stats, link_density, parse_time, centrality_time, total_time);
    print_centrality_summary(&stats);
    print_bundle_summary(&graph, &config);
    Ok(())
}

/// Print the aggregate bundle-size summary section (FT-040).
/// Emits W012 on stderr when any feature has no `bundle` block.
fn print_bundle_summary(graph: &product_lib::graph::KnowledgeGraph, config: &product_lib::config::ProductConfig) {
    let summary = bundle_summary::compute_summary(graph, config);
    println!();
    print!("{}", bundle_summary::render_summary(&summary));
    if !summary.unmeasured.is_empty() {
        eprintln!(
            "warning[W012]: {} feature(s) have no bundle block \u{2014} run `product context --measure-all`",
            summary.unmeasured.len()
        );
    }
}

fn print_stats_summary(
    stats: &product_lib::graph::GraphStats,
    link_density: f64,
    parse_time: std::time::Duration,
    centrality_time: std::time::Duration,
    total_time: std::time::Duration,
) {
    println!("Graph Statistics");
    println!("================");
    println!("  Features:      {}", stats.features);
    println!("  ADRs:          {}", stats.adrs);
    println!("  Tests:         {}", stats.tests);
    println!("  Total nodes:   {}", stats.total_nodes);
    println!("  Total edges:   {}", stats.total_edges);
    println!("  Link density:  {:.3}", link_density);
    println!("  Formal coverage (invariant/chaos): {}%", stats.formal_coverage);
    println!();
    println!("  Timing:");
    println!("    Parse:      {:.1}ms", parse_time.as_secs_f64() * 1000.0);
    println!("    Centrality: {:.1}ms", centrality_time.as_secs_f64() * 1000.0);
    println!("    Total:      {:.1}ms", total_time.as_secs_f64() * 1000.0);
}

fn print_centrality_summary(stats: &product_lib::graph::GraphStats) {
    if !stats.adr_centrality.is_empty() {
        let mut sorted: Vec<_> = stats.adr_centrality.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let max = sorted.first().map(|(_, c)| *c).unwrap_or(0.0);
        let min = sorted.last().map(|(_, c)| *c).unwrap_or(0.0);
        let mean: f64 =
            sorted.iter().map(|(_, c)| c).sum::<f64>() / sorted.len().max(1) as f64;
        println!();
        println!(
            "  ADR centrality: mean={:.3}, max={:.3}, min={:.3}",
            mean, max, min
        );

        let hubs: Vec<_> = sorted
            .iter()
            .filter(|(_, c)| *c > 0.5)
            .map(|(id, _)| id.as_str())
            .collect();
        if !hubs.is_empty() {
            println!("  Structural hubs (>0.5): {}", hubs.join(", "));
        }
    }
}

fn graph_central(top: usize, all: bool) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let centrality = graph.betweenness_centrality();
    let mut adr_centrality: Vec<(String, f64)> = graph
        .adrs
        .keys()
        .map(|id| (id.clone(), centrality.get(id).copied().unwrap_or(0.0)))
        .collect();
    adr_centrality
        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let limit = if all { adr_centrality.len() } else { top.min(adr_centrality.len()) };
    println!(
        "{:<6} {:<10} {:<12} TITLE",
        "RANK", "ID", "CENTRALITY"
    );
    println!("{}", "-".repeat(60));
    for (i, (id, c)) in adr_centrality.iter().take(limit).enumerate() {
        let title = graph
            .adrs
            .get(id)
            .map(|a| a.front.title.as_str())
            .unwrap_or("");
        println!("{:<6} {:<10} {:<12.3} {}", i + 1, id, c, title);
    }
    Ok(())
}

fn graph_infer(dry_run: bool, adr: Option<String>, feature: Option<String>) -> BoxResult {
    let _lock = if !dry_run {
        Some(acquire_write_lock()?)
    } else {
        None
    };
    let (_, _, graph) = load_graph()?;
    let opts = inference::InferenceOptions {
        skip_cross_cutting: true,
        adr_filter: adr,
        feature_filter: feature,
    };
    inference::run_inference(&graph, &opts, dry_run)?;
    Ok(())
}

fn graph_coverage(domain: Option<String>, format: Option<String>, global_format: &str) -> BoxResult {
    let (config, _, graph) = load_graph()?;
    let matrix = domains::build_coverage_matrix(&graph, &config.domains);
    let fmt = format.as_deref().unwrap_or(global_format);
    if fmt == "json" {
        let json = domains::coverage_matrix_to_json(&matrix);
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        print!("{}", domains::render_coverage_matrix_filtered(&matrix, &graph, domain.as_deref()));
    }
    Ok(())
}
