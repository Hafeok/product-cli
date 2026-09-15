//! Subcommand surface, mirroring the flow's verbs.
//!
//! Only the non-delegable half lives here. `import`, `candidates`, `map` and
//! the drafting side of `implement` are the agent host's, reached over MCP;
//! this binary is the half a model may not call, which is why the process
//! boundary is the accountability boundary.

use std::path::Path;

use clap::Subcommand;

use crate::{exit, render, verbs};

#[derive(Subcommand)]
pub enum Commands {
    /// Open an act-time record for a slice built against the specification.
    Implement(verbs::ImplementArgs),
    /// Close an act-time record. Names a principal; a machine cannot be one.
    Close(verbs::CloseArgs),
    /// List act-time records.
    Records(verbs::RecordsArgs),
    /// The CI gate: judge the record store against the closed class set.
    Check(verbs::CheckArgs),
}

/// Run one subcommand, returning its process exit code.
pub fn run(command: Commands, root: &Path) -> i32 {
    let outcome = match command {
        Commands::Implement(args) => verbs::implement(root, &args),
        Commands::Close(args) => verbs::close(root, &args),
        Commands::Records(args) => verbs::records(root, &args),
        Commands::Check(args) => verbs::check(root, &args),
    };
    match outcome {
        Ok(report) => {
            render::emit(&report);
            report.code
        }
        Err(e) => {
            eprintln!("{e}");
            exit::COULD_NOT_RUN
        }
    }
}
