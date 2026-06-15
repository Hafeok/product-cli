//! Graph-aware authoring sessions.

use clap::Subcommand;
use product_core::config::ProductConfig;
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
    /// Start a domain (What-capture) authoring session
    Domain {
        /// The product whose domain (What) is being captured. Defaults to the
        /// repo's configured product name; prompts if none is configured.
        product: Option<String>,
        /// Seed the session from a prior session's Turtle export
        #[arg(long)]
        seed: Option<std::path::PathBuf>,
        /// Agent CLI to host the session: claude | copilot
        /// (overrides `[author].cli` in product.toml)
        #[arg(long)]
        cli: Option<String>,
        /// Print the facilitation prompt and exit (no agent launch)
        #[arg(long = "print-prompt")]
        print_prompt: bool,
        /// Internal: host the domain MCP server over stdio for the agent
        #[arg(long, hide = true)]
        serve: bool,
        /// Internal: session directory used by `--serve`
        #[arg(long, hide = true)]
        session_dir: Option<std::path::PathBuf>,
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
    // Domain (What-capture) is a separate graph and a separate MCP server, so
    // it does not load the FT/ADR/TC knowledge graph. Handle it up front.
    if let AuthorCommands::Domain { .. } = cmd {
        return handle_domain(cmd);
    }

    let (config, root, graph) = load_graph()?;
    let (session_type, cli_override) = match &cmd {
        AuthorCommands::Feature { cli, .. } => (author::SessionType::Feature, cli.clone()),
        AuthorCommands::Adr { cli } => (author::SessionType::Adr, cli.clone()),
        AuthorCommands::Domain { .. } => unreachable!("Domain handled above"),
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

/// Handle `product author domain`. Three paths: `--serve` hosts the domain MCP
/// server over stdio (invoked by the agent's MCP config), `--print-prompt`
/// emits the facilitation prompt, otherwise launch the agent session.
fn handle_domain(cmd: AuthorCommands) -> BoxResult {
    let AuthorCommands::Domain { product, seed, cli, print_prompt, serve, session_dir } = cmd else {
        unreachable!("handle_domain called with non-Domain variant")
    };

    if serve {
        // `--serve` is driven by the agent over stdin (JSON-RPC), so never
        // prompt here — resolve from the flag or config, else error.
        let product = resolve_product(product, false)?;
        let root = std::env::current_dir()?;
        let dir = session_dir.unwrap_or_else(|| author::domain::session_dir(&root, &product));
        std::fs::create_dir_all(&dir)?;
        if let Some(seed_path) = &seed {
            seed_session(&dir, &product, seed_path)?;
        }
        product_mcp::run_domain_stdio(dir)?;
        return Ok(());
    }

    // Interactive paths may prompt for a product name when none is configured.
    let product = resolve_product(product, true)?;

    if print_prompt {
        println!("{}", author::domain::render_prompt(&product));
        return Ok(());
    }

    // Resolve the agent CLI from the flag or, if a product.toml is reachable,
    // its `[author].cli`; otherwise default to claude.
    let cli_str = cli
        .or_else(|| ProductConfig::discover().ok().map(|(c, _)| c.author.cli))
        .unwrap_or_else(|| "claude".to_string());
    let agent_cli = author::AgentCli::parse(&cli_str)?;
    let root = std::env::current_dir()?;
    author::domain::start_session(&product, agent_cli, seed.as_deref(), &root)?;
    Ok(())
}

/// Resolve the product id for a domain session: an explicit positional wins;
/// otherwise default to the repo's configured product name; otherwise (only
/// when `interactive`) prompt on stdin. The resolved id must match the
/// Product-Framework id grammar (it keys the graph namespace).
fn resolve_product(explicit: Option<String>, interactive: bool) -> Result<String, Box<dyn std::error::Error>> {
    let candidate = explicit
        .or_else(super::shared::default_product_name)
        .map(Ok)
        .unwrap_or_else(|| {
            if interactive {
                prompt_for_product_name()
            } else {
                Err("no product specified and none configured in product.toml".into())
            }
        })?;
    let product = candidate.trim().to_string();
    product_core::pf::ids::validate_id(&product).map_err(|e| {
        format!(
            "{e}\n  = the domain session keys its graph by this id; pass a valid \
             name, e.g. `product author domain my-product`"
        )
    })?;
    Ok(product)
}

/// Prompt on stdin for a product name (no product configured).
fn prompt_for_product_name() -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;
    eprint!("No product configured. Name the product whose domain you are capturing: ");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    let n = std::io::stdin().read_line(&mut line)?;
    let name = line.trim().to_string();
    if n == 0 || name.is_empty() {
        return Err("a product name is required to start a domain authoring session".into());
    }
    Ok(name)
}

/// Pre-seed the active session file from a prior Turtle export so the served
/// server starts from it (used when `--serve --seed` is passed together).
fn seed_session(dir: &std::path::Path, product: &str, seed_path: &std::path::Path) -> BoxResult {
    use product_core::pf::session::DomainSession;
    if DomainSession::load(dir).is_ok() {
        return Ok(()); // a session already exists; don't clobber it
    }
    let turtle = std::fs::read_to_string(seed_path)?;
    let now = chrono::Utc::now().to_rfc3339();
    let session = DomainSession::start(product, None, vec![], Some(&turtle), now)?;
    session.save(dir)?;
    Ok(())
}
