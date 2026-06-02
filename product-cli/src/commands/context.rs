//! Context bundle assembly for LLM agents — `--target` selection (FT-063),
//! template management, and the legacy measurement path.

use product_lib::{
    context::{self, summary as bundle_summary, template},
    error::ProductError,
    fileops, parser, types,
};
use std::path::Path;
use std::process;

use super::{load_graph, BoxResult};

mod templates;

pub(crate) struct ContextArgs<'a> {
    pub id: Option<&'a str>,
    pub depth: usize,
    pub phase: Option<u32>,
    pub adrs_only: bool,
    pub order: Option<String>,
    pub measure: bool,
    pub measure_all: bool,
    pub target: Option<String>,
    pub for_llm: bool,
    pub show: Option<String>,
    pub where_flag: bool,
    pub reset: Option<String>,
}

pub(crate) fn handle_context(args: ContextArgs<'_>) -> BoxResult {
    if is_templates_cmd(&args) {
        return templates::dispatch(args.show, args.where_flag, args.reset);
    }
    let (config, root, graph) = load_graph()?;
    let order_by_centrality = args.order.as_deref() != Some("id");
    if args.measure_all {
        return handle_measure_all(&config, &root, &graph, args.depth, order_by_centrality);
    }
    let id = match args.id {
        Some(v) => v,
        None => {
            eprintln!("error: the ID argument is required unless --measure-all is passed");
            process::exit(2);
        }
    };
    if args.for_llm && args.target.is_some() {
        eprintln!("{}", ProductError::ConflictingTargetFlags);
        process::exit(1);
    }
    let effective_target = resolve_effective_target(&args, &config, &graph, id);
    if let Some(p) = args.phase {
        let bundle =
            context::bundle_phase(&graph, p, args.depth, args.adrs_only, order_by_centrality);
        print!("{}", bundle);
        return Ok(());
    }
    render_artifact(
        &args,
        &config,
        &root,
        &graph,
        id,
        order_by_centrality,
        effective_target,
    )
}

fn is_templates_cmd(args: &ContextArgs<'_>) -> bool {
    args.id == Some("templates")
        || args.show.is_some()
        || args.where_flag
        || args.reset.is_some()
}

/// Resolve the effective target name (FT-063 selection rule).
///
/// Priority: `--for-llm` (alias for `claude-opus`) → explicit `--target` →
/// `[context].default-target` from product.toml → `human` (fallback).
/// Returns `None` only for non-feature artifacts (ADRs, phase bundles)
/// that are not covered by feature-bundle templates.
fn resolve_effective_target(
    args: &ContextArgs<'_>,
    config: &product_lib::config::ProductConfig,
    graph: &product_lib::graph::KnowledgeGraph,
    id: &str,
) -> Option<String> {
    if args.for_llm {
        eprintln!("Note: --for-llm is a deprecated alias for --target claude-opus.");
        eprintln!("      Update your scripts to use --target NAME explicitly.");
        return Some("claude-opus".to_string());
    }
    if let Some(t) = args.target.clone() {
        return Some(t);
    }
    if !graph.adrs.contains_key(id) && args.phase.is_none() {
        return Some(
            config
                .context
                .default_target
                .clone()
                .unwrap_or_else(|| "human".to_string()),
        );
    }
    None
}

fn render_artifact(
    args: &ContextArgs<'_>,
    config: &product_lib::config::ProductConfig,
    root: &Path,
    graph: &product_lib::graph::KnowledgeGraph,
    id: &str,
    order_by_centrality: bool,
    effective_target: Option<String>,
) -> BoxResult {
    let product_info = config.responsibility().map(|resp| context::BundleProductInfo {
        product_name: config.product_name(),
        responsibility: resp,
    });
    if graph.features.contains_key(id) {
        if let Some(target_name) = effective_target {
            return render_with_template(root, graph, id, args.depth, &target_name, config, args.measure);
        }
        match context::bundle_feature_with_product(graph, id, args.depth, order_by_centrality, product_info) {
            Some(bundle) => {
                if args.measure {
                    measure_and_write(id, graph, &bundle, root)?;
                }
                print!("{}", bundle);
            }
            None => eprintln!("Feature {} not found", id),
        }
    } else if graph.adrs.contains_key(id) {
        match context::bundle_adr(graph, id, args.depth) {
            Some(bundle) => print!("{}", bundle),
            None => eprintln!("ADR {} not found", id),
        }
    } else {
        eprintln!("Artifact {} not found", id);
        process::exit(1);
    }
    Ok(())
}

fn render_with_template(
    root: &Path,
    graph: &product_lib::graph::KnowledgeGraph,
    feature_id: &str,
    depth: usize,
    target: &str,
    config: &product_lib::config::ProductConfig,
    measure: bool,
) -> BoxResult {
    if target == "legacy" {
        return render_legacy_bundle(graph, feature_id, depth, config, root, measure);
    }
    let outcome = template::resolve_all(root);
    let resolved = match outcome.resolved.get(target) {
        Some(t) => t.clone(),
        None => {
            let mut available: Vec<String> = outcome.resolved.keys().cloned().collect();
            available.sort();
            eprintln!(
                "{}",
                ProductError::UnknownTarget {
                    name: target.to_string(),
                    available,
                }
            );
            process::exit(1);
        }
    };
    let pi = config.responsibility().map(|resp| template::ProductInfo {
        name: config.product_name(),
        responsibility: resp,
    });
    let rendered = match template::render_feature(graph, feature_id, depth, &resolved, pi) {
        Some(r) => r,
        None => {
            eprintln!("Feature {} not found", feature_id);
            process::exit(1);
        }
    };
    if rendered.exceeded_hard_max {
        eprintln!(
            "warning: bundle ({} approx tokens) exceeds template hard_max",
            rendered.token_count_approx,
        );
    } else if rendered.exceeded_target_max {
        eprintln!(
            "note: bundle ({} approx tokens) exceeds template target_max",
            rendered.token_count_approx,
        );
    }
    if measure {
        // FT-040 / FT-071: --measure should also work under the template
        // path so `bundle.patterns` lands in front-matter regardless of
        // which renderer produced the bytes.
        measure_and_write(feature_id, graph, &rendered.content, root)?;
    }
    print!("{}", rendered.content);
    Ok(())
}

/// Render the legacy AISP-framed bundle.
///
/// Reachable via the synthetic `--target legacy` escape hatch (and the
/// equivalent MCP `target: "legacy"` argument). The legacy bundler predates
/// FT-063's per-model templates and emits AISP triples + a `Context Bundle:`
/// header. Kept reachable so pre-FT-063 callers and integration tests that
/// validate the AISP renderer continue to work.
fn render_legacy_bundle(
    graph: &product_lib::graph::KnowledgeGraph,
    feature_id: &str,
    depth: usize,
    config: &product_lib::config::ProductConfig,
    root: &Path,
    measure: bool,
) -> BoxResult {
    let pi = config.responsibility().map(|resp| context::BundleProductInfo {
        product_name: config.product_name(),
        responsibility: resp,
    });
    match context::bundle_feature_with_product(graph, feature_id, depth, true, pi) {
        Some(bundle) => {
            if measure {
                measure_and_write(feature_id, graph, &bundle, root)?;
            }
            print!("{}", bundle);
            Ok(())
        }
        None => {
            eprintln!("Feature {} not found", feature_id);
            process::exit(1);
        }
    }
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
    let depth_1_adrs = feature.front.adrs.len();
    let tcs = feature.front.tests.len();
    let domains = feature.front.domains.clone();
    let tokens_approx = bundle.len() / 4;
    let measured_at = chrono::Utc::now().to_rfc3339();
    // FT-071 / ADR-050: count patterns participating in the bundle.
    let patterns = product_lib::context::collect_patterns_topo(graph, id).len();
    let bundle_metrics = types::BundleMetrics {
        depth_1_adrs,
        tcs,
        domains: domains.clone(),
        patterns,
        tokens_approx,
        measured_at: measured_at.clone(),
    };
    let mut front = feature.front.clone();
    front.bundle = Some(bundle_metrics.clone());
    let content = parser::render_feature(&front, &feature.body);
    fileops::write_file_atomic(&feature.path, &content)?;

    let metrics_path = root.join("metrics.jsonl");
    let entry = serde_json::json!({
        "feature": id,
        "depth-1-adrs": bundle_metrics.depth_1_adrs,
        "tcs": bundle_metrics.tcs,
        "domains": bundle_metrics.domains,
        "patterns": bundle_metrics.patterns,
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

fn handle_measure_all(
    config: &product_lib::config::ProductConfig,
    root: &Path,
    graph: &product_lib::graph::KnowledgeGraph,
    depth: usize,
    order_by_centrality: bool,
) -> BoxResult {
    let product_info = config.responsibility().map(|resp| context::BundleProductInfo {
        product_name: config.product_name(),
        responsibility: resp,
    });
    let mut feature_ids: Vec<&String> = graph.features.keys().collect();
    feature_ids.sort();
    for fid in &feature_ids {
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
    let (config2, _, graph2) = load_graph()?;
    let summary = bundle_summary::compute_summary(&graph2, &config2);
    print!("{}", bundle_summary::render_summary(&summary));
    Ok(())
}
