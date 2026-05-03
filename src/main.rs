//! Product CLI entry point — clap dispatch only.

#![deny(clippy::unwrap_used)]

mod commands;

use clap::{CommandFactory, Parser};
use std::path::PathBuf;
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

    /// Path to the directory containing the .product/ graph. Overrides
    /// PRODUCT_ROOT and walk-up from the current directory. Accepts an
    /// absolute or relative path; ~/ is expanded; symlinks are resolved.
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

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
    if let Err(e) = product_lib::root::early_check() {
        eprintln!("{e}");
        process::exit(1);
    }

    let cli = Cli::parse();
    if let Some(ref root) = cli.root {
        product_lib::root::set_root_flag(root.clone());
    }
    let mut cmd = Cli::command();

    let result = commands::run(cli.command, &cli.format, &mut cmd);
    if let Err(e) = result {
        // FT-058 / E022: route boxed errors through ProductError::exit_code
        // so TcRunnerMissing exits 22 even from a BoxResult handler.
        let exit = e
            .downcast_ref::<product_lib::error::ProductError>()
            .map(|pe| pe.exit_code())
            .unwrap_or(1);
        eprintln!("{e}");
        process::exit(exit);
    }
}

