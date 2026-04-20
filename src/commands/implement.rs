//! Implement pipeline, verify test criteria.

use product_lib::{implement, verify::pipeline};
use std::process;

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn handle_implement(id: &str, dry_run: bool, no_verify: bool, headless: bool) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    implement::run_implement(id, &config, &root, &graph, dry_run, no_verify, headless)?;
    Ok(())
}

pub(crate) fn handle_verify(
    id: Option<&str>,
    platform: bool,
    skip_adr_check: bool,
    phase: Option<u32>,
    ci: bool,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;

    // --platform keeps the existing per-platform behaviour unchanged.
    if platform {
        let any_fail = implement::run_verify_platform(&config, &root, &graph)?;
        if any_fail {
            process::exit(1);
        }
        return Ok(());
    }

    // Per-feature form: positional argument — preserves ADR-021 behaviour.
    if let Some(feature_id) = id {
        implement::run_verify(feature_id, &config, &root, &graph, skip_adr_check)?;
        return Ok(());
    }

    // Unified pipeline (FT-044, ADR-040).
    let scope = pipeline::PipelineScope { phase };
    let result = pipeline::run_all(&config, &root, &graph, &scope);

    if ci {
        println!("{}", pipeline::render_json(&result));
    } else {
        print!("{}", pipeline::render_pretty(&result));
    }

    let code = result.exit_code();
    if code != 0 {
        process::exit(code);
    }
    Ok(())
}
