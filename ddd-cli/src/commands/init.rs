//! `ddd init` — scaffold the store, reporting created vs. skipped files.

use std::path::PathBuf;

use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>) -> Result<()> {
    let repo_root = match root {
        Some(r) => r,
        None => std::env::current_dir().map_err(|e| ProductError::IoError(e.to_string()))?,
    };
    let plan = ddd_core::init::plan_init(&repo_root);
    let created = ddd_core::init::apply_init(&plan)?;
    if created == 0 {
        println!(".ddd/ already scaffolded at {} — nothing to create", plan.ddd_dir.display());
    } else {
        println!(
            "initialised {} — {} file(s) created, {} already present",
            plan.ddd_dir.display(),
            created,
            plan.skipped.len()
        );
    }
    Ok(())
}
