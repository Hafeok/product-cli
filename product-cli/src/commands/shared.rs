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

/// Resolve the repo root for domain-graph commands: the discovered product
/// root, else the current directory.
pub(crate) fn domain_root() -> PathBuf {
    ProductConfig::discover()
        .map(|(_, root)| root)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
}

/// Process-startup hooks that run before every command: one-shot log-path
/// migration (FT-042) and stale tmp-file cleanup (ADR-015).
pub(crate) fn run_startup_hooks() -> BoxResult {
    cleanup_stale_tmp_files();
    migrate_log_path_if_needed();
    Ok(())
}

fn migrate_log_path_if_needed() {
    if let Ok((config, root)) = ProductConfig::discover() {
        let _ = product_core::request_log::migrate_if_needed(&root, Some(&config.paths.requests));
    }
}

fn cleanup_stale_tmp_files() {
    if let Ok((config, root)) = ProductConfig::discover() {
        let dirs = [
            config.resolve_path(&root, &config.paths.features),
            config.resolve_path(&root, &config.paths.adrs),
            config.resolve_path(&root, &config.paths.tests),
            config.resolve_path(&root, &config.paths.dependencies),
        ];
        for dir in &dirs {
            fileops::cleanup_tmp_files(dir);
        }
    }
}
