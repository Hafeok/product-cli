//! Context bundle assembly for LLM agents.

use product_lib::{context, context::summary as bundle_summary, error::ProductError, fileops, parser, types};
use std::path::Path;
use std::process;

use super::{load_graph, BoxResult};

pub(crate) fn handle_context(
    id: Option<&str>,
    depth: usize,
    phase: Option<u32>,
    adrs_only: bool,
    order: Option<String>,
    measure: bool,
    measure_all: bool,
) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let order_by_centrality = order.as_deref() != Some("id");

    // --measure-all takes precedence: measure every feature, print summary only.
    if measure_all {
        return handle_measure_all(&config, &root, &graph, depth, order_by_centrality);
    }

    // Without --measure-all, `id` is required.
    let id = match id {
        Some(v) => v,
        None => {
            eprintln!("error: the ID argument is required unless --measure-all is passed");
            process::exit(2);
        }
    };

    // Build product info for bundle header (FT-039)
    let product_info = config.responsibility().map(|resp| {
        context::BundleProductInfo {
            product_name: config.product_name(),
            responsibility: resp,
        }
    });

    if let Some(p) = phase {
        let bundle = context::bundle_phase(&graph, p, depth, adrs_only, order_by_centrality);
        print!("{}", bundle);
    } else if graph.features.contains_key(id) {
        match context::bundle_feature_with_product(&graph, id, depth, order_by_centrality, product_info) {
            Some(bundle) => {
                if measure {
                    measure_and_write(id, &graph, &bundle, &root)?;
                }
                print!("{}", bundle);
            }
            None => eprintln!("Feature {} not found", id),
        }
    } else if graph.adrs.contains_key(id) {
        match context::bundle_adr(&graph, id, depth) {
            Some(bundle) => print!("{}", bundle),
            None => eprintln!("ADR {} not found", id),
        }
    } else {
        eprintln!("Artifact {} not found", id);
        process::exit(1);
    }
    Ok(())
}

/// Measure a single feature and update its front-matter + metrics.jsonl.
fn measure_and_write(
    id: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    bundle: &str,
    root: &Path,
) -> BoxResult {
    let feature = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    // Count depth-1 ADRs (only direct ADRs)
    let depth_1_adrs = feature.front.adrs.len();
    let tcs = feature.front.tests.len();
    let domains = feature.front.domains.clone();
    // Approximate token count: ~4 chars per token is a reasonable estimate
    let tokens_approx = bundle.len() / 4;
    let measured_at = chrono::Utc::now().to_rfc3339();

    let bundle_metrics = types::BundleMetrics {
        depth_1_adrs,
        tcs,
        domains: domains.clone(),
        tokens_approx,
        measured_at: measured_at.clone(),
    };

    // Update feature front-matter with bundle metrics
    let mut front = feature.front.clone();
    front.bundle = Some(bundle_metrics.clone());
    let content = parser::render_feature(&front, &feature.body);
    fileops::write_file_atomic(&feature.path, &content)?;

    // Append to metrics.jsonl
    let metrics_path = root.join("metrics.jsonl");
    let entry = serde_json::json!({
        "feature": id,
        "depth-1-adrs": bundle_metrics.depth_1_adrs,
        "tcs": bundle_metrics.tcs,
        "domains": bundle_metrics.domains,
        "tokens-approx": bundle_metrics.tokens_approx,
        "measured-at": bundle_metrics.measured_at,
    });
    let mut line = serde_json::to_string(&entry).unwrap_or_default();
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&metrics_path)?;
    std::io::Write::write_all(&mut file, line.as_bytes())?;

    Ok(())
}

/// FT-040: measure every feature in ID order, then print the aggregate
/// summary table. Bundle content is suppressed (not printed to stdout).
fn handle_measure_all(
    config: &product_lib::config::ProductConfig,
    root: &Path,
    graph: &product_lib::graph::KnowledgeGraph,
    depth: usize,
    order_by_centrality: bool,
) -> BoxResult {
    // Build product info once for all bundles (FT-039)
    let product_info = config.responsibility().map(|resp| {
        context::BundleProductInfo {
            product_name: config.product_name(),
            responsibility: resp,
        }
    });

    // Iterate features in ID order.
    let mut feature_ids: Vec<&String> = graph.features.keys().collect();
    feature_ids.sort();

    for fid in &feature_ids {
        // Build a fresh product_info each iteration (borrow of config).
        let pi = product_info.as_ref().map(|p| context::BundleProductInfo {
            product_name: p.product_name,
            responsibility: p.responsibility,
        });
        if let Some(bundle) = context::bundle_feature_with_product(graph, fid, depth, order_by_centrality, pi) {
            if let Err(e) = measure_and_write(fid, graph, &bundle, root) {
                eprintln!("warning: failed to measure {}: {}", fid, e);
            }
        }
    }

    // Reload graph so the bundle summary reflects the freshly-written metrics.
    let (config2, _, graph2) = load_graph()?;
    let summary = bundle_summary::compute_summary(&graph2, &config2);
    print!("{}", bundle_summary::render_summary(&summary));
    Ok(())
}
