//! Build orchestrator (§5) — the new-flow analog of `implement`.
//!
//! Assembles the SPMC frozen context for a deliverable (the What slice, the How
//! to apply, the Decider oracle, the acceptance), optionally spawns an agent to
//! produce the artifact, then reports the §7.2 `done` verdict so the gates are
//! visible in one place.

use product_core::pf::build::assemble;
use product_core::pf::capability::Capability;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::done::feature_done;
use product_core::pf::how::HowContract;
use product_core::pf::model::DomainGraph;
use product_core::pf::run::run_parallel;
use product_core::pf::slice::Slice;
use product_core::pf::verify;
use product_core::pf::work_unit::WorkUnit;
use std::path::{Path, PathBuf};

use super::BoxResult;

/// Which post-dispatch gates `build` runs (§6).
#[derive(Clone, Copy)]
pub(crate) struct Gates {
    /// Run rust-analyzer diagnose + fix over the worker's Rust output.
    pub lsp: bool,
    /// Run each acceptance criterion's runner, recording the verdict.
    pub verify: bool,
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
        dispatch_live(deliverable, &context, &ladder, &units, jobs, gates, &mut d)?;
    }
    println!("\n--- Gate status ---");
    report_gates(&d, &slice, &graph, &deciders);
    Ok(())
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
        super::build_lsp::run(&written, ladder, context, &root);
    }
    if gates.verify {
        super::build_verify::run(d, &written, ladder, context, &root);
    }
    Ok(())
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

fn report_gates(d: &Deliverable, slice: &Slice, graph: &DomainGraph, deciders: &[Decider]) {
    let fd = feature_done(d, slice, graph, deciders, &super::decider::conformed_set());
    super::deliverable::print_feature_done(&fd);
}
