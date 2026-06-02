//! `product graph autolink` — auto-link TCs to features via shared ADRs.

use product_lib::{fileops, parser};
use std::collections::HashMap;

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn graph_autolink(dry_run: bool) -> BoxResult {
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
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
    // Build a map: ADR ID -> list of feature IDs that link to it
    let mut adr_to_features: HashMap<String, Vec<String>> = HashMap::new();
    for f in graph.features.values() {
        for adr_id in &f.front.adrs {
            adr_to_features
                .entry(adr_id.clone())
                .or_default()
                .push(f.front.id.clone());
        }
    }

    // For each TC that validates an ADR, find which features share that ADR
    let mut feature_adds: HashMap<String, Vec<String>> = HashMap::new();
    let mut tc_adds: HashMap<String, Vec<String>> = HashMap::new();

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
    adr_to_features: &HashMap<String, Vec<String>>,
    graph: &product_lib::graph::KnowledgeGraph,
    feature_adds: &mut HashMap<String, Vec<String>>,
    tc_adds: &mut HashMap<String, Vec<String>>,
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

fn print_autolink_dry_run(feature_adds: &HashMap<String, Vec<String>>) {
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
    feature_adds: &HashMap<String, Vec<String>>,
    tc_adds: &HashMap<String, Vec<String>>,
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
