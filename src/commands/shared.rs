//! Shared helpers for command handlers — graph loading, write locking.
//!
//! Each public helper comes in two flavours: a typed variant returning
//! `Result<T, ProductError>` (preferred for new migrated handlers) and a
//! boxed variant returning `BoxResult` for legacy handlers still in
//! transition. The boxed variants simply wrap the typed ones.

use product_lib::{config::ProductConfig, error::ProductError, fileops, graph::KnowledgeGraph, parser};
use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn acquire_write_lock() -> Result<fileops::RepoLock, Box<dyn std::error::Error>> {
    acquire_write_lock_typed().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub(crate) fn acquire_write_lock_typed() -> Result<fileops::RepoLock, ProductError> {
    let (_, root) = ProductConfig::discover()?;
    fileops::RepoLock::acquire(&root)
}

pub(crate) fn load_graph(
) -> Result<(ProductConfig, PathBuf, KnowledgeGraph), Box<dyn std::error::Error>> {
    load_graph_typed().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub(crate) fn load_graph_typed(
) -> Result<(ProductConfig, PathBuf, KnowledgeGraph), ProductError> {
    let (config, root) = ProductConfig::discover()?;

    for warning in config.check_schema_version()? {
        eprintln!("{}", warning);
    }

    let features_dir = config.resolve_path(&root, &config.paths.features);
    let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
    let tests_dir = config.resolve_path(&root, &config.paths.tests);
    let deps_dir = config.resolve_path(&root, &config.paths.dependencies);

    let loaded =
        parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;

    for e in &loaded.parse_errors {
        eprintln!("{}", e);
    }

    let graph = KnowledgeGraph::build_with_deps(
        loaded.features,
        loaded.adrs,
        loaded.tests,
        loaded.dependencies,
    )
    .with_parse_errors(loaded.parse_errors);
    Ok((config, root, graph))
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
        let _ = product_lib::request_log::migrate_if_needed(&root, Some(&config.paths.requests));
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
