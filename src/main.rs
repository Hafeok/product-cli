//! Product CLI entry point — clap dispatch only.

#![deny(clippy::unwrap_used)]

mod commands;

use clap::{CommandFactory, Parser};
use std::process;

#[derive(Parser)]
#[command(
    name = "product",
    about = "Knowledge graph CLI for managing features, ADRs, and test criteria",
    version
)]
struct Cli {
    /// Output format: text (default) or json
    #[arg(long, global = true, default_value = "text")]
    format: String,

    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    // Handle SIGPIPE gracefully — exit silently when piped to `head` etc.
    #[cfg(unix)]
    {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    // E017 (ADR-042): reject malformed `[tc-types].custom` before clap
    // parses anything — reserved TC-type names in the custom list must
    // pre-empt every subcommand, including `--help` and `--version`.
    if let Err(e) = early_config_check() {
        eprintln!("{e}");
        process::exit(1);
    }

    let cli = Cli::parse();
    let mut cmd = Cli::command();

    let result = commands::run(cli.command, &cli.format, &mut cmd);
    if let Err(e) = result {
        eprintln!("{e}");
        process::exit(1);
    }
}

/// Walk up from cwd looking for `product.toml`. If one is found, load it —
/// `ProductConfig::load` runs the E017 check. If no config is found we
/// silently continue.
fn early_config_check() -> Result<(), product_lib::error::ProductError> {
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            let candidate = dir.join("product.toml");
            if candidate.exists() {
                product_lib::config::ProductConfig::load(&candidate)?;
                return Ok(());
            }
            if !dir.pop() {
                return Ok(());
            }
        }
    }
    Ok(())
}
