//! Implement pipeline, verify test criteria.

use product_lib::implement;

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn handle_implement(id: &str, dry_run: bool, no_verify: bool, headless: bool) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    implement::run_implement(id, &config, &root, &graph, dry_run, no_verify, headless)?;
    Ok(())
}

pub(crate) fn handle_verify(id: Option<&str>, platform: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    if platform {
        implement::run_verify_platform(&config, &root, &graph)?;
    } else {
        let feature_id = id.ok_or("feature ID is required unless --platform is used")?;
        implement::run_verify(feature_id, &config, &root, &graph)?;
    }
    Ok(())
}
