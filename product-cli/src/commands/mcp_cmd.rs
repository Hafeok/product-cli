//! MCP server (stdio or HTTP transport).

use product_core::config::ProductConfig;
use product_mcp::{run_stdio, serve_http_blocking};
use std::path::PathBuf;

use super::BoxResult;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_mcp(
    http: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    repo: Option<String>,
    write_flag: bool,
    workflow: bool,
    session: Option<String>,
) -> BoxResult {
    // Resolve the repo root + config in one shot. When `--repo` is supplied,
    // walk the canonical → alias → legacy discovery order against that root
    // (FT-057 / ADR-048). Otherwise reuse the config that `discover()`
    // already loaded — no need to re-read the same file twice.
    let (config, repo_root) = if let Some(ref path) = repo {
        let root = PathBuf::from(path);
        let cfg = ProductConfig::load_from_root(&root)?;
        (cfg, root)
    } else {
        ProductConfig::discover()?
    };

    let mcp_cfg = config.mcp.as_ref();

    // --write flag overrides mcp.write from the config file.
    let write_enabled = write_flag || mcp_cfg.map(|m| m.write).unwrap_or(false);

    // `--session` implies workflow mode (one phase-gated session per process).
    let workflow = workflow || session.is_some();

    if http {
        let cors_origins = mcp_cfg
            .map(|m| m.cors_origins.clone())
            .unwrap_or_default();
        serve_http_blocking(repo_root, write_enabled, port, bind, token, cors_origins, workflow, session)?;
    } else {
        run_stdio(repo_root, write_enabled, session)?;
    }

    Ok(())
}
