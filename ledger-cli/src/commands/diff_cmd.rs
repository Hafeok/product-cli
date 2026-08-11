//! `ledger diff` — decision-level change between two readings of the store.
//!
//! Semantic, never textual (PRD §3.4): the report says which decisions were
//! added, which tips moved and whether that was a fast-forward, what was
//! superseded or forked, and which acceptances were gained, lost or
//! revoked. `<ref>..<ref>` compares two git revisions; a single `<ref>`
//! compares that revision against the working tree.

use std::path::PathBuf;

use ledger_core::{diff, revision, store};

use super::{resolve_root, EXIT_OK};

pub fn run(root: Option<PathBuf>, spec: &str, json: bool) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let (from_rev, to_rev) = parse_spec(spec)?;
    let from = revision::load_at(&repo_root, &from_rev)?;
    let (to_label, to_store) = match &to_rev {
        Some(rev) => (rev.clone(), revision::load_at(&repo_root, rev)?),
        None => ("worktree".to_string(), store::load(&repo_root)),
    };
    let report = diff::diff(&from_rev, &from, &to_label, &to_store);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
    } else {
        println!("{}", diff::render(&report));
    }
    Ok(EXIT_OK)
}

/// `a..b` compares two revisions; `a` (or `a..`) compares against the
/// working tree.
fn parse_spec(spec: &str) -> Result<(String, Option<String>), String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Err("an empty diff spec — expected `<ref>..<ref>` or `<ref>`".to_string());
    }
    match trimmed.split_once("..") {
        None => Ok((trimmed.to_string(), None)),
        Some((from, "")) => Ok((from.to_string(), None)),
        Some(("", _)) => Err("the spec names no base — expected `<ref>..<ref>`".to_string()),
        Some((from, to)) => Ok((from.to_string(), Some(to.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_spec;

    #[test]
    fn the_spec_parses_both_forms() {
        assert_eq!(parse_spec("a..b").expect("two"), ("a".into(), Some("b".into())));
        assert_eq!(parse_spec("a").expect("one"), ("a".into(), None));
        assert_eq!(parse_spec("a..").expect("open"), ("a".into(), None));
        assert!(parse_spec("..b").is_err());
        assert!(parse_spec("  ").is_err());
    }
}
