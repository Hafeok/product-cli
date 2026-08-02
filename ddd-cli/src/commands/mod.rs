//! Subcommand surface for the `ddd` binary.

mod init;
mod validate;
mod why;

use std::path::PathBuf;

use clap::Subcommand;
use product_core::error::{ProductError, Result};

/// The M1 command surface (PRD §7). Keep the variant list sorted.
#[derive(Subcommand)]
pub enum Commands {
    /// Scaffold the .ddd/ store: the §6 layout plus config.yaml
    Init,
    /// Schema + ontology validation of the graph (exit 1 on violations)
    Validate,
    /// Resolve an id to decision -> rationale -> principal -> basedOn claims
    Why {
        /// A decision, claim, diagnostic (rule), pattern, or seam id
        id: String,
    },
}

pub fn run(cmd: Commands, root: Option<PathBuf>) -> Result<()> {
    match cmd {
        Commands::Init => init::run(root),
        Commands::Validate => validate::run(root),
        Commands::Why { id } => why::run(root, &id),
    }
}

/// Resolve the repo root for read commands: `--root` verbatim, else walk up
/// from the current directory to the nearest `.ddd/`.
fn resolve_root(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = flag {
        if root.join(ddd_core::store::STORE_DIR).is_dir() {
            return Ok(root);
        }
        return Err(ProductError::NotFound(format!("no .ddd/ store under {}", root.display())));
    }
    let cwd = std::env::current_dir().map_err(|e| ProductError::IoError(e.to_string()))?;
    ddd_core::store::find_root(&cwd)
        .ok_or_else(|| ProductError::NotFound(".ddd/ store not found — run `ddd init` first".to_string()))
}
