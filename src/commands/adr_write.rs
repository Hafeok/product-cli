//! ADR field management write operations — domain, scope, supersession, source files (FT-038).

use product_lib::{error::ProductError, fileops, graph, parser, types};

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn adr_domain(id: &str, add: Vec<String>, remove: Vec<String>) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    // Validate domains against vocabulary (E012)
    for domain in &add {
        if !config.domains.contains_key(domain) {
            return Err(Box::new(ProductError::ConfigError(format!(
                "error[E012]: unknown domain '{}'\n   = hint: check [domains] vocabulary in product.toml",
                domain
            ))));
        }
    }

    let mut front = a.front.clone();

    // Add domains (idempotent — skip duplicates)
    for domain in &add {
        if !front.domains.contains(domain) {
            front.domains.push(domain.clone());
        }
    }

    // Remove domains (idempotent — no-op if not present)
    for domain in &remove {
        front.domains.retain(|d| d != domain);
    }

    front.domains.sort();

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} domains: [{}]", id, front.domains.join(", "));
    Ok(())
}

pub(crate) fn adr_scope(id: &str, scope_str: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    let scope: types::AdrScope = scope_str.parse().map_err(|e: String| {
        ProductError::ConfigError(format!("error[E001]: {}", e))
    })?;

    let mut front = a.front.clone();
    front.scope = scope;

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} scope -> {}", id, scope);
    Ok(())
}

pub(crate) fn adr_supersede(
    id: &str,
    supersedes: Option<String>,
    remove: Option<String>,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;

    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    if let Some(ref target_id) = supersedes {
        supersede_add(id, target_id, a, &graph)?;
    } else if let Some(ref target_id) = remove {
        supersede_remove(id, target_id, a, &graph)?;
    } else {
        return Err(Box::new(ProductError::ConfigError(
            "must specify --supersedes or --remove".to_string(),
        )));
    }

    Ok(())
}

fn supersede_add(
    id: &str,
    target_id: &str,
    a: &types::Adr,
    graph: &graph::KnowledgeGraph,
) -> BoxResult {
    let target = graph
        .adrs
        .get(target_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", target_id)))?;

    // Prepare the new state to check for cycles
    let mut new_front = a.front.clone();
    if !new_front.supersedes.contains(&target_id.to_string()) {
        new_front.supersedes.push(target_id.to_string());
    }

    let mut target_front = target.front.clone();
    if !target_front.superseded_by.contains(&id.to_string()) {
        target_front.superseded_by.push(id.to_string());
    }

    // Cycle detection: build a temporary graph and check
    let mut test_adrs: Vec<types::Adr> = graph.adrs.values().cloned().collect();
    test_adrs.retain(|adri| adri.front.id != id && adri.front.id != target_id);
    test_adrs.push(types::Adr {
        front: new_front.clone(),
        body: a.body.clone(),
        path: a.path.clone(),
    });
    test_adrs.push(types::Adr {
        front: target_front.clone(),
        body: target.body.clone(),
        path: target.path.clone(),
    });
    let test_graph = graph::KnowledgeGraph::build(vec![], test_adrs, vec![]);
    if let Some(cycle) = test_graph.detect_supersession_cycle() {
        return Err(Box::new(ProductError::SupersessionCycle { cycle }));
    }

    // If target was accepted, change to superseded
    if target_front.status == types::AdrStatus::Accepted {
        target_front.status = types::AdrStatus::Superseded;
    }

    // Batch write both files atomically
    let content_a = parser::render_adr(&new_front, &a.body);
    let content_target = parser::render_adr(&target_front, &target.body);
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&a.path, &content_a),
        (&target.path, &content_target),
    ];
    fileops::write_batch_atomic(&writes)?;

    println!("{} supersedes {}", id, target_id);
    if target.front.status == types::AdrStatus::Accepted {
        println!("{} status -> superseded", target_id);
    }
    Ok(())
}

fn supersede_remove(
    id: &str,
    target_id: &str,
    a: &types::Adr,
    graph: &graph::KnowledgeGraph,
) -> BoxResult {
    let target = graph
        .adrs
        .get(target_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", target_id)))?;

    let mut new_front = a.front.clone();
    new_front.supersedes.retain(|s| s != target_id);

    let mut target_front = target.front.clone();
    target_front.superseded_by.retain(|s| s != id);

    let content_a = parser::render_adr(&new_front, &a.body);
    let content_target = parser::render_adr(&target_front, &target.body);
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&a.path, &content_a),
        (&target.path, &content_target),
    ];
    fileops::write_batch_atomic(&writes)?;

    println!("{} removed supersession link to {}", id, target_id);
    Ok(())
}

pub(crate) fn adr_source_files(
    id: &str,
    add: Vec<String>,
    remove: Vec<String>,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, root, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    // Warn (W-class) if added paths don't exist
    for path_str in &add {
        let full_path = root.join(path_str);
        if !full_path.exists() {
            eprintln!(
                "warning[W012]: path '{}' does not exist (yet) in repository",
                path_str
            );
        }
    }

    let mut front = a.front.clone();

    // Add source files (idempotent — skip duplicates)
    for path_str in &add {
        if !front.source_files.contains(path_str) {
            front.source_files.push(path_str.clone());
        }
    }

    // Remove source files (idempotent — no-op if not present)
    for path_str in &remove {
        front.source_files.retain(|s| s != path_str);
    }

    front.source_files.sort();

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} source-files: [{}]", id, front.source_files.join(", "));
    Ok(())
}
