//! Pre-flight analysis: domain coverage, cross-cutting checks.

use product_lib::domains;
use std::process;

use super::{load_graph, BoxResult};

pub(crate) fn handle_preflight(id: &str) -> BoxResult {
    let (config, _root, graph) = load_graph()?;
    let result = domains::preflight(&graph, id, &config.domains)?;
    print!("{}", domains::render_preflight(&result));
    if !result.is_clean {
        process::exit(1);
    }
    Ok(())
}
