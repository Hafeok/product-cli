//! Adapters for the signature verbs: accept, revoke.

use std::path::PathBuf;

use ledger_core::author::{AcceptArgs, RevokeArgs};

use super::common::{finish, open_author, parse_date};

pub fn accept(root: Option<PathBuf>, decision: &str, expires: Option<&str>) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args = AcceptArgs {
        decision: decision.parse()?,
        expires_at: expires.map(parse_date).transpose()?,
    };
    finish(author.accept(args))
}

pub fn revoke(root: Option<PathBuf>, acceptance: &str, reason: String) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args = RevokeArgs { acceptance: acceptance.parse()?, reason };
    finish(author.revoke(args))
}
