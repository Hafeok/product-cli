//! `ddd serve` plus `ddd warmup` — the MCP surface adapters (PRD §7).

use std::path::PathBuf;
use std::time::Duration;

use product_core::error::Result;

/// Serve the `ddd_*` MCP tools over stdio until the client hangs up.
pub fn run(root: Option<PathBuf>) -> Result<()> {
    let root = super::resolve_root(root)?;
    eprintln!("ddd MCP server (stdio) — root {}", root.display());
    ddd_mcp::serve_stdio(root)
}

/// Pre-load the LSP hosts; prints per-host readiness with timing.
pub fn warmup(root: Option<PathBuf>, languages: Vec<String>, timeout_secs: u64) -> Result<()> {
    let root = super::resolve_root(root)?;
    let mut manager = ddd_lsp::HostManager::new(root);
    let selection = (!languages.is_empty()).then_some(languages);
    let mut failures = 0usize;
    for (language, first) in manager.warm_all(selection.as_deref()) {
        match first {
            Err(e) => {
                failures += 1;
                println!("{language}: failed to start — {e}");
            }
            Ok(_) => match manager
                .host(&language)
                .and_then(|h| h.wait_ready(Duration::from_secs(timeout_secs)))
            {
                Ok(ddd_lsp::host::Readiness::Ready { elapsed_ms }) => {
                    println!("{language}: ready in {elapsed_ms}ms");
                }
                Ok(other) => {
                    failures += 1;
                    println!("{language}: not ready within {timeout_secs}s — {other:?}");
                }
                Err(e) => {
                    failures += 1;
                    println!("{language}: {e}");
                }
            },
        }
        if let Ok(host) = manager.host(&language) {
            for note in host.notes() {
                println!("  note: {note}");
            }
        }
    }
    if failures > 0 {
        return Err(product_core::error::ProductError::ConfigError(format!(
            "{failures} host(s) failed to warm up"
        )));
    }
    Ok(())
}
