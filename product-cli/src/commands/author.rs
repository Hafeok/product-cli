//! Graph-aware domain (What-capture) authoring sessions.

use clap::Subcommand;
use product_core::author;
use product_core::pf::workflow::Phase;

use super::BoxResult;

#[derive(Subcommand)]
pub enum AuthorCommands {
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
}

pub(crate) fn handle_author(cmd: AuthorCommands) -> BoxResult {
    handle_domain(cmd)
}

/// Handle `product author domain`. Three paths: `--serve` hosts the domain MCP
/// server over stdio (invoked by the agent's MCP config), `--print-prompt`
/// emits the facilitation prompt, otherwise launch the agent session.
fn handle_domain(cmd: AuthorCommands) -> BoxResult {
    let AuthorCommands::Domain { product, seed, cli, print_prompt, serve, session_dir } = cmd;

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

    // Resolve the agent CLI from the flag, otherwise default to claude.
    let cli_str = cli.unwrap_or_else(|| "claude".to_string());
    let agent_cli = author::AgentCli::parse(&cli_str)?;
    let root = std::env::current_dir()?;
    let _ = seed; // the session workspace seeds the draft from canonical `.product`.

    // Generalized (ADR): `author domain` is now a What-capped What→How→Build
    // session. The phase-gated server exposes the What tools; advancing past
    // What is disabled by the `until` cap.
    println!("note: `author domain` now starts a What-capped session (see `product session`).");
    let now = chrono::Utc::now();
    let id = format!("{product}-{}", now.format("%Y%m%d-%H%M%S"));
    author::workflow::scaffold(&root, &id, &product, &cli_str, Phase::What, now.to_rfc3339())?;
    author::workflow::launch(&id, &product, agent_cli, &root)?;
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
