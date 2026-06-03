//! Graph-aware authoring sessions.

use clap::Subcommand;
use product_core::{author, domains};
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum AuthorCommands {
    /// Start an ADR authoring session
    Adr {
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
    },
    /// Start a feature authoring session
    Feature {
        /// Feature ID (optional — enables preflight gate)
        #[arg(long)]
        feature: Option<String>,
        /// Optional comma-separated domains for pattern-suggestion (FT-073).
        /// When supplied with `--print-prompt`, the rendered prompt
        /// includes a "Matching patterns" block. Without an agent
        /// process to interview the author, this is the deterministic
        /// path for testing and scripting.
        #[arg(long, value_delimiter = ',')]
        domains: Vec<String>,
        /// Print the assembled prompt to stdout and exit without launching
        /// the agent. Used by tests and by anyone who wants to feed the
        /// prompt into a different tool (FT-073).
        #[arg(long = "print-prompt")]
        print_prompt: bool,
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
    },
    /// Start a pattern authoring session (FT-073, ADR-050)
    Pattern {
        /// Optional title hint for the pattern being authored
        #[arg(long)]
        title: Option<String>,
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
        AuthorCommands::Pattern { cli, .. } => (author::SessionType::Pattern, cli.clone()),
        AuthorCommands::Review { cli } => (author::SessionType::Review, cli.clone()),
    };

    // FT-073 print-prompt path — render the prompt with optional pattern
    // suggestion block and exit without launching the agent.
    if let AuthorCommands::Feature {
        print_prompt: true,
        ref domains,
        ..
    } = cmd
    {
        let prompt = author::render_feature_prompt(&config, &root, &graph, domains);
        println!("{}", prompt);
        return Ok(());
    }
    if let AuthorCommands::Pattern { .. } = cmd {
        // No print-prompt support for pattern yet — the session itself is
        // small enough that the agent flow is the primary path.
    }

    let cli_str = cli_override.unwrap_or_else(|| config.author.cli.clone());
    let agent_cli = author::AgentCli::parse(&cli_str)?;

    // ADR-026: if authoring a feature, run preflight first
    if let AuthorCommands::Feature { feature: Some(ref fid), .. } = cmd {
        let result = domains::preflight(&graph, fid, &config.domains, &config.features.default_acknowledged_cross_cutting)?;
        if !result.is_clean {
            eprintln!("{}", domains::render_preflight(&result));
            eprintln!("  Resolve preflight gaps before starting author session.");
            process::exit(1);
        }
    }

    author::start_session(session_type, agent_cli, &config, &root)?;
    Ok(())
}
