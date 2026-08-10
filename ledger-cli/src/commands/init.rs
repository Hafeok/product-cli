//! `ledger init` — scaffold the store layout.

use std::path::PathBuf;

use ledger_core::init::{apply_init, plan_init, render_init};

use super::EXIT_OK;

pub fn run(root: Option<PathBuf>) -> Result<i32, String> {
    // Not `resolve_root`: init is what creates the store, so requiring one
    // to already exist would be circular.
    let repo_root = match root {
        Some(explicit) => explicit,
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };
    let plan = plan_init(&repo_root);
    apply_init(&repo_root, &plan).map_err(|e| e.to_string())?;
    println!("{}", render_init(&plan));
    Ok(EXIT_OK)
}
