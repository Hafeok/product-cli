//! Adapters for the read-only projections: status, log, blame, show.

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

/// What a `show` covers and how it prints.
pub struct ShowFlags {
    pub decision: Option<String>,
    pub set: Option<String>,
    pub group: Option<String>,
    pub json: bool,
    pub today: Option<String>,
}

/// Decisions on screen — the read a signature is given before it.
///
/// A named id that has no filed version is an error, not an empty screen:
/// a blank read is the one outcome an acceptance pass must never mistake
/// for "nothing to sign". A selector that matches nothing says so plainly.
pub fn show(root: Option<PathBuf>, flags: ShowFlags) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let today = match flags.today.as_deref() {
        Some(raw) => parse_date(raw)?,
        None => Utc::now().date_naive(),
    };
    let selector = selector(&flags)?;
    let store = store::load(&repo_root);
    let texts = super::basis_text::DddBasisText::load(&repo_root);
    let screens = ledger_core::show::screens(&store, &selector, today, &texts);
    if let (Some(id), true) = (&flags.decision, screens.is_empty()) {
        return Err(format!("{id} has no filed version in this store"));
    }
    if flags.json {
        println!("{}", serde_json::to_string_pretty(&screens).map_err(|e| e.to_string())?);
    } else {
        print!("{}", ledger_core::show::render_all(&screens));
    }
    Ok(EXIT_OK)
}

/// Exactly one selector, so a pass never silently covers more than the
/// caller asked for. No selector at all reads the whole store.
fn selector(flags: &ShowFlags) -> Result<ledger_core::show::Selector, String> {
    use ledger_core::show::{Group, Selector};
    match (&flags.decision, &flags.set, &flags.group) {
        (Some(id), None, None) => Ok(Selector::One(id.parse()?)),
        (None, Some(set), None) => Ok(Selector::Set(set.clone())),
        (None, None, Some(group)) => Ok(Selector::Group(Group::parse(group)?)),
        (None, None, None) => Ok(Selector::All),
        _ => Err("name a decision, or --set, or --group — not more than one".to_string()),
    }
}

pub fn blame(root: Option<PathBuf>, decision: &str) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    println!("{}", author::blame(&store::load(&repo_root), &decision.parse()?));
    Ok(EXIT_OK)
}
