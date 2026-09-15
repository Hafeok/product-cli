//! Specification-flow CLI entry point — clap dispatch only.

#![deny(clippy::unwrap_used)]

mod commands;
mod exit;
mod render;
mod verbs;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "spec",
    about = "The specification flow's non-delegable half: open, close, and gate act-time records",
    version
)]
struct Cli {
    /// Repo root holding `.spec/` (default: the current directory)
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or_else(|| PathBuf::from("."));
    std::process::exit(commands::run(cli.command, &root));
}
