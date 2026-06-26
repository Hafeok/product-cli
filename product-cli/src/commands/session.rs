//! What → How → Build sessions — start, list, show, resume.
//!
//! `start` scaffolds an isolated session workspace and launches the agent CLI
//! against the phase-gated `product mcp --workflow` server; the agent walks the
//! lifecycle and promotes the draft to the canonical spec on finalize.

use clap::Subcommand;
use product_core::author;
use product_core::pf::workflow::{Phase, WorkflowSession};

use super::BoxResult;

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List the sessions under this repo
    List,
    /// Relaunch the agent against an existing session
    Resume {
        /// The session id
        id: String,
    },
    /// Show one session's phase, history, and build verdict
    Show {
        /// The session id
        id: String,
    },
    /// Start a What→How→Build session: scaffold the workspace and launch the agent
    Start {
        /// The product to author (defaults to the repo's configured name)
        product: Option<String>,
        /// Agent CLI to host the session: claude | copilot
        #[arg(long)]
        cli: Option<String>,
        /// Cap the session at a phase: what | how | build (default: build)
        #[arg(long, default_value = "build")]
        until: String,
        /// Scaffold the session without launching the agent (prints the session id)
        #[arg(long = "no-launch")]
        no_launch: bool,
    },
}

pub(crate) fn handle_session(cmd: SessionCommands) -> BoxResult {
    match cmd {
        SessionCommands::Start { product, cli, until, no_launch } => start(product, cli, &until, no_launch),
        SessionCommands::List => list(),
        SessionCommands::Show { id } => show(&id),
        SessionCommands::Resume { id } => resume(&id),
    }
}

fn start(product: Option<String>, cli: Option<String>, until: &str, no_launch: bool) -> BoxResult {
    let product = resolve_product(product)?;
    let cli_str = cli.unwrap_or_else(|| "claude".to_string());
    let agent_cli = author::AgentCli::parse(&cli_str)?;
    let until = Phase::parse(until)?;
    let root = std::env::current_dir()?;

    let now = chrono::Utc::now();
    let id = format!("{product}-{}", now.format("%Y%m%d-%H%M%S"));
    let session_root = author::workflow::scaffold(&root, &id, &product, &cli_str, until, now.to_rfc3339())?;

    if no_launch {
        println!("Scaffolded session '{}'", id);
        println!("  Session: {}", session_root.display());
        println!("  Run the agent against it with: product mcp --workflow --session {} --repo {}", id, root.display());
        return Ok(());
    }
    author::workflow::launch(&id, &product, agent_cli, &root)?;
    Ok(())
}

fn list() -> BoxResult {
    let root = std::env::current_dir()?;
    let dir = root.join(".product").join("sessions");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        println!("No sessions yet.");
        return Ok(());
    };
    let mut rows: Vec<(String, WorkflowSession)> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let id = e.file_name().to_string_lossy().to_string();
            WorkflowSession::load(&e.path()).ok().map(|s| (id, s))
        })
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    if rows.is_empty() {
        println!("No sessions yet.");
        return Ok(());
    }
    for (id, s) in rows {
        let state = if s.finalized { "finalized" } else { "active" };
        println!("{id}  phase={}  until={}  {state}", s.phase, s.until);
    }
    Ok(())
}

fn show(id: &str) -> BoxResult {
    let root = std::env::current_dir()?;
    let session_root = author::workflow::session_root(&root, id);
    let session = WorkflowSession::load(&session_root)
        .map_err(|_| format!("no session '{id}' under {}", root.display()))?;
    println!("{}", serde_json::to_string_pretty(&session.status_json())?);
    // Surface the live view URL when the session's view is running.
    if let Ok(url) = std::fs::read_to_string(session_root.join(author::workflow::VIEW_URL_FILE)) {
        println!("live view: {}", url.trim());
    }
    Ok(())
}

fn resume(id: &str) -> BoxResult {
    let root = std::env::current_dir()?;
    let session_root = author::workflow::session_root(&root, id);
    let session = WorkflowSession::load(&session_root)
        .map_err(|_| format!("no session '{id}' under {}", root.display()))?;
    let agent_cli = author::AgentCli::parse(&session.agent_cli)?;
    author::workflow::launch(id, &session.product, agent_cli, &root)?;
    Ok(())
}

/// Resolve the product id: an explicit positional wins; else the repo's
/// configured name; else error. Must match the framework id grammar.
fn resolve_product(explicit: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let candidate = explicit
        .or_else(super::shared::default_product_name)
        .ok_or("no product specified and none configured in product.toml")?;
    let product = candidate.trim().to_string();
    product_core::pf::ids::validate_id(&product)?;
    Ok(product)
}
