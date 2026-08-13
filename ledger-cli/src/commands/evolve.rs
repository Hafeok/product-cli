//! Adapters for the next-version verbs: allocate, escape, revise, supersede.

use std::path::PathBuf;

use ledger_core::author::{EscapeArgs, ReviseArgs, SupersedeArgs};
use ledger_core::id::DecisionId;

use super::common::{allocation_args, finish, open_author, parse_based_on, parse_date};

fn decision(raw: &str) -> Result<DecisionId, String> {
    raw.parse()
}

pub fn allocate(
    root: Option<PathBuf>,
    id: &str,
    store: &str,
    discharge: &[String],
    stage: Option<&str>,
    expectation: Option<String>,
    actor: Option<&str>,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let alloc = allocation_args(Some(store), discharge, stage, expectation, actor)?;
    finish(author.allocate(&decision(id)?, alloc))
}

pub fn escape(
    root: Option<PathBuf>,
    id: &str,
    exposure: String,
    review_by: &str,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args = EscapeArgs { exposure, review_by: parse_date(review_by)? };
    finish(author.escape(&decision(id)?, args))
}

/// What a revision states about its reopen edges: the flags as given.
pub struct ReviseEdges<'a> {
    pub based_on: &'a [String],
    pub revisit_if: &'a [String],
    /// Drop every inherited reopen edge — the decision has been reopened
    /// and re-decided, so the edge that reopened it is spent.
    pub no_revisit_if: bool,
    /// What this act was, recorded on the change-set.
    pub note: Option<String>,
}

pub fn revise(
    root: Option<PathBuf>,
    id: &str,
    statement: String,
    edges: ReviseEdges<'_>,
    parent: Option<&str>,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let revisit_if = match (edges.no_revisit_if, edges.revisit_if.is_empty()) {
        (true, false) => {
            return Err("--no-revisit-if and --revisit-if state opposite things".to_string())
        }
        (true, true) => Some(Vec::new()),
        (false, true) => None,
        (false, false) => Some(super::common::parse_revisit_if(edges.revisit_if)?),
    };
    let args = ReviseArgs {
        statement,
        based_on: parse_based_on(edges.based_on)?,
        revisit_if,
        note: edges.note.clone(),
        expected_parent: parent.map(str::parse).transpose()?,
    };
    finish(author.revise(&decision(id)?, args))
}

pub fn supersede(
    root: Option<PathBuf>,
    superseded: &str,
    by: &str,
    reason: Option<String>,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    let args =
        SupersedeArgs { superseded: decision(superseded)?, by: decision(by)?, reason };
    finish(author.supersede(args))
}
