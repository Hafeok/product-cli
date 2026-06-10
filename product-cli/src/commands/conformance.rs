//! Two Pillars conformance command adapter (FT-108, ADR-052).
//!
//! Thin adapter over the `product_core::conformance` slice. Stays on
//! `BoxResult` deliberately: the check has exit-code semantics (exit 1 on
//! any MUST violation) that `CmdResult` cannot express.

use clap::Subcommand;
use product_core::conformance;
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum ConformanceCommands {
    /// Check the graph against the Two Pillars clause set (Level 3 subset)
    Check {
        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },
}

pub(crate) fn handle_conformance(cmd: ConformanceCommands, global_fmt: &str) -> BoxResult {
    match cmd {
        ConformanceCommands::Check { format } => {
            let effective_fmt = if format == "text" && global_fmt == "json" {
                global_fmt
            } else {
                format.as_str()
            };
            conformance_check(effective_fmt)
        }
    }
}

fn conformance_check(format: &str) -> BoxResult {
    let (config, _root, graph) = load_graph()?;

    let project = conformance::ProjectDeclarations {
        name: config.name.clone(),
        responsibility: config.responsibility().map(str::to_string),
        features_path: config.paths.features.clone(),
        adrs_path: config.paths.adrs.clone(),
    };
    let report = conformance::check(&graph, &project);

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print!("{}", conformance::render_report_text(&report, &config.name));
    }

    if report.has_violations() {
        process::exit(1);
    }
    Ok(())
}
