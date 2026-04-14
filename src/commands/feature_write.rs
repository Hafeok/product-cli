//! Feature write operations — creation, linking, status changes.

use product_lib::{domains, error::ProductError, fileops, graph, parser, types};

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

    changed |= link_adr(&mut front, id, adr);
    changed |= link_test(&mut front, id, test);
    if let Some(dep_id) = dep {
        changed |= link_dep(&mut front, id, &dep_id, f, &graph)?;
    }

    if changed {
        let content = parser::render_feature(&front, &f.body);
        fileops::write_file_atomic(&f.path, &content)?;
    }
    Ok(())
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
