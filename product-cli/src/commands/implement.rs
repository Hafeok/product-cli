//! Implement pipeline, verify test criteria.

use product_lib::error::ProductError;
use product_lib::{implement, verify::pipeline};
use std::process;

use super::{acquire_write_lock, load_graph, BoxResult};

pub(crate) fn handle_implement(
    id: &str,
    dry_run: bool,
    no_verify: bool,
    headless: bool,
    no_auto_runners: bool,
    target: Option<&str>,
) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    implement::run_implement(
        id,
        &config,
        &root,
        &graph,
        dry_run,
        no_verify,
        headless,
        no_auto_runners,
        target,
    )?;
    Ok(())
}

pub(crate) fn handle_verify(
    id: Option<&str>,
    platform: bool,
    skip_adr_check: bool,
    phase: Option<u32>,
    ci: bool,
    format: &str,
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
        match implement::run_verify(feature_id, &config, &root, &graph, skip_adr_check) {
            Ok(()) => return Ok(()),
            Err(err) => {
                // FT-058 / E022: emit structured JSON to stdout when the
                // caller asked for JSON output, and exit 22.
                if let ProductError::TcRunnerMissing {
                    feature_id: fid,
                    tc_ids,
                    ..
                } = &err
                {
                    if format == "json" || ci {
                        let json = serde_json::json!({
                            "error": "E022",
                            "feature_id": fid,
                            "tc_ids": tc_ids,
                        });
                        println!("{}", json);
                    }
                }
                return Err(Box::new(err));
            }
        }
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
