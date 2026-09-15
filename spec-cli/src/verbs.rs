//! The four verbs this binary owns.
//!
//! Each is a thin adapter: parse the arguments, call `spec_core`, wrap the
//! result in a report. No verdict logic lives here — the gate is `check`'s,
//! the refusal is `store`'s, so neither can be softer at the door than it is
//! at the gate.

use std::path::Path;

use chrono::Utc;
use clap::Args;
use ledger_core::identity::Identity;
use ledger_core::mint::UlidMint;
use product_core::error::{ProductError, Result};
use serde_json::json;
use spec_core::closure::ClosureKind;
use spec_core::record::Opening;
use spec_core::{check, store};

use crate::exit;
use crate::render::Report;

mod git;

#[derive(Args)]
pub struct ImplementArgs {
    /// The slice built against the specification.
    #[arg(long)]
    pub slice: String,
    /// The specification act the slice realises.
    #[arg(long = "act")]
    pub act_ref: String,
    /// Who is opening the record. Defaults to the git identity. May be a
    /// machine — building is the delegable half.
    #[arg(long)]
    pub by: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct CloseArgs {
    /// The record to close.
    pub id: String,
    /// Who answers for the closure. Defaults to the git identity.
    #[arg(long)]
    pub principal: Option<String>,
    /// A determination produced while acting, filed at the address it was
    /// read from. Repeatable.
    #[arg(long = "determination")]
    pub determinations: Vec<String>,
    /// Declare that acting produced no determination. A positive
    /// declaration, not an absence — silence closes nothing.
    #[arg(long)]
    pub nothing_arose: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct RecordsArgs {
    /// Show only records that are still open.
    #[arg(long)]
    pub open: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Verdicts only; the exit code is the output.
    #[arg(long)]
    pub ci: bool,
    #[arg(long)]
    pub json: bool,
}

/// Open an act-time record. Never exits conformant: the write-back has not
/// happened yet, and `implement` is the half that cannot do it.
pub fn implement(root: &Path, args: &ImplementArgs) -> Result<Report> {
    let opened_by = resolve_identity(root, args.by.as_deref())?;
    let opening = Opening {
        id: UlidMint::system().mint(),
        act: "implement".to_string(),
        slice: args.slice.clone(),
        act_ref: args.act_ref.clone(),
        opened_at: Utc::now(),
        opened_by,
        base_revision: git::head_revision(root),
    };
    let record = store::open(root, opening)?;
    let text = format!(
        "opened {} for slice `{}` against `{}`\n  closure pending — `spec close {}` names a principal",
        record.id, record.slice, record.act_ref, record.id
    );
    let body = args.json.then(|| json!({
        "record": record.id,
        "slice": record.slice,
        "act_ref": record.act_ref,
        "status": "pending-closure",
    }));
    Ok(Report::text(exit::PENDING_CLOSURE, text).with_json(body))
}

/// Close an act-time record.
pub fn close(root: &Path, args: &CloseArgs) -> Result<Report> {
    let kind = closure_kind(args)?;
    let principal = resolve_identity(root, args.principal.as_deref())?;
    let closing = store::Closing {
        kind,
        principal,
        at: Utc::now(),
        determinations: args.determinations.clone(),
    };
    match store::close(root, &args.id, closing)? {
        store::Closed::Refused(findings) => Ok(refusal_report(&args.id, &findings, args.json)),
        store::Closed::Sealed(record) => {
            let filed = record.closure.as_ref().map_or(0, |c| c.determinations.len());
            let text =
                format!("closed {} as `{}` ({filed} determination(s))", record.id, kind.as_str());
            let body = args.json.then(|| json!({
                "record": record.id,
                "kind": kind.as_str(),
                "determinations": args.determinations,
                "status": "closed",
            }));
            Ok(Report::text(exit::CONFORMANT, text).with_json(body))
        }
    }
}

/// The report a refused close prints. Same class, same message the gate would
/// report — the refusal is the gate, not a second opinion about it.
fn refusal_report(id: &str, findings: &[check::Finding], as_json: bool) -> Report {
    let listed = findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n  ");
    let text = format!("refusing to close {id} — the write would introduce:\n  {listed}");
    let body = as_json.then(|| json!({
        "record": id,
        "status": "refused",
        "findings": findings.iter().map(|f| json!({
            "class": f.class.to_string(),
            "record": f.record,
            "message": f.message,
        })).collect::<Vec<_>>(),
    }));
    Report::text(exit::FINDINGS, text).with_json(body)
}

/// List act-time records.
pub fn records(root: &Path, args: &RecordsArgs) -> Result<Report> {
    let all = store::load_all(root)?;
    let shown: Vec<_> = all.iter().filter(|r| !args.open || r.is_open()).collect();
    let text = shown
        .iter()
        .map(|r| {
            let state = if r.is_open() { "open" } else { "closed" };
            format!("{:<7} {}  {}  {}", state, r.id, r.slice, r.act_ref)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = args.json.then(|| json!(shown.iter().map(|r| json!({
        "record": r.id,
        "slice": r.slice,
        "act_ref": r.act_ref,
        "open": r.is_open(),
    })).collect::<Vec<_>>()));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// The CI gate.
pub fn check(root: &Path, args: &CheckArgs) -> Result<Report> {
    let all = store::load_all(root)?;
    let findings = check::judge_store(&all);
    let code = if findings.is_empty() { exit::CONFORMANT } else { exit::FINDINGS };
    let text = check_text(&all, &findings, args.ci);
    let body = args.json.then(|| json!({
        "records": all.len(),
        "findings": findings.iter().map(|f| json!({
            "class": f.class.to_string(),
            "record": f.record,
            "message": f.message,
        })).collect::<Vec<_>>(),
    }));
    Ok(Report::text(code, text).with_json(body))
}

fn check_text(all: &[spec_core::ActRecord], findings: &[check::Finding], ci: bool) -> String {
    let listed = findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    if findings.is_empty() {
        return if ci { String::new() } else { format!("{} record(s), no findings", all.len()) };
    }
    if ci {
        listed
    } else {
        format!("{listed}\n\n{} record(s), {} finding(s)", all.len(), findings.len())
    }
}

/// Which closure kind the flags name. Silence is neither, so a `close` that
/// declares nothing is refused rather than defaulted.
fn closure_kind(args: &CloseArgs) -> Result<ClosureKind> {
    match (args.nothing_arose, args.determinations.is_empty()) {
        (true, true) => Ok(ClosureKind::NothingArose),
        (false, false) => Ok(ClosureKind::Determinations),
        (true, false) => Err(ProductError::ConfigError(
            "`--nothing-arose` contradicts the determinations named alongside it".into(),
        )),
        (false, true) => Err(ProductError::ConfigError(
            "a closure declares something: pass `--determination` or `--nothing-arose`".into(),
        )),
    }
}

fn resolve_identity(root: &Path, supplied: Option<&str>) -> Result<Identity> {
    let raw = match supplied {
        Some(s) => s.to_string(),
        None => ledger_core::whoami::git_identity(root).map_err(ProductError::ConfigError)?.as_str().to_string(),
    };
    raw.parse().map_err(ProductError::ConfigError)
}

#[path = "verbs_tests.rs"]
#[cfg(test)]
mod tests;
