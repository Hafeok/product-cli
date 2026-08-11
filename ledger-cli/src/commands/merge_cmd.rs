//! `ledger merge` — the pre-merge plan, the driver plumbing, the install.
//!
//! Merge is two mechanisms (PRD §7): the git merge driver settles the
//! mechanical majority and refuses judgment; `ledger merge --resolve`
//! (in `resolve_cmd`) records the human's arbitration. This module owns
//! the read-only plan (`ledger merge <ref>`), the per-file driver git
//! invokes, and `--install`, which registers driver and attributes.

use std::path::{Path, PathBuf};
use std::process::Command;

use ledger_core::merge::driver::{three_way, DriverOutcome};
use ledger_core::merge::plan;
use ledger_core::{revision, store};

use super::{resolve_root, EXIT_OK, EXIT_VIOLATIONS};

pub struct Flags {
    pub rev: Option<String>,
    pub resolve: bool,
    pub install: bool,
    pub json: bool,
}

pub fn run(root: Option<PathBuf>, flags: Flags) -> Result<i32, String> {
    if flags.install {
        return install(root);
    }
    if flags.resolve {
        return super::resolve_cmd::run(root);
    }
    match flags.rev {
        Some(rev) => plan_against(root, &rev, flags.json),
        None => Err(
            "nothing to do — `ledger merge <ref>` plans, `--resolve` arbitrates, `--install` registers the driver"
                .to_string(),
        ),
    }
}

/// The read-only plan: what merging `<rev>` would do, conflict by conflict.
fn plan_against(root: Option<PathBuf>, rev: &str, json: bool) -> Result<i32, String> {
    let repo = resolve_root(root)?;
    let ours = store::load(&repo);
    let theirs = revision::load_at(&repo, rev)?;
    let (base_label, base) = match merge_base(&repo, rev) {
        Some(base_rev) => {
            let loaded = revision::load_at(&repo, &base_rev)?;
            (short_rev(&base_rev), loaded)
        }
        None => ("no common history".to_string(), store::Store::default()),
    };
    let report = plan::plan(["worktree", rev, &base_label], &ours, &theirs, &base);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
    } else {
        println!("{}", plan::render(&report));
    }
    Ok(if report.conflicts.is_empty() { EXIT_OK } else { EXIT_VIOLATIONS })
}

fn merge_base(repo: &Path, rev: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-base", "HEAD", rev])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn short_rev(rev: &str) -> String {
    rev.get(..12).unwrap_or(rev).to_string()
}

/// The git merge driver: judge one file three-way. Mechanical outcomes
/// complete; judgment exits non-zero with the conflict preserved and the
/// class named on stderr.
pub fn driver(base: &Path, ours: &Path, theirs: &Path, path: &str) -> Result<i32, String> {
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let (b, o, t) = (read(base)?, read(ours)?, read(theirs)?);
    match three_way(path, &b, &o, &t) {
        DriverOutcome::KeepOurs => Ok(EXIT_OK),
        DriverOutcome::TakeTheirs => {
            std::fs::write(ours, t).map_err(|e| e.to_string())?;
            Ok(EXIT_OK)
        }
        DriverOutcome::Conflict { message, .. } => {
            eprintln!("{message}");
            Ok(EXIT_VIOLATIONS)
        }
    }
}

/// Register the driver in repo git config and scope it to the store's
/// files via `.decisions/.gitattributes`.
fn install(root: Option<PathBuf>) -> Result<i32, String> {
    let repo = resolve_root(root)?;
    let attributes = repo.join(ledger_core::STORE_DIR).join(".gitattributes");
    let content = "\
# Structured merge for the decision ledger (L3). The driver completes\n\
# mechanical cases and exits non-zero on anything requiring judgment,\n\
# preserving the conflict for `ledger merge --resolve`.\n\
log/*.yml merge=ledger\n\
log/*.yaml merge=ledger\n\
sets/*.yml merge=ledger\n\
sets/*.yaml merge=ledger\n";
    std::fs::write(&attributes, content).map_err(|e| e.to_string())?;
    git_config(&repo, "merge.ledger.name", "decision ledger structured merge (L3)")?;
    git_config(&repo, "merge.ledger.driver", "ledger merge-driver %O %A %B %P")?;
    println!("wrote {}", attributes.display());
    println!("registered merge.ledger.driver in this repository's git config");
    println!("(git config is per-clone — run `ledger merge --install` once per checkout)");
    Ok(EXIT_OK)
}

fn git_config(repo: &Path, key: &str, value: &str) -> Result<(), String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", key, value])
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git config {key} failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(())
}
