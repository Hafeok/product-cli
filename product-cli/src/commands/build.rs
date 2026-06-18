//! Build orchestrator (§5) — the new-flow analog of `implement`.
//!
//! Assembles the SPMC frozen context for a deliverable (the What slice, the How
//! to apply, the Decider oracle, the acceptance), optionally spawns an agent to
//! produce the artifact, then reports the §7.2 `done` verdict so the gates are
//! visible in one place.

use product_core::pf::build::assemble;
use product_core::pf::build_metrics::{BuildSession, FileChange, Verdict};
use product_core::pf::capability::Capability;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::done::{feature_done, FeatureDone};
use product_core::pf::how::HowContract;
use product_core::pf::model::DomainGraph;
use product_core::pf::run::run_parallel;
use product_core::pf::slice::Slice;
use product_core::pf::verify;
use product_core::pf::work_unit::WorkUnit;
use std::path::{Path, PathBuf};

use super::BoxResult;

/// Which post-dispatch gates `build` runs (§6) + their operational limits.
#[derive(Clone, Copy)]
pub(crate) struct Gates {
    /// Run rust-analyzer diagnose + fix over the worker's Rust output.
    pub lsp: bool,
    /// Run each acceptance criterion's runner, recording the verdict.
    pub verify: bool,
    /// Max diagnose→fix rounds per gate before recording what stands.
    pub max_rounds: usize,
    /// Optional token budget; escalation stops once total tokens reach it.
    pub budget: Option<u64>,
}

pub(crate) fn handle_build(deliverable: &str, role: &str, jobs: usize, dry_run: bool, gates: Gates, product: Option<String>) -> BoxResult {
    let mut d = super::deliverable::load(deliverable)?;
    let slice = super::deliverable::load_slice(&d.slice)?;
    let graph = super::deliverable::load_graph(product.clone())?;
    let deciders = super::deliverable::load_deciders();
    let how = load_how();
    let p = product.clone().or_else(super::shared::default_product_name).unwrap_or_else(|| "product".to_string());
    let context = assemble(&d, &slice, &graph, how.as_ref(), &deciders, &p);

    // Resolve the worker by role → capability (the SPMC Model layer). The §5
    // parallel unit is the work unit (from `cell dispatch`); absent any, the
    // deliverable is one unit of work.
    // The capability ladder (weakest first); the fix loops climb it as rounds fail.
    let ladder = super::worker::ladder(&super::worker::load_catalog(), role);
    let units = load_work_units();

    if dry_run {
        print_dry_run(&context, role, &ladder, &units, jobs, gates, &d);
    } else {
        super::build_session::begin(deliverable);
        dispatch_live(deliverable, &context, &ladder, &units, jobs, gates, &mut d)?;
    }
    println!("\n--- Gate status ---");
    let fd = report_gates(&d, &slice, &graph, &deciders);
    if !dry_run {
        finish_session(&fd);
    }
    Ok(())
}

/// Close the build session, persisting + summarizing its cost metrics.
fn finish_session(fd: &FeatureDone) {
    let verdict = Verdict {
        done: fd.done,
        passing: fd.checks.iter().filter(|c| c.passing).count(),
        total: fd.checks.len(),
    };
    let Some(session) = super::build_session::finish(verdict) else {
        return;
    };
    let dir = super::shared::domain_root().join(".product").join("build");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = session.to_json() {
        let _ = std::fs::write(dir.join(format!("{}.session.json", session.deliverable)), json);
    }
    summarize_session(&session);
}

/// Print the session's cost + outcome metrics for the feature slice.
fn summarize_session(s: &BuildSession) {
    println!("\n--- Session ---");
    println!("  verdict: {} ({}/{} checks)", if s.verdict.done { "DONE" } else { "not done" }, s.verdict.passing, s.verdict.total);
    println!("  tokens: {} ({} prompt + {} completion)", s.total_tokens(), s.prompt_tokens(), s.completion_tokens());
    let rounds: Vec<String> = s.rounds().iter().map(|(g, n)| format!("{g}={n}")).collect();
    println!("  calls by gate: {}", rounds.join(", "));
    for (cap, t) in s.tokens_by_capability() {
        if t > 0 {
            println!("  {cap}: {t} tokens");
        }
    }
    println!("  elapsed: {}s", s.elapsed_secs);
    println!("  record: .product/build/{}.session.json", s.deliverable);
}

/// Show the assembled context, worker ladder, run plan, and verify plan — no dispatch.
fn print_dry_run(context: &str, role: &str, ladder: &[Capability], units: &[WorkUnit], jobs: usize, gates: Gates, d: &Deliverable) {
    let cap = &ladder[0];
    print!("{context}");
    println!("\n--- Worker ---");
    println!("role '{role}' → capability '{}' (endpoint {}, model {})", cap.id, cap.endpoint, cap.model_identifier);
    if ladder.len() > 1 {
        let rungs: Vec<&str> = ladder.iter().map(|c| c.id.as_str()).collect();
        println!("escalation ladder: {}", rungs.join(" ⇡ "));
    }
    println!(
        "limits: max {} round(s)/gate, budget {}",
        gates.max_rounds,
        gates.budget.map(|b| format!("{b} tokens")).unwrap_or_else(|| "unbounded".to_string()),
    );
    if !units.is_empty() {
        println!("\n--- Parallel run plan ---");
        println!("{jobs} job(s) over {} work unit(s):", units.len());
        for u in units {
            println!("  - {} → {} ({})", u.id, cap.id, cap.endpoint);
        }
    }
    if gates.lsp {
        println!("\n--- LSP gate ---\n  rust-analyzer diagnose + fix over the worker's Rust output");
    }
    if gates.verify {
        print_verify_plan(d);
    }
}

/// Persist the frozen context, dispatch the work, then run the LSP + verify gates.
fn dispatch_live(deliverable: &str, context: &str, ladder: &[Capability], units: &[WorkUnit], jobs: usize, gates: Gates, d: &mut Deliverable) -> BoxResult {
    let root = super::shared::domain_root();
    let cap = &ladder[0];
    let dir = root.join(".product").join("build");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{deliverable}.md")), context)?;
    let written = if units.is_empty() {
        println!("Dispatching to '{}' (endpoint {})…", cap.id, cap.endpoint);
        super::worker::dispatch(cap, context)?
    } else {
        dispatch_parallel(units, cap, context, jobs)
    };
    if gates.lsp {
        super::build_lsp::run(&written, ladder, context, &root, gates.max_rounds, gates.budget);
    }
    if gates.verify {
        super::build_verify::run(d, &written, ladder, context, &root, gates.max_rounds, gates.budget);
    }
    report_changes(&written, &root);
    Ok(())
}

/// Surface which repo files the worker created vs modified — the safety review
/// surface when running a worker against a real tree.
fn report_changes(written: &[PathBuf], root: &Path) {
    let paths: std::collections::BTreeSet<&PathBuf> = written.iter().collect();
    if paths.is_empty() {
        return;
    }
    println!("\n--- Changes ---");
    let mut changes = Vec::new();
    for p in paths {
        let rel = p.strip_prefix(root).unwrap_or(p).to_string_lossy().to_string();
        let status = git_status(root, &rel);
        println!("  {status:8} {rel}");
        changes.push(FileChange { path: rel, status: status.to_string() });
    }
    super::build_session::set_files(changes);
}

/// A file's git state, classified for the change summary.
fn git_status(root: &Path, rel: &str) -> &'static str {
    let Ok(out) = std::process::Command::new("git")
        .arg("-C").arg(root)
        .args(["status", "--porcelain", "--", rel])
        .output()
    else {
        return "written";
    };
    let line = String::from_utf8_lossy(&out.stdout);
    let code = line.lines().next().unwrap_or("").get(0..2).unwrap_or("");
    if code.contains('?') {
        "new"
    } else if code.contains('A') {
        "added"
    } else if code.contains('M') {
        "modified"
    } else if code.trim().is_empty() {
        "unchanged"
    } else {
        "changed"
    }
}

/// Show the verify steps `build` will run (dry-run).
fn print_verify_plan(d: &Deliverable) {
    let steps = verify::plan(d);
    println!("\n--- Verify plan (§6) ---");
    if steps.is_empty() {
        println!("  (no acceptance criteria carry a runner)");
    }
    for s in &steps {
        println!("  - {}: {} {}", s.criterion, s.program, s.args.join(" "));
    }
    for id in verify::unknown_runners(d) {
        println!("  ! {id}: unknown runner — skipped");
    }
}

/// Fan the work units out across at most `jobs` workers, each its own capability
/// instance. Coherence is gated afterwards (§6.1) by `report_gates`.
fn dispatch_parallel(units: &[WorkUnit], cap: &Capability, shared: &str, jobs: usize) -> Vec<PathBuf> {
    println!("Dispatching {} work unit(s) across {jobs} job(s) → '{}'…", units.len(), cap.id);
    let root = super::shared::domain_root();
    let results = run_parallel(units.to_vec(), jobs, |_, wu| {
        let prompt = unit_prompt(shared, wu, &root);
        super::worker::dispatch(cap, &prompt).map_err(|e| format!("{}: {e}", wu.id))
    });
    let mut written = Vec::new();
    let mut ok = 0;
    for r in &results {
        match r {
            Ok(paths) => {
                ok += 1;
                written.extend(paths.clone());
            }
            Err(e) => eprintln!("  - failed: {e}"),
        }
    }
    println!("  {ok}/{} work unit(s) succeeded", results.len());
    written
}

/// The frozen per-unit prompt: shared context + the unit's instruction, plus the
/// current content of the file a wiring unit edits — so the worker returns a
/// precise edit instead of guessing the file's shape (§5).
fn unit_prompt(shared: &str, wu: &WorkUnit, root: &Path) -> String {
    let mut p = format!("{shared}\n\n## Work unit: {}\n{}", wu.id, wu.prompt);
    if let Some(path) = &wu.produces.path_hint {
        if let Ok(content) = std::fs::read_to_string(root.join(path)) {
            p.push_str(&format!(
                "\n\n## Existing file to edit: {path}\nThis file already exists — return an `edits` entry that modifies it (find a unique snippet, replace it), NOT a `files` overwrite.\n```\n{content}\n```",
            ));
        }
    }
    p
}

fn work_units_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("work-units")
}

fn load_work_units() -> Vec<WorkUnit> {
    let dir = work_units_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut units: Vec<WorkUnit> = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|t| WorkUnit::from_yaml(&t).ok())
        .collect();
    units.sort_by(|a, b| a.id.cmp(&b.id));
    units
}

fn load_how() -> Option<HowContract> {
    let pdir = super::shared::domain_root().join(".product");
    let mut candidates = vec![pdir.join("how-contract.yaml")];
    if let Some(product) = super::shared::default_product_name() {
        candidates.push(pdir.join("archetypes").join(product).join("how-contract.yaml"));
    }
    candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok().and_then(|t| HowContract::from_yaml(&t).ok()))
}

fn report_gates(d: &Deliverable, slice: &Slice, graph: &DomainGraph, deciders: &[Decider]) -> FeatureDone {
    let fd = feature_done(d, slice, graph, deciders, &super::decider::conformed_set());
    super::deliverable::print_feature_done(&fd);
    fd
}
