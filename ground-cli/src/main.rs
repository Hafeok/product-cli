//! `ground` — code extraction into a ground-registry instance.
//!
//! One verb: `extract`. It reads a codebase through the declaration-level
//! LSP subset, proposes triples with their Readings, gates them against the
//! instance's own shapes, and writes one file per assertion for per-triple
//! review. Ratification is the reviewer's merge; this binary never merges,
//! never pushes, and never writes outside the instance directory.

mod extract;
mod read;
mod symbols;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ground", about = "Ground-registry code extraction", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read a codebase and propose triples into a registry instance
    Extract {
        /// The corpus's abstracted name, used in every provenance link
        #[arg(long)]
        corpus: String,
        /// The commit the read is pinned to
        #[arg(long = "ref")]
        git_ref: String,
        /// Repository root, so citations are repo-relative
        #[arg(long)]
        repo: PathBuf,
        /// Directory to read; repeatable. Defaults to the repository root
        #[arg(long = "source")]
        sources: Vec<PathBuf>,
        /// Directory the language host is rooted at (a solution directory)
        #[arg(long)]
        workspace: PathBuf,
        /// The language-host command line
        #[arg(long = "host", num_args = 1.., value_delimiter = ' ')]
        host: Vec<String>,
        /// The registry instance to propose into
        #[arg(long)]
        instance: PathBuf,
        /// Graph directory inside the instance
        #[arg(long = "graph", default_value = "graphs/canonical")]
        graph_dir: String,
        /// RFC-3339 instant recorded as every Reading's as-of
        #[arg(long = "as-of")]
        as_of: String,
        /// Instrument string recorded in every Reading's assurance
        #[arg(long)]
        instrument: String,
        /// Plan and report without writing anything
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Extract {
            corpus,
            git_ref,
            repo,
            sources,
            workspace,
            host,
            instance,
            graph_dir,
            as_of,
            instrument,
            dry_run,
        } => {
            let sources = if sources.is_empty() { vec![repo.clone()] } else { sources };
            let args = extract::ExtractArgs {
                corpus,
                git_ref,
                repo_root: repo,
                sources,
                workspace,
                host_command: host,
                instance,
                graph_dir,
                as_of,
                instrument,
                dry_run,
            };
            match extract::extract(&args) {
                Ok(outcome) => {
                    print!("{}", extract::render(&outcome));
                    if outcome.plan.conformant() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("ground extract: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
