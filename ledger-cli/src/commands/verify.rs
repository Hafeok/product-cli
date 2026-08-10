//! `ledger verify` — the gate, wired to an exit code.

use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use ledger_core::finding::Gate;
use ledger_core::render::render;
use ledger_core::store;
use ledger_core::verify::{verify, Options};

use super::{resolve_root, EXIT_OK, EXIT_VIOLATIONS};

/// The flags, already separated from clap so the handler stays testable.
pub struct Args {
    pub gate: Option<String>,
    pub json: bool,
    pub today: Option<String>,
    pub blame: bool,
}

pub fn run(root: Option<PathBuf>, args: Args) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let options = Options {
        gate: args.gate.as_deref().map(parse_gate).transpose()?,
        today: parse_today(args.today.as_deref())?,
        blame: args.blame,
    };
    let loaded = store::load(&repo_root);
    let report = verify(&loaded, &options);
    if args.json {
        let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
        println!("{text}");
    } else if report.is_conformant() {
        println!("{}", render(&report));
    } else {
        eprintln!("{}", render(&report));
    }
    Ok(if report.is_conformant() { EXIT_OK } else { EXIT_VIOLATIONS })
}

fn parse_gate(name: &str) -> Result<Gate, String> {
    match name {
        "readiness" => Ok(Gate::Readiness),
        "completeness" => Ok(Gate::Completeness),
        other => Err(format!("`{other}` is not a gate — expected `readiness` or `completeness`")),
    }
}

fn parse_today(raw: Option<&str>) -> Result<NaiveDate, String> {
    match raw {
        None => Ok(Utc::now().date_naive()),
        Some(text) => text
            .parse()
            .map_err(|_| format!("`{text}` is not a date — expected YYYY-MM-DD")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_gates_parse_by_name() {
        assert_eq!(parse_gate("readiness").expect("gate"), Gate::Readiness);
        assert_eq!(parse_gate("completeness").expect("gate"), Gate::Completeness);
        let err = parse_gate("release").expect_err("unknown");
        assert!(err.contains("is not a gate"), "{err}");
    }

    #[test]
    fn an_absent_date_means_today() {
        assert_eq!(parse_today(None).expect("today"), Utc::now().date_naive());
    }

    #[test]
    fn a_malformed_date_says_what_it_wanted() {
        let err = parse_today(Some("10-08-2026")).expect_err("bad date");
        assert!(err.contains("expected YYYY-MM-DD"), "{err}");
    }
}
