//! Build orchestrator (§5) — the new-flow analog of `implement`.
//!
//! Assembles the SPMC frozen context for a deliverable (the What slice, the How
//! to apply, the Decider oracle, the acceptance), optionally spawns an agent to
//! produce the artifact, then reports the §7.2 `done` verdict so the gates are
//! visible in one place.

use product_core::pf::build::assemble;
use product_core::pf::build_spmc::emit_session_spmc;
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_build(deliverable: &str, role: &str, jobs: usize, dry_run: bool, gates: Gates, emit_spmc: bool, out: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let mut d = super::deliverable::load(deliverable)?;
    let slice = super::deliverable::load_slice(&d.slice)?;
    let graph = super::deliverable::load_graph(product.clone())?;
    let deciders = super::deliverable::load_deciders();
    let how = load_how();
    let p = product.clone().or_else(super::shared::default_product_name).unwrap_or_else(|| "product".to_string());

    // Resolve the worker by role → capability (the SPMC Model layer). The §5
    // parallel unit is the work unit (from `cell dispatch`); absent any, the
    // deliverable is one unit of work.
    // The capability ladder (weakest first); the fix loops climb it as rounds fail.
    let ladder = super::worker::ladder(&super::worker::load_catalog(), role);
    let units = load_work_units();

    if emit_spmc {
        return emit(deliverable, &d, &slice, &graph, how.as_ref(), &deciders, &units, &p, out);
    }

    let context = assemble(&d, &slice, &graph, how.as_ref(), &deciders, &p);
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

/// Write (or print, with `--out -`) the self-contained SPMC prompt a Claude Code
/// `-p` session uses to realise the whole deliverable in-repo and self-verify.
#[allow(clippy::too_many_arguments)]
fn emit(deliverable: &str, d: &Deliverable, slice: &Slice, graph: &DomainGraph, how: Option<&HowContract>, deciders: &[Decider], units: &[WorkUnit], product: &str, out: Option<PathBuf>) -> BoxResult {
    let spmc = emit_session_spmc(d, slice, graph, how, deciders, units, product);
    if out.as_deref().map(Path::as_os_str) == Some(std::ffi::OsStr::new("-")) {
        print!("{spmc}");
        return Ok(());
    }
    let path = out.unwrap_or_else(|| {
        super::shared::domain_root().join(".product").join("build").join(format!("{deliverable}.spmc.md"))
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &spmc)?;
    println!("Wrote SPMC → {}", path.display());
    println!("Hand it to a Claude Code session (from the repo root):");
    println!("  claude -p \"$(cat {})\"", path.display());
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
        let paths = super::worker::dispatch(cap, context)?;
        // No declared artifact in the single-unit case, so the worker may not
        // touch the acceptance tests at all (oracle integrity, ADR-076).
        let reverted = super::build_guard::enforce(&root, &[], &paths);
        if !reverted.is_empty() {
            println!("  ! oracle guard: reverted {} worker edit(s) to test files {reverted:?}", reverted.len());
        }
        paths.into_iter().filter(|p| !reverted.contains(p)).collect()
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

/// Dispatch the work units in dependency order (§5): each topological layer runs
/// in parallel, then its oracle artifacts are frozen before the next layer — so a
/// `write-test` unit's test is immutable by the time the `implement` unit that
/// derives from it runs. Coherence is gated afterwards (§6.1) by `report_gates`.
fn dispatch_parallel(units: &[WorkUnit], cap: &Capability, shared: &str, jobs: usize) -> Vec<PathBuf> {
    let root = super::shared::domain_root();
    let layers = product_core::pf::schedule::layers(units);
    println!("Dispatching {} work unit(s) in {} dependency layer(s) → '{}'…", units.len(), layers.len(), cap.id);
    let mut written = Vec::new();
    let mut ok = 0;
    for (li, layer) in layers.iter().enumerate() {
        if layers.len() > 1 {
            println!("  layer {}/{}: {} unit(s)", li + 1, layers.len(), layer.len());
        }
        let layer_units: Vec<WorkUnit> = layer.iter().map(|&i| units[i].clone()).collect();
        let results = run_parallel(layer_units, jobs, |_, wu| {
            let prompt = unit_prompt(shared, wu, units, &root);
            super::worker::dispatch_to(cap, &prompt, &wu.produces.path)
                .map(|paths| (wu.clone(), paths))
                .map_err(|e| format!("{}: {e}", wu.id))
        });
        for r in &results {
            match r {
                Ok((wu, paths)) => {
                    ok += 1;
                    // a unit may only write its own declared artifact; revert any
                    // write to a frozen oracle from an earlier layer.
                    let allowed = unit_artifacts(wu, &root);
                    let reverted = super::build_guard::enforce(&root, &allowed, paths);
                    if !reverted.is_empty() {
                        println!("  ! oracle guard: {} reverted {} out-of-scope write(s) {reverted:?}", wu.id, reverted.len());
                    }
                    written.extend(paths.iter().filter(|p| !reverted.contains(p)).cloned());
                }
                Err(e) => eprintln!("  - failed: {e}"),
            }
        }
        freeze_oracles(&written, &root);
    }
    println!("  {ok}/{} work unit(s) succeeded", units.len());
    written
}

/// The repo path a unit is allowed to write — its one declared artifact.
fn unit_artifacts(wu: &WorkUnit, root: &Path) -> Vec<PathBuf> {
    vec![root.join(&wu.produces.path)]
}

/// Stage (`git add`) every test/oracle file produced so far, freezing it so a
/// later layer's worker — or a fix loop — cannot rewrite it: the guard restores
/// the staged version on any tamper.
fn freeze_oracles(written: &[PathBuf], root: &Path) {
    for p in written {
        let s = p.to_string_lossy();
        if s.ends_with("_tests.rs") || s.ends_with("_test.rs") || s.contains("/tests/") {
            let rel = p.strip_prefix(root).unwrap_or(p);
            let _ = std::process::Command::new("git").arg("-C").arg(root).args(["add", "--"]).arg(rel).status();
        }
    }
}

/// The frozen per-unit prompt: shared context + the unit's instruction, the
/// read-only artifacts of the units it derives from (e.g. the test a `write-test`
/// unit produced, which an `implement` unit must satisfy but not edit), and the
/// current content of the file it edits — so the worker returns a precise edit
/// instead of guessing the shape (§5).
fn unit_prompt(shared: &str, wu: &WorkUnit, all: &[WorkUnit], root: &Path) -> String {
    let mut p = format!("{shared}\n\n## Work unit: {}\n{}", wu.id, wu.prompt);
    for dep in &wu.context.derived_from {
        let upstream = all.iter().find(|u| u.id != wu.id && product_core::pf::schedule::references(dep, &u.id));
        if let Some(path) = upstream.map(|u| u.produces.path.as_str()).filter(|p| !p.is_empty()) {
            if let Ok(content) = std::fs::read_to_string(root.join(path)) {
                p.push_str(&format!(
                    "\n\n## Frozen input — `{path}` (READ-ONLY; satisfy it, do NOT edit it):\n```\n{content}\n```",
                ));
            }
        }
    }
    // The harness writes your content to `produces.path`; if that file already
    // exists, its current content is shown so you return a precise edit.
    let path = &wu.produces.path;
    if let Ok(content) = std::fs::read_to_string(root.join(path)) {
        p.push_str(&format!(
            "\n\n## Existing file to edit: {path}\nThis file already exists — return an `edits` entry that modifies it (find a unique snippet, replace it), NOT a `files` overwrite.\n```\n{content}\n```",
        ));
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
    let fd = feature_done(d, slice, graph, deciders, &super::decider::conformed_set(), &super::deliverable::load_projectors());
    super::deliverable::print_feature_done(&fd);
    fd
}
