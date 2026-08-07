//! Subcommand surface for the `ddd` binary.

mod diff;
mod init;
mod render;
mod report;
mod serve;
mod validate;
mod what;
mod why;

use std::path::PathBuf;

use clap::Subcommand;
use product_core::error::{ProductError, Result};

/// The M1-M4 command surface (PRD §7). Keep the variant list sorted.
#[derive(Subcommand)]
pub enum Commands {
    /// Declared vs. detected rules: UNGOVERNED / STALE / UNCITED_SUPPRESSION
    Diff {
        /// SARIF file(s) with emitted diagnostics (adds to config detect.sarif)
        #[arg(long, value_name = "FILE")]
        sarif: Vec<PathBuf>,
    },
    /// Scaffold the .ddd/ store: the §6 layout plus config.yaml
    Init,
    /// Project the graph to one static, self-contained HTML file
    Render {
        /// Output file (default: .ddd/render.html)
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
        /// SARIF file(s) with emitted diagnostics (adds to config detect.sarif)
        #[arg(long, value_name = "FILE")]
        sarif: Vec<PathBuf>,
        /// Judge cadence against this date instead of today (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Escape report: diff findings, cadence violations, basis loss
    Report {
        #[command(subcommand)]
        what: ReportWhat,
    },
    /// Start the ddd_* MCP server over stdio (language tools + interceptor)
    Serve,
    /// Schema + ontology validation of the graph (exit 1 on violations)
    Validate,
    /// Pre-load the LSP hosts (Roslyn solution load) so tools answer warm
    Warmup {
        /// Limit to one language (repeatable); default warms every adapter
        #[arg(long, value_name = "LANG")]
        language: Vec<String>,
        /// Give up after this many seconds per host
        #[arg(long, default_value = "120")]
        timeout: u64,
    },
    /// What-graph boundaries (§3) carrying no governing declaration
    What {
        /// Limit to one product; default reports every product under .product/
        #[arg(long, value_name = "NAME")]
        product: Option<String>,
        /// Also list the surface elements that are already declared
        #[arg(long)]
        all: bool,
        /// Exit non-zero when a boundary carries no declaration (CI gate)
        #[arg(long)]
        strict: bool,
    },
    /// Resolve an id to decision -> rationale -> principal -> basedOn claims
    Why {
        /// A decision, claim, diagnostic (rule), pattern, or seam id
        id: String,
        /// SARIF file(s) with emitted diagnostics (adds to config detect.sarif)
        #[arg(long, value_name = "FILE")]
        sarif: Vec<PathBuf>,
    },
}

/// What `ddd report` reports on (M2 ships `escapes`).
#[derive(Subcommand)]
pub enum ReportWhat {
    /// Detected-but-undeclared items, claims past cadence, basis loss
    Escapes {
        /// SARIF file(s) with emitted diagnostics (adds to config detect.sarif)
        #[arg(long, value_name = "FILE")]
        sarif: Vec<PathBuf>,
        /// Judge cadence against this date instead of today (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
}

pub fn run(cmd: Commands, root: Option<PathBuf>) -> Result<()> {
    match cmd {
        Commands::Diff { sarif } => diff::run(root, sarif),
        Commands::Init => init::run(root),
        Commands::Render { out, sarif, today } => render::run(root, out, sarif, today),
        Commands::Report { what } => match what {
            ReportWhat::Escapes { sarif, today } => report::run(root, sarif, today),
        },
        Commands::Serve => serve::run(root),
        Commands::Validate => validate::run(root),
        Commands::Warmup { language, timeout } => serve::warmup(root, language, timeout),
        Commands::What { product, all, strict } => what::run(root, product, all, strict),
        Commands::Why { id, sarif } => why::run(root, &id, sarif),
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
