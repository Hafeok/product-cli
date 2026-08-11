//! `ledger add` — the decision-creation adapter.

use std::path::PathBuf;

use ledger_core::author::AddArgs;

use super::common::{allocation_args, finish, open_author, parse_based_on, parse_tier};

pub struct Flags {
    pub set: String,
    pub statement: String,
    pub namespace: Option<String>,
    pub store: Option<String>,
    pub discharge: Vec<String>,
    pub stage: Option<String>,
    pub expectation: Option<String>,
    pub actor: Option<String>,
    pub tolerance_override: Option<String>,
    pub based_on: Vec<String>,
}

pub fn run(root: Option<PathBuf>, flags: Flags) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args = AddArgs {
        set: flags.set,
        statement: flags.statement,
        namespace: flags.namespace,
        allocation: allocation_args(
            flags.store.as_deref(),
            &flags.discharge,
            flags.stage.as_deref(),
            flags.expectation,
            flags.actor.as_deref(),
        )?,
        tolerance_override: flags.tolerance_override.as_deref().map(parse_tier).transpose()?,
        based_on: parse_based_on(&flags.based_on)?,
    };
    finish(author.add(args))
}
