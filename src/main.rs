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

    let cli = Cli::parse();
    let mut cmd = Cli::command();

    let result = commands::run(cli.command, &cli.format, &mut cmd);
    if let Err(e) = result {
        eprintln!("{e}");
        process::exit(1);
    }
}
