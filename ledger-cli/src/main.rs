//! Decision Ledger CLI entry point — clap dispatch only.

#![deny(clippy::unwrap_used)]

mod commands;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ledger",
    about = "The decision-record substrate: the L0 file format with its CI gate",
    version
)]
struct Cli {
    /// Path to the repo root holding .decisions/ (default: walk up from cwd)
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(commands::run(cli.command, cli.root));
}
