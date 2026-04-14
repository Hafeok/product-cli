//! Context bundle assembly for LLM agents.

use product_lib::{context, error::ProductError, fileops, parser, types};
use std::process;

use super::{load_graph, BoxResult};

pub(crate) fn handle_context(
    id: &str,
    depth: usize,
    phase: Option<u32>,
    adrs_only: bool,
    order: Option<String>,
    measure: bool,
) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let order_by_centrality = order.as_deref() != Some("id");

    if let Some(p) = phase {
        let bundle = context::bundle_phase(&graph, p, depth, adrs_only, order_by_centrality);
        print!("{}", bundle);
    } else if graph.features.contains_key(id) {
        match context::bundle_feature(&graph, id, depth, order_by_centrality) {
            Some(bundle) => {
                if measure {
                    // Compute bundle metrics
                    let feature = graph.features.get(id).ok_or_else(|| {
                        ProductError::NotFound(format!("feature {}", id))
                    })?;

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
                    let (config, root, _) = load_graph()?;
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

                    let _ = config; // suppress unused warning
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
