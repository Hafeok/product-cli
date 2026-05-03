//! Workspace convention checker entry point.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use xtask::{diagnostic, drift, Diagnostic, Format, Registry, Severity};

#[derive(Parser)]
#[command(name = "xtask", about = "Workspace convention enforcement")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run convention checks across the workspace.
    Check {
        /// Run only the named check (e.g. `--only CTX001`). Repeatable.
        #[arg(long)]
        only: Vec<String>,

        /// Output format (`text` or `json`).
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,

        /// Validate that every registered check has a matching convention doc.
        #[arg(long)]
        self_test: bool,

        /// Workspace root. Defaults to the directory containing the `xtask`
        /// crate's parent (i.e. the repo root).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum FormatArg {
    Text,
    Json,
}

impl From<FormatArg> for Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Text => Format::Text,
            FormatArg::Json => Format::Json,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Check { only, format, self_test, root } => {
            let format: Format = format.into();
            let root = root.unwrap_or_else(default_root);
            let registry = Registry::default_set();

            let mut diagnostics: Vec<Diagnostic> = Vec::new();

            if self_test {
                diagnostics.extend(drift::run(&registry, &root));
            } else {
                let filter: Option<Vec<String>> =
                    if only.is_empty() { None } else { Some(only) };
                for check in registry.iter() {
                    if let Some(ids) = filter.as_ref() {
                        if !ids.iter().any(|id| id.eq_ignore_ascii_case(check.id().as_str())) {
                            continue;
                        }
                    }
                    diagnostics.extend(check.run(&root));
                }
            }

            let has_error = diagnostics.iter().any(|d| d.severity == Severity::Error);
            diagnostic::emit(&diagnostics, format);
            if has_error { ExitCode::from(1) } else { ExitCode::SUCCESS }
        }
    }
}

/// Walks up from the xtask manifest directory to find the workspace root.
/// In a single-workspace repo, the parent of `xtask/` is the root.
fn default_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
