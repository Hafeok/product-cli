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
    },
    /// Start an ADR authoring session
    Adr,
    /// Start a spec review session
    Review,
}

pub(crate) fn handle_author(cmd: AuthorCommands) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let session_type = match &cmd {
        AuthorCommands::Feature { .. } => author::SessionType::Feature,
        AuthorCommands::Adr => author::SessionType::Adr,
        AuthorCommands::Review => author::SessionType::Review,
    };

    // ADR-026: if authoring a feature, run preflight first
    if let AuthorCommands::Feature { feature: Some(ref fid) } = cmd {
        let result = domains::preflight(&graph, fid, &config.domains)?;
        if !result.is_clean {
            eprintln!("{}", domains::render_preflight(&result));
            eprintln!("  Resolve preflight gaps before starting author session.");
            process::exit(1);
        }
    }

    author::start_session(session_type, &config, &root)?;
    Ok(())
}
