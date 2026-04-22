//! Graph-aware authoring sessions.

use clap::Subcommand;
use product_lib::{author, domains};
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum AuthorCommands {
    /// Start a feature authoring session
    Feature {
        /// Feature ID (optional — enables preflight gate)
        #[arg(long)]
        feature: Option<String>,
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
    },
    /// Start an ADR authoring session
    Adr {
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
    },
    /// Start a spec review session
    Review {
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
    },
}

pub(crate) fn handle_author(cmd: AuthorCommands) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let (session_type, cli_override) = match &cmd {
        AuthorCommands::Feature { cli, .. } => (author::SessionType::Feature, cli.clone()),
        AuthorCommands::Adr { cli } => (author::SessionType::Adr, cli.clone()),
        AuthorCommands::Review { cli } => (author::SessionType::Review, cli.clone()),
    };

    let cli_str = cli_override.unwrap_or_else(|| config.author.cli.clone());
    let agent_cli = author::AgentCli::parse(&cli_str)?;

    // ADR-026: if authoring a feature, run preflight first
    if let AuthorCommands::Feature { feature: Some(ref fid), .. } = cmd {
        let result = domains::preflight(&graph, fid, &config.domains)?;
        if !result.is_clean {
            eprintln!("{}", domains::render_preflight(&result));
            eprintln!("  Resolve preflight gaps before starting author session.");
            process::exit(1);
        }
    }

    author::start_session(session_type, agent_cli, &config, &root)?;
    Ok(())
}
