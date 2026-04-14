//! Content hash operations (ADR-032, FT-034).

use clap::Subcommand;
use product_lib::{error::ProductError, fileops, graph::KnowledgeGraph, hash, parser, types};

use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum HashCommands {
    /// Compute and write content-hash for a TC (ADR-032)
    Seal {
        /// TC ID (omit with --all-unsealed to seal all)
        id: Option<String>,
        /// Seal all TCs without a content-hash
        #[arg(long)]
        all_unsealed: bool,
    },
    /// Verify content-hashes independently of full graph check (ADR-032)
    Verify {
        /// Artifact ID (ADR or TC, omit for all)
        id: Option<String>,
    },
}

pub(crate) fn handle_hash(cmd: HashCommands) -> BoxResult {
    match cmd {
        HashCommands::Seal { id, all_unsealed } => hash_seal(id, all_unsealed),
        HashCommands::Verify { id } => hash_verify(id),
    }
}

fn hash_seal(id: Option<String>, all_unsealed: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;

    if all_unsealed {
        hash_seal_all(&graph)
    } else {
        hash_seal_one(id, &graph)
    }
}

fn hash_seal_all(graph: &KnowledgeGraph) -> BoxResult {
    let mut sealed = 0;
    let mut skipped = 0;
    let mut tests: Vec<&types::TestCriterion> = graph.tests.values().collect();
    tests.sort_by_key(|t| &t.front.id);
    for t in tests {
        if t.front.content_hash.is_some() {
            skipped += 1;
            continue;
        }
        if t.body.trim().is_empty() {
            continue;
        }
        let h = hash::seal_tc(t);
        let mut front = t.front.clone();
        front.content_hash = Some(h.clone());
        let content = parser::render_test(&front, &t.body);
        fileops::write_file_atomic(&t.path, &content)?;
        println!("  sealed {} -> {}", t.front.id, h);
        sealed += 1;
    }
    println!("{} TC(s) sealed, {} already sealed", sealed, skipped);
    Ok(())
}

fn hash_seal_one(id: Option<String>, graph: &KnowledgeGraph) -> BoxResult {
    let tc_id = id.ok_or_else(|| {
        ProductError::ConfigError(
            "specify a TC ID or use --all-unsealed".to_string(),
        )
    })?;
    let t = graph
        .tests
        .get(&tc_id)
        .ok_or_else(|| ProductError::NotFound(format!("TC {}", tc_id)))?;
    if t.front.content_hash.is_some() {
        println!("{} is already sealed", tc_id);
        return Ok(());
    }
    let h = hash::seal_tc(t);
    let mut front = t.front.clone();
    front.content_hash = Some(h.clone());
    let content = parser::render_test(&front, &t.body);
    fileops::write_file_atomic(&t.path, &content)?;
    println!("{} sealed: content-hash = {}", tc_id, h);
    Ok(())
}

fn hash_verify(id: Option<String>) -> BoxResult {
    let (_, _, graph) = load_graph()?;

    let (adrs, tests): (Vec<&types::Adr>, Vec<&types::TestCriterion>) = if let Some(ref artifact_id) = id {
        if let Some(a) = graph.adrs.get(artifact_id.as_str()) {
            (vec![a], vec![])
        } else if let Some(t) = graph.tests.get(artifact_id.as_str()) {
            (vec![], vec![t])
        } else {
            return Err(Box::new(ProductError::NotFound(format!(
                "artifact {}",
                artifact_id
            ))));
        }
    } else {
        let adrs: Vec<&types::Adr> = graph.adrs.values().collect();
        let tests: Vec<&types::TestCriterion> = graph.tests.values().collect();
        (adrs, tests)
    };

    let result = hash::verify_all(&adrs, &tests);
    result.print_stderr();

    let exit_code = result.exit_code();
    if exit_code == 0 {
        println!("All content-hashes verified.");
    }
    std::process::exit(exit_code);
}
