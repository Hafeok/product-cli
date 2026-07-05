//! Reify-from-How adapter — run the contract's declared realisations.
//!
//! `product reify emit` reads the §4.2 `realisations:` block of the How
//! contract — the human's captured backend/tier/namespace decision — and
//! derives every emission from it: built-in backends via the registry,
//! external ones via their declared `plugin_cmd` over the manifest
//! protocol. A realisation's `system` link is checked against the What at
//! emit time, so a contract cannot claim to realise a system the graph
//! does not declare.

use product_core::pf::decider::Decider;
use product_core::pf::ids::NodeKind;
use product_core::pf::model::DomainGraph;
use product_core::pf::projector::Projector;
use product_core::pf::reify::{ReifyOptions, ReifyPlan};
use product_core::pf::reify_backend::{
    backend, external_plan, realisation_opts, realisation_out, resolve_realisations,
};
use product_core::pf::HowContract;

use super::BoxResult;

pub(super) fn emit_from_how(id: Option<String>, product: Option<String>) -> BoxResult {
    let (name, graph, deciders, projectors, base) = super::reify::resolve_inputs(None, product)?;
    let path = super::shared::artifact_dir(Some(&name), "").join("how-contract.yaml");
    let contract = HowContract::load_opt(&path)?
        .ok_or_else(|| format!("no how-contract at {} — scaffold one with `product how init`, then declare a §4.2 `realisations:` block", path.display()))?;
    for r in resolve_realisations(&contract, id.as_deref())? {
        check_system(&graph, r)?;
        let mut opts = realisation_opts(r, &name, &base.what_version)?;
        opts.design_system = super::reify::load_bound_ds(Some(&name))?;
        let plan = plan_for(r, &graph, &deciders, &projectors, &opts)?;
        let root = super::shared::domain_root().join(realisation_out(r, &name));
        let stale = super::reify::remove_stale(&root, &plan);
        let (written, kept) = super::reify::write_plan(&root, &plan, false)?;
        let mode = format!(
            "{}{}, from the How",
            r.backend,
            if opts.oracle_only { " oracle-only" } else { " full" },
        );
        super::reify::report(
            &name, &root, &r.id, &mode, &plan, (written, kept, stale),
            "see the tree's README.g.md; drift gate: `product reify check --out <dir>`",
        );
    }
    Ok(())
}

/// A realisation's `system` link must name a declared §3.2.5 system.
fn check_system(graph: &DomainGraph, r: &product_core::pf::how::Realisation) -> BoxResult {
    if let Some(sys) = &r.system {
        if !graph.is_kind(sys, NodeKind::System) {
            return Err(format!(
                "realisation '{}' is for system '{sys}', but the What graph declares no such system",
                r.id
            )
            .into());
        }
    }
    Ok(())
}

fn plan_for(
    r: &product_core::pf::how::Realisation,
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
) -> Result<ReifyPlan, Box<dyn std::error::Error>> {
    if r.backend == "plugin" {
        let cmd = r.plugin_cmd.as_deref().ok_or_else(|| {
            format!("realisation '{}' has backend 'plugin' but no plugin_cmd", r.id)
        })?;
        return run_external(cmd, graph, deciders, projectors, opts);
    }
    Ok(backend(&r.backend)?.plan(graph, deciders, projectors, opts)?)
}

/// Spawn an external backend: manifest JSON on stdin, file plan on stdout.
pub(super) fn run_external(
    cmd: &str,
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
) -> Result<ReifyPlan, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let json = product_core::pf::reify_manifest::manifest_json(graph, deciders, projectors, opts)?;
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start backend plugin: {e}"))?;
    {
        let mut stdin = child.stdin.take().ok_or("plugin has no stdin")?;
        stdin.write_all(json.as_bytes())?;
    } // closing stdin lets the plugin finish
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "plugin failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(external_plan(&String::from_utf8_lossy(&output.stdout), graph, deciders, projectors, opts)?)
}
