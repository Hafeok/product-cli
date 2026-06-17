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
use product_core::pf::work_unit::WorkUnit;
use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn handle_build(deliverable: &str, role: &str, jobs: usize, dry_run: bool, product: Option<String>) -> BoxResult {
    let d = super::deliverable::load(deliverable)?;
    let slice = super::deliverable::load_slice(&d.slice)?;
    let graph = super::deliverable::load_graph(product.clone())?;
    let deciders = super::deliverable::load_deciders();
    let how = load_how();
    let p = product.clone().or_else(super::shared::default_product_name).unwrap_or_else(|| "product".to_string());
    let context = assemble(&d, &slice, &graph, how.as_ref(), &deciders, &p);

    // Resolve the worker by role → capability (the SPMC Model layer).
    let cap = super::worker::resolve(&super::worker::load_catalog(), role, &[]);
    // The §5 parallel unit: work units (from `cell dispatch`). When present they
    // fan out across workers; otherwise the deliverable is one unit of work.
    let units = load_work_units();

    if dry_run {
        print!("{context}");
        println!("\n--- Worker ---");
        println!("role '{role}' → capability '{}' (endpoint {}, model {})", cap.id, cap.endpoint, cap.model_identifier);
        if !units.is_empty() {
            println!("\n--- Parallel run plan ---");
            println!("{jobs} job(s) over {} work unit(s):", units.len());
            for u in &units {
                println!("  - {} → {} ({})", u.id, cap.id, cap.endpoint);
            }
        }
        println!("\n--- Gate status ---");
        report_gates(&d, &slice, &graph, &deciders);
        return Ok(());
    }

    // Live: persist the frozen context, dispatch, gate.
    let dir = super::shared::domain_root().join(".product").join("build");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{deliverable}.md")), &context)?;
    if units.is_empty() {
        println!("Dispatching to '{}' (endpoint {})…", cap.id, cap.endpoint);
        super::worker::dispatch(&cap, &context)?;
    } else {
        dispatch_parallel(&units, &cap, &context, jobs);
    }
    println!("\n--- Gate status ---");
    report_gates(&d, &slice, &graph, &deciders);
    Ok(())
}

/// Fan the work units out across at most `jobs` workers, each its own capability
/// instance. Coherence is gated afterwards (§6.1) by `report_gates`.
fn dispatch_parallel(units: &[WorkUnit], cap: &Capability, shared: &str, jobs: usize) {
    println!("Dispatching {} work unit(s) across {jobs} job(s) → '{}'…", units.len(), cap.id);
    let results = run_parallel(units.to_vec(), jobs, |_, wu| {
        let prompt = format!("{shared}\n\n## Work unit: {}\n{}", wu.id, wu.prompt);
        super::worker::dispatch(cap, &prompt)
            .map(|_| wu.id.clone())
            .map_err(|e| format!("{}: {e}", wu.id))
    });
    let ok = results.iter().filter(|r| r.is_ok()).count();
    println!("  {ok}/{} work unit(s) succeeded", results.len());
    for r in &results {
        if let Err(e) = r {
            eprintln!("  - failed: {e}");
        }
    }
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
    let path = super::shared::domain_root().join(".product").join("how-contract.yaml");
    std::fs::read_to_string(path).ok().and_then(|t| HowContract::from_yaml(&t).ok())
}

fn report_gates(d: &Deliverable, slice: &Slice, graph: &DomainGraph, deciders: &[Decider]) {
    let fd = feature_done(d, slice, graph, deciders, &super::decider::conformed_set());
    super::deliverable::print_feature_done(&fd);
}
