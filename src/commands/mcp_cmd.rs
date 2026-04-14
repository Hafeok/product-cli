//! MCP server (stdio or HTTP transport).

use product_lib::{config::ProductConfig, error::ProductError, mcp};
use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn handle_mcp(
    http: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    repo: Option<String>,
    write_flag: bool,
) -> BoxResult {
    let repo_root = if let Some(ref path) = repo {
        PathBuf::from(path)
    } else {
        let (_config, root) = ProductConfig::discover()?;
        root
    };

    // --write flag overrides product.toml mcp.write
    let write_enabled = write_flag || {
        let toml_path = repo_root.join("product.toml");
        if toml_path.exists() {
            let cfg = ProductConfig::load(&toml_path)?;
            cfg.mcp.map(|m| m.write).unwrap_or(false)
        } else {
            false
        }
    };

    if http {
        let toml_path = repo_root.join("product.toml");
        let cors_origins = if toml_path.exists() {
            let cfg = ProductConfig::load(&toml_path)?;
            cfg.mcp.map(|m| m.cors_origins).unwrap_or_default()
        } else {
            vec![]
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            ProductError::IoError(format!("Failed to create tokio runtime: {}", e))
        })?;
        rt.block_on(mcp::run_http(
            repo_root,
            write_enabled,
            port,
            bind,
            token,
            cors_origins,
        ))?;
    } else {
        mcp::run_stdio(repo_root, write_enabled)?;
    }

    Ok(())
}
