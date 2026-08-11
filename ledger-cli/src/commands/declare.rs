//! `ledger declare` — the set-declaration adapter.

use std::path::PathBuf;

use ledger_core::author::DeclareArgs;

use super::common::{finish, open_author, parse_ground, parse_tier};

pub struct Flags {
    pub set: String,
    pub title: Option<String>,
    pub tolerance_floor: String,
    pub ground: String,
    pub owner: Option<String>,
    pub notes: Option<String>,
}

pub fn run(root: Option<PathBuf>, flags: Flags) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args = DeclareArgs {
        id: flags.set.clone(),
        title: flags.title.unwrap_or_else(|| flags.set.clone()),
        tolerance_floor: parse_tier(&flags.tolerance_floor)?,
        ground: parse_ground(&flags.ground)?,
        owner: flags.owner.as_deref().map(str::parse).transpose()?,
        notes: flags.notes,
    };
    finish(author.declare(args))
}
