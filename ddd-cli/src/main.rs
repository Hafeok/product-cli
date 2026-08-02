//! DDD governance CLI entry point — clap dispatch only.

#![deny(clippy::unwrap_used)]

mod commands;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ddd",
    about = "Decision-Driven Design governance over the repo-local .ddd/ graph",
    version
)]
struct Cli {
    /// Path to the repo root holding .ddd/ (default: walk up from the cwd)
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli.command, cli.root) {
        eprintln!("{e}");
        std::process::exit(e.exit_code());
    }
}
