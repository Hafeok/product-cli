//! Adapters for the L2 graph commands: coverage, reindex.

use std::path::PathBuf;

use chrono::Utc;
use ledger_core::coverage;
use ledger_core::graph::index;
use ledger_core::store;

use super::common::parse_date;
use super::{resolve_root, EXIT_OK};

pub fn coverage(
    root: Option<PathBuf>,
    set: Option<&str>,
    json: bool,
    today: Option<&str>,
) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let today = match today {
        Some(raw) => parse_date(raw)?,
        None => Utc::now().date_naive(),
    };
    let report = coverage::coverage(&store::load(&repo_root), today, set);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
    } else {
        println!("{}", coverage::render(&report));
    }
    Ok(EXIT_OK)
}

pub fn reindex(root: Option<PathBuf>) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let outcome = index::reindex(&repo_root)?;
    println!(
        "wrote {} — {} triple(s), {} byte(s); derived from the log, disposable by proof",
        outcome.path.display(),
        outcome.triples,
        outcome.bytes
    );
    Ok(EXIT_OK)
}
