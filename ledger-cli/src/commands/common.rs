//! Shared plumbing for the authoring subcommands.
//!
//! Flag values parse through `ledger-core`'s own types (tiers, identities,
//! discharge refs), so the CLI cannot accept what the store would refuse.
//! Outcome mapping follows the gate's exit discipline: a refusal or a
//! conflict is the tool saying no (exit 1); bad input or a broken
//! environment is the tool unable to run (exit 2).

use std::path::PathBuf;

use chrono::NaiveDate;
use ledger_core::allocation::AllocationKind;
use ledger_core::author::{AllocationArgs, Applied, Author, AuthorError};
use ledger_core::set::Ground;
use ledger_core::tier::Tier;

use super::{resolve_root, EXIT_OK, EXIT_VIOLATIONS};

/// Open the authoring session at the resolved root: identity from git
/// config (OD-3), wall clock, system mint.
pub fn open_author(root: Option<PathBuf>) -> Result<Author, String> {
    let repo_root = resolve_root(root)?;
    Author::open(&repo_root).map_err(|e| e.to_string())
}

/// Map a verb's outcome to the exit discipline.
pub fn finish(result: Result<Applied, AuthorError>) -> Result<i32, String> {
    match result {
        Ok(applied) => {
            for line in &applied.lines {
                println!("{line}");
            }
            println!("filed {}", applied.path.display());
            Ok(EXIT_OK)
        }
        Err(err @ (AuthorError::Refused(_) | AuthorError::Conflict(_))) => {
            eprint!("{err}");
            let text = err.to_string();
            if !text.ends_with('\n') {
                eprintln!();
            }
            Ok(EXIT_VIOLATIONS)
        }
        Err(AuthorError::Usage(m) | AuthorError::Io(m)) => Err(m),
    }
}

pub fn parse_tier(raw: &str) -> Result<Tier, String> {
    match raw {
        "T0" => Ok(Tier::T0),
        "T1" => Ok(Tier::T1),
        "T2" => Ok(Tier::T2),
        other => Err(format!("`{other}` is not a tier — expected T0, T1 or T2")),
    }
}

pub fn parse_ground(raw: &str) -> Result<Ground, String> {
    match raw {
        "characterised" => Ok(Ground::Characterised),
        "uncharacterised" => Ok(Ground::Uncharacterised),
        other => Err(format!("`{other}` is not a ground — expected characterised or uncharacterised")),
    }
}

pub fn parse_store(raw: &str) -> Result<AllocationKind, String> {
    match raw {
        "constraint" => Ok(AllocationKind::Constraint),
        "criterion" => Ok(AllocationKind::Criterion),
        "judgment" => Ok(AllocationKind::Judgment),
        other => Err(format!(
            "`{other}` is not a store — expected constraint, criterion or judgment (escapes go through `ledger escape`)"
        )),
    }
}

pub fn parse_date(raw: &str) -> Result<NaiveDate, String> {
    raw.parse().map_err(|_| format!("`{raw}` is not a date — expected YYYY-MM-DD"))
}

pub fn parse_stage(raw: &str) -> Result<ledger_core::discharge::Stage, String> {
    match raw {
        "pr" => Ok(ledger_core::discharge::Stage::Pr),
        "dev" => Ok(ledger_core::discharge::Stage::Dev),
        "staging" => Ok(ledger_core::discharge::Stage::Staging),
        "prod" => Ok(ledger_core::discharge::Stage::Prod),
        other => Err(format!("`{other}` is not a stage — expected pr, dev, staging or prod")),
    }
}

/// The allocation flags several verbs share, assembled into core args.
#[allow(clippy::too_many_arguments)]
pub fn allocation_args(
    store: Option<&str>,
    discharge: &[String],
    stage: Option<&str>,
    expectation: Option<String>,
    actor: Option<&str>,
) -> Result<AllocationArgs, String> {
    Ok(AllocationArgs {
        store: store.map(parse_store).transpose()?,
        discharge: discharge.iter().map(|d| d.parse()).collect::<Result<Vec<_>, _>>()?,
        discharge_stage: stage.map(parse_stage).transpose()?,
        expectation,
        actor: actor.map(str::parse).transpose()?,
        exposure: None,
        accepted_by: None,
        review_by: None,
    })
}

pub fn parse_based_on(raw: &[String]) -> Result<Vec<ledger_core::version::BasisRef>, String> {
    raw.iter().map(|b| b.parse()).collect()
}

/// Reopen edges parse through their own type, never through `BasisRef` —
/// the distinction the ruling draws is a type distinction here too.
pub fn parse_revisit_if(raw: &[String]) -> Result<Vec<ledger_core::revisit::RevisitRef>, String> {
    raw.iter().map(|r| r.parse()).collect()
}
