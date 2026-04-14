//! Graph operations: check, rebuild, query, stats, centrality, autolink, coverage.

use clap::Subcommand;
use product_lib::{domains, fileops, parser, rdf};
use std::process;

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
        GraphCommands::Coverage { domain, format } => graph_coverage(domain, format, global_format),
    }
}

fn graph_check(format: Option<String>, global_format: &str) -> BoxResult {
    let (config, _, graph) = load_graph()?;
    let mut result = graph.check();
    domains::validate_domains(&graph, &config.domains, &mut result.errors, &mut result.warnings);
    let fmt = format.as_deref().unwrap_or(global_format);

    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&result.to_json())?);
        let code = result.exit_code();
        if code != 0 {
            process::exit(code);
        }
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
    let (_, _, graph) = load_graph()?;
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
    Ok(())
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

fn graph_autolink(dry_run: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;

    let (feature_adds, tc_adds) = compute_autolink_candidates(&graph);

    let total_feature_links: usize = feature_adds.values().map(|v| v.len()).sum();
    let total_tc_links: usize = tc_adds.values().map(|v| v.len()).sum();

    if total_feature_links == 0 && total_tc_links == 0 {
        println!("No new links to add. Graph is already fully connected via ADRs.");
        return Ok(());
    }

    println!(
        "Autolink: {} TC\u{2192}Feature links, {} Feature\u{2192}TC links to add",
        total_tc_links, total_feature_links
    );

    if dry_run {
        print_autolink_dry_run(&feature_adds);
        return Ok(());
    }

    let (features_written, tcs_written) = write_autolink_files(&graph, &feature_adds, &tc_adds)?;
    println!(
        "  Updated {} feature files, {} TC files",
        features_written, tcs_written
    );
    Ok(())
}

fn compute_autolink_candidates(
    graph: &product_lib::graph::KnowledgeGraph,
) -> (
    std::collections::HashMap<String, Vec<String>>,
    std::collections::HashMap<String, Vec<String>>,
) {
    // Build a map: ADR ID -> list of feature IDs that link to it
    let mut adr_to_features: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for f in graph.features.values() {
        for adr_id in &f.front.adrs {
            adr_to_features
                .entry(adr_id.clone())
                .or_default()
                .push(f.front.id.clone());
        }
    }

    // For each TC that validates an ADR, find which features share that ADR
    let mut feature_adds: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut tc_adds: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for tc in graph.tests.values() {
        collect_autolink_for_tc(tc, &adr_to_features, graph, &mut feature_adds, &mut tc_adds);
    }

    // Deduplicate
    for v in feature_adds.values_mut() {
        v.sort();
        v.dedup();
    }
    for v in tc_adds.values_mut() {
        v.sort();
        v.dedup();
    }

    (feature_adds, tc_adds)
}

fn collect_autolink_for_tc(
    tc: &product_lib::types::TestCriterion,
    adr_to_features: &std::collections::HashMap<String, Vec<String>>,
    graph: &product_lib::graph::KnowledgeGraph,
    feature_adds: &mut std::collections::HashMap<String, Vec<String>>,
    tc_adds: &mut std::collections::HashMap<String, Vec<String>>,
) {
    for adr_id in &tc.front.validates.adrs {
        if let Some(feature_ids) = adr_to_features.get(adr_id) {
            for fid in feature_ids {
                if let Some(f) = graph.features.get(fid) {
                    if !f.front.tests.contains(&tc.front.id) {
                        feature_adds
                            .entry(fid.clone())
                            .or_default()
                            .push(tc.front.id.clone());
                    }
                }
                if !tc.front.validates.features.contains(fid) {
                    tc_adds
                        .entry(tc.front.id.clone())
                        .or_default()
                        .push(fid.clone());
                }
            }
        }
    }
}

fn print_autolink_dry_run(feature_adds: &std::collections::HashMap<String, Vec<String>>) {
    println!();
    let mut sorted_features: Vec<_> = feature_adds.iter().collect();
    sorted_features.sort_by_key(|(k, _)| (*k).clone());
    for (fid, tcs) in &sorted_features {
        println!("  {} += tests: [{}]", fid, tcs.join(", "));
    }
    println!();
    println!("Run without --dry-run to write these links.");
}

fn write_autolink_files(
    graph: &product_lib::graph::KnowledgeGraph,
    feature_adds: &std::collections::HashMap<String, Vec<String>>,
    tc_adds: &std::collections::HashMap<String, Vec<String>>,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut features_written = 0;
    for (fid, new_tcs) in feature_adds {
        if let Some(f) = graph.features.get(fid) {
            let mut front = f.front.clone();
            for tc_id in new_tcs {
                if !front.tests.contains(tc_id) {
                    front.tests.push(tc_id.clone());
                }
            }
            front.tests.sort();
            let content = parser::render_feature(&front, &f.body);
            fileops::write_file_atomic(&f.path, &content)?;
            features_written += 1;
        }
    }

    let mut tcs_written = 0;
    for (tc_id, new_features) in tc_adds {
        if let Some(tc) = graph.tests.get(tc_id) {
            let mut front = tc.front.clone();
            for fid in new_features {
                if !front.validates.features.contains(fid) {
                    front.validates.features.push(fid.clone());
                }
            }
            front.validates.features.sort();
            let content = parser::render_test(&front, &tc.body);
            fileops::write_file_atomic(&tc.path, &content)?;
            tcs_written += 1;
        }
    }

    Ok((features_written, tcs_written))
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
