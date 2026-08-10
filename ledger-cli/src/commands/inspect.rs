//! Adapters for the read-only projections: status, log, blame.

use std::path::PathBuf;

use chrono::Utc;
use ledger_core::author;
use ledger_core::store;

use super::common::parse_date;
use super::{resolve_root, EXIT_OK};

pub fn status(root: Option<PathBuf>, today: Option<&str>) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let today = match today {
        Some(raw) => parse_date(raw)?,
        None => Utc::now().date_naive(),
    };
    println!("{}", author::status(&store::load(&repo_root), today));
    Ok(EXIT_OK)
}

pub fn log(root: Option<PathBuf>, set: Option<&str>) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    println!("{}", author::log(&store::load(&repo_root), set));
    Ok(EXIT_OK)
}

pub fn blame(root: Option<PathBuf>, decision: &str) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    println!("{}", author::blame(&store::load(&repo_root), &decision.parse()?));
    Ok(EXIT_OK)
}
