//! ADR sealing — amend + rehash handlers.

use product_lib::{error::ProductError, fileops, hash, parser, types};

use super::{acquire_write_lock, load_graph, BoxResult};

pub fn adr_amend(id: &str, reason: Option<String>) -> BoxResult {
    let reason = reason.ok_or_else(|| {
        ProductError::ConfigError("--reason is required for amendments".to_string())
    })?;
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    let (new_hash, amendment) = hash::amend_adr(a, &reason)?;

    let mut front = a.front.clone();
    front.content_hash = Some(new_hash.clone());
    front.amendments.push(amendment);

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} amended: content-hash updated to {}", id, new_hash);
    Ok(())
}

pub fn adr_rehash(id: Option<String>, all: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;

    if all {
        rehash_all(&graph)?;
    } else {
        rehash_single(id, &graph)?;
    }
    Ok(())
}

fn rehash_all(graph: &product_lib::graph::KnowledgeGraph) -> BoxResult {
    let mut sealed = 0;
    let mut skipped = 0;
    let mut adrs: Vec<&types::Adr> = graph.adrs.values().collect();
    adrs.sort_by_key(|a| &a.front.id);
    for a in adrs {
        if a.front.status != types::AdrStatus::Accepted {
            continue;
        }
        if a.front.content_hash.is_some() {
            skipped += 1;
            continue;
        }
        let h = hash::seal_adr(a)?;
        let mut front = a.front.clone();
        front.content_hash = Some(h.clone());
        let content = parser::render_adr(&front, &a.body);
        fileops::write_file_atomic(&a.path, &content)?;
        println!("  sealed {} -> {}", a.front.id, h);
        sealed += 1;
    }
    println!("{} ADR(s) sealed, {} already sealed", sealed, skipped);
    Ok(())
}

fn rehash_single(id: Option<String>, graph: &product_lib::graph::KnowledgeGraph) -> BoxResult {
    let adr_id = id.ok_or_else(|| {
        ProductError::ConfigError("specify an ADR ID or use --all".to_string())
    })?;
    let a = graph
        .adrs
        .get(&adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
    if a.front.content_hash.is_some() {
        println!("{} is already sealed", adr_id);
        return Ok(());
    }
    let h = hash::seal_adr(a)?;
    let mut front = a.front.clone();
    front.content_hash = Some(h.clone());
    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} sealed: content-hash = {}", adr_id, h);
    Ok(())
}
