//! Feature write operations — creation, linking, status changes.

use product_lib::{domains, error::ProductError, fileops, graph, parser, types};
use std::io::{self, BufRead, IsTerminal, Write};

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn feature_new(title: &str, phase: u32) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    let existing: Vec<String> = graph.features.keys().cloned().collect();
    let id = parser::next_id(&config.prefixes.feature, &existing);
    let filename = parser::id_to_filename(&id, title);
    let dir = config.resolve_path(&root, &config.paths.features);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(&filename);

    let front = types::FeatureFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        phase,
        status: types::FeatureStatus::Planned,
        depends_on: vec![],
        adrs: vec![],
        tests: vec![],
        domains: vec![],
        domains_acknowledged: std::collections::HashMap::new(),
        bundle: None,
    };
    let body = format!("## Description\n\n[Describe {} here.]\n", title);
    let content = parser::render_feature(&front, &body);
    fileops::write_file_atomic(&path, &content)?;
    println!("Created: {} at {}", id, path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn feature_link(
    id: &str,
    adr: Option<String>,
    test: Option<String>,
    dep: Option<String>,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_config, _root, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    let mut front = f.front.clone();
    let mut changed = false;

    let adr_linked = link_adr(&mut front, id, adr.clone());
    changed |= adr_linked;
    changed |= link_test(&mut front, id, test);
    if let Some(dep_id) = dep {
        changed |= link_dep(&mut front, id, &dep_id, f, &graph)?;
    }

    // Interactive TC inference when an ADR link was added (ADR-027)
    if adr_linked {
        if let Some(ref adr_id) = adr {
            // Check if the ADR is cross-cutting — skip if so
            let is_cross_cutting = graph
                .adrs
                .get(adr_id.as_str())
                .map(|a| a.front.scope == types::AdrScope::CrossCutting)
                .unwrap_or(false);

            if !is_cross_cutting {
                let inferred = compute_inferred_tc_links(&graph, id, adr_id);
                if !inferred.is_empty() {
                    println!();
                    println!("  Transitive TC links inferred:");
                    for (tc_id, tc_title) in &inferred {
                        println!(
                            "    {} {:<30} \u{2192} {}  (via {})",
                            tc_id, tc_title, id, adr_id
                        );
                    }
                    println!();

                    if prompt_confirm("  Add these TC links automatically? [Y/n] ") {
                        // Add TC IDs to the feature's tests list
                        for (tc_id, _) in &inferred {
                            if !front.tests.contains(tc_id) {
                                front.tests.push(tc_id.clone());
                            }
                        }
                        front.tests.sort();

                        // Prepare batch writes: feature file + TC files
                        let feature_content = parser::render_feature(&front, &f.body);
                        let mut writes: Vec<(&std::path::Path, String)> = Vec::new();
                        writes.push((&f.path, feature_content));

                        for (tc_id, _) in &inferred {
                            if let Some(tc) = graph.tests.get(tc_id.as_str()) {
                                let mut tc_front = tc.front.clone();
                                if !tc_front.validates.features.contains(&id.to_string()) {
                                    tc_front.validates.features.push(id.to_string());
                                }
                                tc_front.validates.features.sort();
                                let tc_content = parser::render_test(&tc_front, &tc.body);
                                writes.push((&tc.path, tc_content));
                            }
                        }

                        // Write atomically
                        let write_refs: Vec<(&std::path::Path, &str)> = writes
                            .iter()
                            .map(|(p, c)| (*p, c.as_str()))
                            .collect();
                        fileops::write_batch_atomic(&write_refs)?;
                        println!("  Applied {} TC links.", inferred.len());
                        return Ok(());
                    }
                    // User declined — fall through to write only the feature file
                }
            }
        }
    }

    if changed {
        let content = parser::render_feature(&front, &f.body);
        fileops::write_file_atomic(&f.path, &content)?;
    }
    Ok(())
}

/// Compute TC links that would be inferred from linking a specific ADR to a feature
fn compute_inferred_tc_links(
    graph: &graph::KnowledgeGraph,
    feature_id: &str,
    adr_id: &str,
) -> Vec<(String, String)> {
    let mut inferred = Vec::new();
    for tc in graph.tests.values() {
        if tc.front.validates.adrs.contains(&adr_id.to_string())
            && !tc.front.validates.features.contains(&feature_id.to_string())
        {
            inferred.push((tc.front.id.clone(), tc.front.title.clone()));
        }
    }
    inferred.sort_by(|a, b| a.0.cmp(&b.0));
    inferred
}

/// Prompt user for y/n confirmation, defaulting to yes
fn prompt_confirm(prompt: &str) -> bool {
    let stdin = io::stdin();
    let is_tty = stdin.is_terminal();

    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim().to_lowercase();
        // Empty (just enter) or "y"/"yes" = confirm; "n"/"no" = decline
        trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
    } else if !is_tty {
        // Non-interactive with no data: default to no
        false
    } else {
        true
    }
}

fn link_adr(front: &mut types::FeatureFrontMatter, id: &str, adr: Option<String>) -> bool {
    if let Some(adr_id) = adr {
        if !front.adrs.contains(&adr_id) {
            front.adrs.push(adr_id.clone());
            println!("Linked {} -> {}", id, adr_id);
            return true;
        }
        println!("{} already linked to {}", id, adr_id);
    }
    false
}

fn link_test(front: &mut types::FeatureFrontMatter, id: &str, test: Option<String>) -> bool {
    if let Some(test_id) = test {
        if !front.tests.contains(&test_id) {
            front.tests.push(test_id.clone());
            println!("Linked {} -> {}", id, test_id);
            return true;
        }
        println!("{} already linked to {}", id, test_id);
    }
    false
}

fn link_dep(
    front: &mut types::FeatureFrontMatter,
    id: &str,
    dep_id: &str,
    f: &types::Feature,
    graph: &graph::KnowledgeGraph,
) -> Result<bool, Box<dyn std::error::Error>> {
    if !graph.features.contains_key(dep_id) {
        return Err(Box::new(ProductError::NotFound(format!("feature {}", dep_id))));
    }
    if !front.depends_on.contains(&dep_id.to_string()) {
        // Validate no cycle would be introduced
        front.depends_on.push(dep_id.to_string());
        let mut test_features: Vec<types::Feature> = graph.features.values().cloned().collect();
        // Replace the feature with our modified version for cycle check
        test_features.retain(|tf| tf.front.id != id);
        test_features.push(types::Feature {
            front: front.clone(),
            body: f.body.clone(),
            path: f.path.clone(),
        });
        let test_graph = graph::KnowledgeGraph::build(test_features, vec![], vec![]);
        if let Err(ProductError::DependencyCycle { cycle }) = test_graph.topological_sort() {
            front.depends_on.retain(|d| d != dep_id);
            return Err(Box::new(ProductError::DependencyCycle { cycle }));
        }
        println!("Linked {} depends-on {}", id, dep_id);
        return Ok(true);
    }
    println!("{} already depends on {}", id, dep_id);
    Ok(false)
}

pub(crate) fn feature_status(id: &str, new_status: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    let status: types::FeatureStatus = new_status
        .parse()
        .map_err(|e: String| ProductError::ConfigError(e))?;

    let mut front = f.front.clone();
    front.status = status;
    let content = parser::render_feature(&front, &f.body);
    fileops::write_file_atomic(&f.path, &content)?;
    println!("{} status -> {}", id, status);

    // ADR-010: Auto-orphan tests on feature abandonment
    if status == types::FeatureStatus::Abandoned {
        orphan_tests_for_abandoned_feature(id, f, &graph)?;
    }
    Ok(())
}

fn orphan_tests_for_abandoned_feature(
    id: &str,
    f: &types::Feature,
    graph: &graph::KnowledgeGraph,
) -> BoxResult {
    println!("Auto-orphaning test criteria linked to abandoned feature:");
    for test_id in &f.front.tests {
        if let Some(tc) = graph.tests.get(test_id.as_str()) {
            let mut test_front = tc.front.clone();
            test_front.validates.features.retain(|fid| fid != id);
            let test_content = parser::render_test(&test_front, &tc.body);
            fileops::write_file_atomic(&tc.path, &test_content)?;
            println!("  {} — removed {} from validates.features", test_id, id);
        }
    }
    Ok(())
}

pub(crate) fn feature_acknowledge(
    id: &str,
    domain: Option<String>,
    adr: Option<String>,
    reason: &str,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let feature = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

    let updated_front = if let Some(ref domain_name) = domain {
        domains::acknowledge_domain(feature, domain_name, reason)?
    } else if let Some(ref adr_id) = adr {
        let adr_obj = graph
            .adrs
            .get(adr_id.as_str())
            .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
        domains::acknowledge_adr(feature, adr_obj, reason)?
    } else {
        return Err(Box::new(ProductError::ConfigError(
            "must specify --domain or --adr".to_string(),
        )));
    };

    let content = parser::render_feature(&updated_front, &feature.body);
    fileops::write_file_atomic(&feature.path, &content)?;
    if let Some(ref d) = domain {
        println!("{} acknowledged domain '{}': {}", id, d, reason);
    } else if let Some(ref a) = adr {
        println!("{} acknowledged ADR '{}': {}", id, a, reason);
    }
    Ok(())
}
