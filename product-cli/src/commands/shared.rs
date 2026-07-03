//! Shared helpers for command handlers — graph loading, write locking.
//!
//! Each public helper comes in two flavours: a typed variant returning
//! `Result<T, ProductError>` (preferred for new migrated handlers) and a
//! boxed variant returning `BoxResult` for legacy handlers still in
//! transition. The boxed variants simply wrap the typed ones.

use product_core::{config::ProductConfig, error::ProductError, fileops};
use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn acquire_write_lock() -> Result<fileops::RepoLock, Box<dyn std::error::Error>> {
    acquire_write_lock_typed().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub(crate) fn acquire_write_lock_typed() -> Result<fileops::RepoLock, ProductError> {
    let (_, root) = ProductConfig::discover()?;
    fileops::RepoLock::acquire(&root)
}

/// The repo's configured product name, if a product.toml is discoverable and
/// carries a non-empty `name`. Used to default the `<product>` argument of the
/// domain-graph commands in single-product repos.
pub(crate) fn default_product_name() -> Option<String> {
    let (config, _) = ProductConfig::discover().ok()?;
    let name = config.name.trim().to_string();
    (!name.is_empty()).then_some(name)
}

/// The configured default agent CLI for authoring sessions: the repo's
/// `[author].cli` if discoverable, otherwise the global user config's. Returns
/// `None` when neither is set, leaving the caller to fall back to the built-in
/// `claude` default. The `--cli` flag takes precedence over this.
pub(crate) fn default_author_cli() -> Option<String> {
    ProductConfig::discover()
        .ok()
        .and_then(|(config, _)| config.author_cli())
        .or_else(product_core::config::load_global_author_cli)
}

/// The on-disk directory for a product's `<kind>` artifacts (features,
/// deliverables, deciders, work-units, …), scoped per-product: the root product
/// uses `.product/<kind>`, every other product `.product/products/<name>/<kind>`.
/// `product` is the command's `--product` (else the configured default).
pub(crate) fn artifact_dir(product: Option<&str>, kind: &str) -> PathBuf {
    let root = domain_root();
    let name = product.map(str::to_string).or_else(default_product_name).unwrap_or_default();
    product_core::pf::paths::product_base(&root, &name).join(kind)
}

/// Resolve the repo root for domain-graph commands: the discovered product
/// root, else the current directory.
pub(crate) fn domain_root() -> PathBuf {
    ProductConfig::discover()
        .map(|(_, root)| root)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
}

/// Process-startup hooks that run before every command: stale tmp-file cleanup
/// over the `.product/` framework tree (ADR-015).
pub(crate) fn run_startup_hooks() -> BoxResult {
    cleanup_stale_tmp_files();
    Ok(())
}

fn cleanup_stale_tmp_files() {
    if let Ok((_, root)) = ProductConfig::discover() {
        fileops::cleanup_tmp_files(&root.join(".product"));
    }
}
