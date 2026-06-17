//! Build orchestrator (§5) — the new-flow analog of `implement`.
//!
//! Assembles the SPMC frozen context for a deliverable (the What slice, the How
//! to apply, the Decider oracle, the acceptance), optionally spawns an agent to
//! produce the artifact, then reports the §7.2 `done` verdict so the gates are
//! visible in one place.

use product_core::pf::build::assemble;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::Deliverable;
use product_core::pf::done::feature_done;
use product_core::pf::how::HowContract;
use product_core::pf::model::DomainGraph;
use product_core::pf::slice::Slice;
use std::process::Command;

use super::BoxResult;

pub(crate) fn handle_build(deliverable: &str, dry_run: bool, product: Option<String>) -> BoxResult {
    let d = super::deliverable::load(deliverable)?;
    let slice = super::deliverable::load_slice(&d.slice)?;
    let graph = super::deliverable::load_graph(product.clone())?;
    let deciders = super::deliverable::load_deciders();
    let how = load_how();
    let p = product.clone().or_else(super::shared::default_product_name).unwrap_or_else(|| "product".to_string());
    let context = assemble(&d, &slice, &graph, how.as_ref(), &deciders, &p);

    if dry_run {
        print!("{context}");
        println!("\n--- Gate status ---");
        report_gates(&d, &slice, &graph, &deciders);
        return Ok(());
    }

    // Live: persist the frozen context, spawn the agent, then report the gates.
    let dir = super::shared::domain_root().join(".product").join("build");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{deliverable}.md"));
    std::fs::write(&path, &context)?;
    println!("Frozen build context → {}", path.display());
    spawn_agent(&context)?;
    println!("\n--- Gate status ---");
    report_gates(&d, &slice, &graph, &deciders);
    Ok(())
}

fn load_how() -> Option<HowContract> {
    let path = super::shared::domain_root().join(".product").join("how-contract.yaml");
    std::fs::read_to_string(path).ok().and_then(|t| HowContract::from_yaml(&t).ok())
}

fn report_gates(d: &Deliverable, slice: &Slice, graph: &DomainGraph, deciders: &[Decider]) {
    let fd = feature_done(d, slice, graph, deciders, &super::decider::conformed_set());
    super::deliverable::print_feature_done(&fd);
}

fn spawn_agent(context: &str) -> BoxResult {
    let status = Command::new("claude")
        .arg("-p")
        .arg(context)
        .status()
        .map_err(|e| format!("failed to launch agent (`claude`): {e}"))?;
    if !status.success() {
        return Err(format!("agent exited with {status}").into());
    }
    Ok(())
}
