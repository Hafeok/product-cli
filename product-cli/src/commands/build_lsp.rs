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

/// Where diagnostics come from: a live rust-analyzer session, or scripted files
/// under `PRODUCT_MOCK_LSP` (`diag-<n>.json` = a JSON array of messages) so the
/// fix loop is testable in CI without a cargo project + rust-analyzer.
enum Diagnoser {
    Live(LspSession),
    Mock(String),
}

impl Diagnoser {
    fn start(root: &Path) -> Option<Self> {
        if let Some(dir) = std::env::var("PRODUCT_MOCK_LSP").ok().filter(|s| !s.is_empty()) {
            return Some(Diagnoser::Mock(dir));
        }
        match LspSession::start(root) {
            Ok(s) => Some(Diagnoser::Live(s)),
            Err(e) => {
                eprintln!("  rust-analyzer unavailable — skipping ({e})");
                None
            }
        }
    }

    fn diagnose(&mut self, path: &Path) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
        match self {
            Diagnoser::Live(s) => s.diagnostics(path),
            Diagnoser::Mock(dir) => mock_diagnostics(dir, path),
        }
    }

    fn shutdown(&mut self) {
        if let Diagnoser::Live(s) = self {
            s.shutdown();
        }
    }
}

/// The next scripted diagnostics set (`diag-<n>.json` = `["message", …]`);
/// a missing file means clean, so a loop converges once scripts run out.
fn mock_diagnostics(dir: &str, path: &Path) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALL: AtomicUsize = AtomicUsize::new(0);
    let n = CALL.fetch_add(1, Ordering::SeqCst);
    let Ok(text) = std::fs::read_to_string(Path::new(dir).join(format!("diag-{n}.json"))) else {
        return Ok(Vec::new());
    };
    let msgs: Vec<String> = serde_json::from_str(&text)?;
    Ok(msgs
        .into_iter()
        .map(|m| Diagnostic {
            path: path.to_string_lossy().to_string(),
            line: 0,
            character: 0,
            severity: "error".to_string(),
            message: m,
            source: Some("clippy".to_string()),
            code: None,
        })
        .collect())
}

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
    let Some(mut diagnoser) = Diagnoser::start(root) else {
        return 0;
    };
    let mut dirty = 0;
    for path in rust {
        match fix_file(&mut diagnoser, ladder, shared, path, root, max_rounds, budget) {
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
    diagnoser.shutdown();
    dirty
}

/// Diagnose one file; while it is dirty (and rounds remain) re-dispatch a fix,
/// climbing the capability ladder each round.
fn fix_file(diagnoser: &mut Diagnoser, ladder: &[Capability], shared: &str, path: &Path, root: &Path, max_rounds: usize, budget: Option<u64>) -> Result<usize, Box<dyn std::error::Error>> {
    let mut diags = diagnoser.diagnose(path)?;
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
        diags = diagnoser.diagnose(path)?;
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
