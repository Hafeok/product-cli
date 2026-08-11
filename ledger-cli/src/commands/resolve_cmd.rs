//! `ledger merge --resolve` — present each conflict, record the human's
//! arbitration as an ordinary ledger act.
//!
//! Each conflict is shown with both chains, the common parent, and the
//! consequence of each choice; the answer is read from stdin. A skipped
//! or unanswerable conflict stays a conflict — the command exits non-zero
//! rather than resolve anything by default. Acceptance is never part of an
//! arbitration: a reconciled version always exits this command unaccepted.

use std::io::BufRead;
use std::path::{Path, PathBuf};

use ledger_core::author::Author;
use ledger_core::merge::conflicts::{self, Conflicts, Fork, SupersessionConflict};
use ledger_core::merge::resolve::ForkChoice;
use ledger_core::store;

use super::{resolve_root, EXIT_OK, EXIT_VIOLATIONS};

pub fn run(root: Option<PathBuf>) -> Result<i32, String> {
    let repo = resolve_root(root)?;
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut input = move || lines.next().and_then(Result::ok);
    let mut unresolved = resolve_sets(&repo, &mut input)?;
    let found = conflicts::find(&store::load(&repo));
    if found.is_empty() && unresolved == 0 {
        println!("nothing to arbitrate — the store carries no merge conflicts");
        return Ok(EXIT_OK);
    }
    unresolved += resolve_conflicts(&repo, &found, &mut input)?;
    if unresolved > 0 {
        println!("{unresolved} conflict(s) remain — nothing was resolved by default");
        return Ok(EXIT_VIOLATIONS);
    }
    println!("every conflict is arbitrated and on the record; reconciled versions await fresh acceptance");
    Ok(EXIT_OK)
}

fn resolve_conflicts(
    repo: &Path,
    found: &Conflicts,
    input: &mut dyn FnMut() -> Option<String>,
) -> Result<usize, String> {
    let mut unresolved = 0;
    for fork in &found.forks {
        present_fork(fork);
        match read_fork_choice(fork, input) {
            Some(choice) => {
                let mut author = Author::open(repo).map_err(|e| e.to_string())?;
                match author.resolve_fork(fork, &choice) {
                    Ok(applied) => report(&applied),
                    Err(e) => {
                        eprintln!("{e}");
                        unresolved += 1;
                    }
                }
            }
            None => {
                println!("skipped — {} stays forked", fork.decision);
                unresolved += 1;
            }
        }
    }
    for conflict in &found.supersessions {
        present_supersession(conflict);
        match read_index(conflict.claimants.len(), input) {
            Some(winner) => {
                let mut author = Author::open(repo).map_err(|e| e.to_string())?;
                match author.resolve_supersession(conflict, winner) {
                    Ok(applied) => report(&applied),
                    Err(e) => {
                        eprintln!("{e}");
                        unresolved += 1;
                    }
                }
            }
            None => {
                println!("skipped — {} keeps both claims", conflict.target);
                unresolved += 1;
            }
        }
    }
    Ok(unresolved)
}

fn report(applied: &ledger_core::author::Applied) {
    for line in &applied.lines {
        println!("{line}");
    }
    println!("recorded {}", applied.path.display());
}

fn present_fork(fork: &Fork) {
    println!();
    println!("CONFLICT [{}] {}", fork.class, fork.decision);
    match &fork.common_parent {
        Some(parent) => println!("  common parent: {parent}"),
        None => println!("  the chains share no common version"),
    }
    for (i, tip) in fork.tips.iter().enumerate() {
        let accepted = if tip.accepted_by.is_empty() {
            String::new()
        } else {
            format!("; accepted by {}", tip.accepted_by.join(", "))
        };
        println!(
            "  [{}] {} \"{}\" ({}) — filed by {}{accepted}",
            i + 1,
            tip.hash.short(),
            tip.statement,
            tip.allocation,
            tip.filed_by
        );
    }
    println!(
        "  consequence: the chosen content becomes an unaccepted reconciled version; \
the other tip stays on the record as history, and every prior acceptance keeps \
signing only the version it named"
    );
    println!("  choose: <n> that tip stands · c <n> compose on tip n · s skip");
}

/// Read one fork answer. `None` is a skip — on EOF, malformed input, or an
/// explicit `s`: the default is always "still a conflict".
fn read_fork_choice(fork: &Fork, input: &mut dyn FnMut() -> Option<String>) -> Option<ForkChoice> {
    let line = input()?;
    let trimmed = line.trim();
    if let Ok(n) = trimmed.parse::<usize>() {
        return (n >= 1 && n <= fork.tips.len()).then(|| ForkChoice::TipStands(n - 1));
    }
    let rest = trimmed.strip_prefix("c ")?;
    let (n, _) = rest.split_once(' ').unwrap_or((rest, ""));
    let base_tip = n.parse::<usize>().ok().filter(|n| *n >= 1 && *n <= fork.tips.len())? - 1;
    println!("  composed statement:");
    let statement = input()?.trim().to_string();
    (!statement.is_empty()).then_some(ForkChoice::Compose { base_tip, statement })
}

fn present_supersession(conflict: &SupersessionConflict) {
    println!();
    println!("CONFLICT [competing-supersession] {}", conflict.target);
    for (i, claim) in conflict.claimants.iter().enumerate() {
        println!(
            "  [{}] {} \"{}\" — filed by {}",
            i + 1,
            claim.decision,
            claim.statement,
            claim.filed_by
        );
    }
    println!(
        "  consequence: the chosen claim stands; every other claimant files a new \
version withdrawing its edge (its history keeps the claim it once made)"
    );
    println!("  choose: <n> that claim stands · s skip");
}

fn read_index(len: usize, input: &mut dyn FnMut() -> Option<String>) -> Option<usize> {
    let line = input()?;
    let n = line.trim().parse::<usize>().ok()?;
    (n >= 1 && n <= len).then(|| n - 1)
}

/// Set files git left unmerged (the driver's divergent-floor refusal):
/// present base/ours/theirs, record the choice by writing the file and
/// staging it. Returns how many were skipped.
fn resolve_sets(repo: &Path, input: &mut dyn FnMut() -> Option<String>) -> Result<usize, String> {
    let mut unresolved = 0;
    for (path, stages) in unmerged_sets(repo)? {
        println!();
        println!("CONFLICT [divergent-floor] {path}");
        for (name, content) in [("base", &stages[0]), ("ours", &stages[1]), ("theirs", &stages[2])]
        {
            println!("  {name}: {}", set_summary(content));
        }
        println!("  choose: o ours stands · t theirs stands · s skip (edit by hand)");
        let choice = input().map(|l| l.trim().to_string());
        let chosen = match choice.as_deref() {
            Some("o") => Some(&stages[1]),
            Some("t") => Some(&stages[2]),
            _ => None,
        };
        match chosen {
            Some(content) => {
                let file = repo.join(&path);
                std::fs::write(&file, content).map_err(|e| e.to_string())?;
                git_add(repo, &path)?;
                println!("recorded {path} and staged it");
            }
            None => {
                println!("skipped — {path} stays conflicted");
                unresolved += 1;
            }
        }
    }
    Ok(unresolved)
}

fn set_summary(content: &str) -> String {
    serde_yaml::from_str::<ledger_core::set::DecisionSet>(content)
        .map(|s| format!("floor {}, owner {}", s.tolerance_floor, s.owner))
        .unwrap_or_else(|_| "(does not parse)".to_string())
}

/// Unmerged `.decisions/` paths with their three stage contents
/// `[base, ours, theirs]` (empty when a stage is missing).
fn unmerged_sets(repo: &Path) -> Result<Vec<(String, [String; 3])>, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-u", "--", ledger_core::STORE_DIR])
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let mut by_path: Vec<(String, [String; 3])> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Some((meta, path)) = line.split_once('\t') else { continue };
        let mut fields = meta.split_whitespace();
        let (Some(_mode), Some(sha), Some(stage)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(stage) = stage.parse::<usize>() else { continue };
        if !(1..=3).contains(&stage) {
            continue;
        }
        let entry = match by_path.iter_mut().find(|(p, _)| p == path) {
            Some(entry) => entry,
            None => {
                by_path.push((path.to_string(), Default::default()));
                by_path.last_mut().ok_or("unreachable: just pushed")?
            }
        };
        entry.1[stage - 1] = cat_blob(repo, sha)?;
    }
    Ok(by_path)
}

fn cat_blob(repo: &Path, sha: &str) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["cat-file", "blob", sha])
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git cat-file {sha} failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_add(repo: &Path, path: &str) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["add", "--", path])
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git add {path} failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(())
}
