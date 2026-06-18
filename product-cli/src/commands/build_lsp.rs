//! LSP fix loop — diagnose the worker's output, re-dispatch fixes in place.
//!
//! After a build dispatches its work, this drives one reused rust-analyzer
//! session over every Rust file the worker wrote: it surfaces diagnostics
//! (clippy via the check command) and, while any remain, re-dispatches the
//! worker with the diagnostics + current content injected, bounded by a small
//! round count. Deterministic orchestration (§6), not an open-ended agent loop.

use std::path::{Path, PathBuf};

use product_core::pf::capability::Capability;
use product_core::pf::lsp::Diagnostic;

use super::lsp::LspSession;

/// Diagnose every written Rust file and fix in place; returns how many files
/// still carry diagnostics after the loop. Each fix round climbs the `ladder`,
/// bounded by `max_rounds` and an optional token `budget`.
pub(super) fn run(written: &[PathBuf], ladder: &[Capability], shared: &str, root: &Path, max_rounds: usize, budget: Option<u64>) -> usize {
    let rust: Vec<&PathBuf> = written.iter().filter(|p| p.extension().is_some_and(|e| e == "rs")).collect();
    if rust.is_empty() {
        return 0;
    }
    println!("\n--- LSP diagnose + fix (rust-analyzer + clippy) ---");
    super::build_session::set_gate("lsp");
    let mut session = match LspSession::start(root) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  rust-analyzer unavailable — skipping ({e})");
            return 0;
        }
    };
    let mut dirty = 0;
    for path in rust {
        match fix_file(&mut session, ladder, shared, path, root, max_rounds, budget) {
            Ok(0) => println!("  [x] {}: clean", rel(path, root)),
            Ok(n) => {
                dirty += 1;
                println!("  [ ] {}: {n} diagnostic(s) remain", rel(path, root));
            }
            Err(e) => {
                dirty += 1;
                eprintln!("  ! {}: {e}", rel(path, root));
            }
        }
    }
    session.shutdown();
    dirty
}

/// Diagnose one file; while it is dirty (and rounds remain) re-dispatch a fix,
/// climbing the capability ladder each round.
fn fix_file(session: &mut LspSession, ladder: &[Capability], shared: &str, path: &Path, root: &Path, max_rounds: usize, budget: Option<u64>) -> Result<usize, Box<dyn std::error::Error>> {
    let mut diags = session.diagnostics(path)?;
    let mut round = 0;
    while !diags.is_empty() && round < max_rounds {
        if super::build_session::over_budget(budget) {
            println!("  budget reached — leaving {} diagnostic(s) on {}", diags.len(), rel(path, root));
            break;
        }
        round += 1;
        let cap = &ladder[round.min(ladder.len() - 1)];
        println!("  fixing {} via '{}' (round {round}/{max_rounds}): {} diagnostic(s)", rel(path, root), cap.id, diags.len());
        let content = std::fs::read_to_string(path)?;
        let prompt = fix_prompt(shared, &rel(path, root), &content, &diags);
        super::worker::dispatch(cap, &prompt)?;
        diags = session.diagnostics(path)?;
    }
    Ok(diags.len())
}

/// The fix prompt: shared context + the diagnostics + the file's current content.
fn fix_prompt(shared: &str, rel_path: &str, content: &str, diags: &[Diagnostic]) -> String {
    let list: Vec<String> = diags
        .iter()
        .map(|d| {
            let src = d.source.as_deref().unwrap_or("rustc");
            let code = d.code.as_deref().map(|c| format!(" {c}")).unwrap_or_default();
            format!("- {}:{}:{} {} ({src}{code}): {}", rel_path, d.line + 1, d.character + 1, d.severity, d.message)
        })
        .collect();
    format!(
        "{shared}\n\n## Fix diagnostics in {rel_path}\nrust-analyzer reports these (resolve every one, introduce no new ones):\n{}\n\n## Current content of {rel_path}\n```\n{content}\n```\nReturn an `edits` entry (or a `files` overwrite of {rel_path}) that makes the file clean.",
        list.join("\n"),
    )
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
}
