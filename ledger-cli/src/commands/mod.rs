//! Subcommand surface for the `ledger` binary.
//!
//! L0 is the format plus the gate, so the surface is deliberately two
//! commands. The authoring verbs — `add`, `allocate`, `escape`, `accept`,
//! `revoke`, `status`, `log`, `blame`, `diff` — are L1, and every one of
//! them will call the same `ledger-core` validators this gate calls rather
//! than growing a second copy of the rules.

mod init;
mod verify;

use std::path::PathBuf;

use clap::Subcommand;

/// Exit codes, CI-conventional: a gate that says no is a different outcome
/// from a gate that could not run, and CI has to be able to tell them apart.
pub const EXIT_OK: i32 = 0;
pub const EXIT_VIOLATIONS: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

/// The L0 command surface. Keep the variant list sorted.
#[derive(Subcommand)]
pub enum Commands {
    /// Scaffold the .decisions/ store: the §5 layout plus the ignore line
    Init,
    /// Run the gate over the log (exit 1 on findings, 2 on failure to run)
    Verify {
        /// Limit to one gate: `readiness` blocks produce, `completeness`
        /// blocks release. Default runs every class.
        #[arg(long, value_name = "GATE")]
        gate: Option<String>,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
        /// Judge acceptance expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
        /// Skip the git blame pass (class L009)
        #[arg(long)]
        no_blame: bool,
    },
}

/// Dispatch, mapping every outcome to an exit code.
pub fn run(command: Commands, root: Option<PathBuf>) -> i32 {
    let result = match command {
        Commands::Init => init::run(root),
        Commands::Verify { gate, json, today, no_blame } => {
            verify::run(root, verify::Args { gate, json, today, blame: !no_blame })
        }
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            EXIT_ERROR
        }
    }
}

/// Resolve the repo root: the flag, else the first ancestor with a store.
pub fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(explicit) = root {
        return Ok(explicit);
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    ledger_core::store::find_root(&cwd).ok_or_else(|| {
        format!(
            "no {}/ found here or in any parent — run `ledger init` to scaffold one",
            ledger_core::STORE_DIR
        )
    })
}
