//! Verify fix loop — run each acceptance runner; feed failures back to the worker.
//!
//! The §6 gate that turns "produced" into "working": run every acceptance
//! runner (cargo test, strict clippy), and while any fails (and rounds remain)
//! re-dispatch the worker with the failure output plus the current files, then
//! re-run. Records the final verdict into the deliverable so `done` reads it.
//! Deterministic orchestration, not an open-ended agent loop.

use std::path::{Path, PathBuf};
use std::process::Command;

use product_core::pf::capability::Capability;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::verify::{self, VerifyStep};

/// How much of a runner's output to feed back (its tail — where errors land).
const TAIL: usize = 2400;

/// One runner's outcome: the step, whether it passed, and its captured output.
type Outcome = (VerifyStep, bool, String);

/// Run the verify steps, fixing failures in place, recording final verdicts.
/// Each failed round climbs the capability `ladder` (a stronger model fixes what
/// a weaker one could not).
#[allow(clippy::too_many_arguments)]
pub(super) fn run(d: &mut Deliverable, written: &[PathBuf], ladder: &[Capability], shared: &str, root: &Path, max_rounds: usize, budget: Option<u64>, product: Option<&str>) {
    let steps = verify::plan(d);
    if steps.is_empty() {
        return;
    }
    println!("\n--- Verify + fix (§6) ---");
    super::build_session::set_gate("verify");
    let mut round = 0;
    loop {
        let outcomes: Vec<Outcome> = steps.iter().map(|s| run_step(s, root)).collect();
        record(d, &outcomes);
        for (s, ok, _) in &outcomes {
            println!("  [{}] {}: {} {}", if *ok { "x" } else { " " }, s.criterion, s.program, s.args.join(" "));
        }
        let failing: Vec<&Outcome> = outcomes.iter().filter(|(_, ok, _)| !ok).collect();
        if failing.is_empty() || round >= max_rounds {
            break;
        }
        if super::build_session::over_budget(budget) {
            println!("  budget reached — stopping with {} failing check(s)", failing.len());
            break;
        }
        round += 1;
        let cap = &ladder[round.min(ladder.len() - 1)];
        println!("  re-dispatching to '{}' to fix {} failing check(s) (round {round}/{max_rounds})", cap.id, failing.len());
        match super::worker::dispatch(cap, &fix_prompt(shared, &failing, written, root)) {
            Ok(paths) => {
                let reverted = super::build_guard::enforce(root, written, &paths);
                if !reverted.is_empty() {
                    println!("  ! oracle guard: reverted {} worker edit(s) to test files {reverted:?} — a fix may not rewrite the acceptance test", reverted.len());
                }
            }
            Err(e) => {
                eprintln!("  ! fix dispatch to '{}' failed: {e} — stopping with {} failing check(s)", cap.id, failing.len());
                break;
            }
        }
    }
    if let Err(e) = super::deliverable::save(d, product) {
        eprintln!("  ! could not save verdicts: {e}");
    }
}

fn run_step(s: &VerifyStep, root: &Path) -> Outcome {
    match Command::new(&s.program).args(&s.args).current_dir(root).output() {
        Ok(o) => {
            let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&o.stderr));
            let mut ok = o.status.success();
            if ok && cargo_test_ran_nothing(s, &text) {
                ok = false;
                text.push_str("\nvacuous pass refused: the filter matched zero tests — the acceptance test does not exist yet.");
            }
            (s.clone(), ok, text)
        }
        Err(e) => (s.clone(), false, e.to_string()),
    }
}

/// A cargo-test runner that ran zero tests proves nothing — `cargo test` exits
/// 0 when a filter matches no test, which would record a criterion as passing
/// before its acceptance test exists.
fn cargo_test_ran_nothing(s: &VerifyStep, out: &str) -> bool {
    s.program == "cargo"
        && s.args.first().is_some_and(|a| a == "test")
        && !out.lines().any(|l| l.contains("test result:") && !l.contains(" 0 passed"))
}

fn record(d: &mut Deliverable, outcomes: &[Outcome]) {
    for (step, ok, _) in outcomes {
        if let Some(c) = d.acceptance.iter_mut().find(|c| c.id == step.criterion) {
            c.status = if *ok { "passing" } else { "failing" }.to_string();
        }
    }
}

/// Build the fix prompt: the failing checks' output + the current Rust files.
fn fix_prompt(shared: &str, failing: &[&Outcome], written: &[PathBuf], root: &Path) -> String {
    let mut p = format!("{shared}\n\n## Failing checks — change the code so every one passes\n");
    for (s, _, out) in failing {
        p.push_str(&format!("\n### {} (`{} {}`)\n```\n{}\n```\n", s.criterion, s.program, s.args.join(" "), tail(out)));
    }
    p.push_str("\n## Current files\n");
    for path in written.iter().filter(|p| p.extension().is_some_and(|e| e == "rs")) {
        if let Ok(content) = std::fs::read_to_string(path) {
            p.push_str(&format!("\n### {}\n```\n{content}\n```\n", rel(path, root)));
        }
    }
    p.push_str("\nReturn `edits` (or a `files` overwrite) that make every failing check pass without breaking the others.");
    p
}

fn tail(s: &str) -> &str {
    if s.len() <= TAIL {
        return s;
    }
    let mut start = s.len() - TAIL;
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    &s[start..]
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
}
