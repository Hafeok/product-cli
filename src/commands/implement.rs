//! Implement pipeline, verify test criteria.

use product_lib::implement;

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn handle_implement(id: &str, dry_run: bool, no_verify: bool, headless: bool) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    implement::run_implement(id, &config, &root, &graph, dry_run, no_verify, headless)?;
    Ok(())
}

pub(crate) fn handle_verify(id: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    implement::run_verify(id, &config, &root, &graph)?;
    Ok(())
}
