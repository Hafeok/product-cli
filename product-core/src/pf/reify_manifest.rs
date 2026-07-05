//! Reify manifest — the whole oracle, by value, language-neutral.
//!
//! The plugin seam is a protocol, not a linkage (the same stance as the
//! §5.1 build seam): everything a language backend needs to render a
//! verification shell is computed once by the deterministic core and
//! serialized — inferred/declared payload shapes, decider + projector
//! scenarios, the oracle-baked flow chains, the screen seam facts with
//! their projector-derived fixtures, and the pinned graph hash. External
//! backends consume this JSON on stdin and answer a file plan; the MCP
//! `product_reify_manifest` tool hands the same document to a session
//! agent so it can realise (or render) without touching the repo.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::Result;

use super::decider::Decider;
use super::decider_logic::{CommandRef, Scenario, State};
use super::decider_sim::Outcome;
use super::model::DomainGraph;
use super::projector::Projector;
use super::projector_logic::ProjectorScenario;
use super::reify::{aggregate_names, input_hash, ReifyOptions};
use super::reify_ident::CsTy;
use super::reify_infer::{infer_shape, Fields};

/// Field name → wire type (`"long" | "bool" | "string"`, or `null` when
/// neither declared nor observed).
pub type FieldTypes = BTreeMap<String, Option<&'static str>>;

#[derive(Serialize)]
pub struct ReifyManifest {
    pub manifest_version: String,
    pub product: String,
    pub namespace: String,
    pub what_version: String,
    pub graph_hash: String,
    pub aggregates: Vec<AggregateManifest>,
    pub projectors: Vec<ProjectorManifest>,
    pub flows: Vec<FlowManifest>,
    pub screens: Vec<ScreenManifest>,
}

#[derive(Serialize)]
pub struct AggregateManifest {
    pub decider_id: String,
    pub aggregate: String,
    pub decides_for: String,
    pub handles: Vec<String>,
    pub emits: Vec<String>,
    pub evolves_from: Vec<String>,
    pub rejects: Vec<String>,
    pub commands: BTreeMap<String, FieldTypes>,
    pub events: BTreeMap<String, FieldTypes>,
    pub state: FieldTypes,
    pub state_defaults: State,
    pub scenarios: Vec<Scenario>,
}

#[derive(Serialize)]
pub struct ProjectorManifest {
    pub projector_id: String,
    pub projects_for: String,
    pub folds: Vec<String>,
    pub view: FieldTypes,
    pub view_defaults: State,
    pub scenarios: Vec<ProjectorScenario>,
}

#[derive(Serialize)]
pub struct FlowManifest {
    pub name: String,
    pub commands: Vec<FlowCommand>,
    pub views: Vec<FlowView>,
}

/// One oracle-baked command step: drive `when` at this point of the
/// stream, expect exactly `outcome`.
#[derive(Serialize)]
pub struct FlowCommand {
    pub decider_id: String,
    pub when: CommandRef,
    pub outcome: Outcome,
}

#[derive(Serialize)]
pub struct FlowView {
    pub projector_id: String,
    pub view: State,
}

#[derive(Serialize)]
pub struct ScreenManifest {
    pub step_id: String,
    pub surfaces: Vec<String>,
    pub offers: Vec<String>,
    /// Non-waived degraded (projection, state) pairs the screen must handle.
    pub degraded_states: Vec<(String, String)>,
    /// The `present`-state view data (a projector scenario's oracle fold).
    pub present_fixture: Option<State>,
}

/// Assemble the manifest for a product.
pub fn manifest(
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
) -> Result<ReifyManifest> {
    let graph_hash = input_hash(graph, &opts.product, deciders, projectors)?;
    let mut sorted: Vec<&Decider> = deciders.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let mut sorted_p: Vec<&Projector> = projectors.iter().collect();
    sorted_p.sort_by(|a, b| a.id.cmp(&b.id));
    let aggs = aggregate_names(&sorted)?;
    Ok(ReifyManifest {
        manifest_version: "1".to_string(),
        product: opts.product.clone(),
        namespace: opts.namespace.clone(),
        what_version: opts.what_version.clone(),
        graph_hash: format!("sha256:{graph_hash}"),
        aggregates: sorted.iter().zip(&aggs).map(|(d, a)| aggregate(graph, d, a)).collect(),
        projectors: sorted_p.iter().map(|p| projector(p)).collect(),
        flows: flows(graph, &sorted, &sorted_p),
        screens: screens(graph, &sorted_p),
    })
}

/// The manifest sliced to one work unit's neighbourhood — the frozen SPMC
/// context for realising a single Decider or Projector, cheap-model sized:
/// the named unit, its counterpart(s) across the event stream (projectors
/// folding the decider's events, or deciders feeding the projector's
/// folds), the flows the retained set fully covers, and the screens that
/// surface a retained read model or offer a retained command. The
/// `graph_hash` stays the **whole product's** — a slice is a view of the
/// same specification, pinned identically.
pub fn manifest_unit(
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
    unit: &str,
) -> Result<ReifyManifest> {
    let full_hash = input_hash(graph, &opts.product, deciders, projectors)?;
    let (ds, ps) = slice_artifacts(deciders, projectors, unit)?;
    let mut m = manifest(graph, &ds, &ps, opts)?;
    m.graph_hash = format!("sha256:{full_hash}");
    let handles: std::collections::BTreeSet<&str> =
        ds.iter().flat_map(|d| d.handles.iter().map(String::as_str)).collect();
    let read_models: std::collections::BTreeSet<&str> =
        ps.iter().map(|p| p.projects_for.as_str()).collect();
    m.screens.retain(|s| {
        s.surfaces.iter().any(|rm| read_models.contains(rm.as_str()))
            || s.offers.iter().any(|c| handles.contains(c.as_str()))
    });
    Ok(m)
}

/// Resolve a unit id to the artifacts in its neighbourhood.
fn slice_artifacts(
    deciders: &[Decider],
    projectors: &[Projector],
    unit: &str,
) -> Result<(Vec<Decider>, Vec<Projector>)> {
    if let Some(d) = deciders.iter().find(|d| d.id == unit) {
        let events: std::collections::BTreeSet<&str> = d
            .emits
            .iter()
            .chain(&d.evolves_from)
            .map(String::as_str)
            .collect();
        let ps = projectors
            .iter()
            .filter(|p| p.folds.iter().any(|f| events.contains(f.as_str())))
            .cloned()
            .collect();
        return Ok((vec![d.clone()], ps));
    }
    if let Some(p) = projectors.iter().find(|p| p.id == unit) {
        let folds: std::collections::BTreeSet<&str> =
            p.folds.iter().map(String::as_str).collect();
        let ds = deciders
            .iter()
            .filter(|d| d.emits.iter().chain(&d.evolves_from).any(|e| folds.contains(e.as_str())))
            .cloned()
            .collect();
        return Ok((ds, vec![p.clone()]));
    }
    let known: Vec<&str> = deciders
        .iter()
        .map(|d| d.id.as_str())
        .chain(projectors.iter().map(|p| p.id.as_str()))
        .collect();
    Err(crate::error::ProductError::NotFound(format!(
        "no decider or projector '{unit}' — known units: {}",
        known.join(", ")
    )))
}

/// The manifest as pretty JSON (trailing newline).
pub fn manifest_json(
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
) -> Result<String> {
    let m = manifest(graph, deciders, projectors, opts)?;
    let mut s = serde_json::to_string_pretty(&m)
        .map_err(|e| crate::error::ProductError::Internal(format!("serialize manifest: {e}")))?;
    s.push('\n');
    Ok(s)
}

fn aggregate(graph: &DomainGraph, d: &Decider, agg: &str) -> AggregateManifest {
    let shape = infer_shape(d, graph);
    AggregateManifest {
        decider_id: d.id.clone(),
        aggregate: agg.to_string(),
        decides_for: d.decides_for.clone(),
        handles: d.handles.clone(),
        emits: d.emits.clone(),
        evolves_from: d.evolves_from.clone(),
        rejects: d.rejects.clone(),
        commands: shape.commands.iter().map(|(k, f)| (k.clone(), types(f))).collect(),
        events: shape.events.iter().map(|(k, f)| (k.clone(), types(f))).collect(),
        state: types(&shape.state),
        state_defaults: shape.state_defaults.clone(),
        scenarios: d.scenarios.clone(),
    }
}

fn projector(p: &Projector) -> ProjectorManifest {
    let (fields, defaults) = super::reify_projector::infer_view(p);
    ProjectorManifest {
        projector_id: p.id.clone(),
        projects_for: p.projects_for.clone(),
        folds: p.folds.clone(),
        view: types(&fields),
        view_defaults: defaults,
        scenarios: p.scenarios.clone(),
    }
}

fn flows(graph: &DomainGraph, sorted: &[&Decider], sorted_p: &[&Projector]) -> Vec<FlowManifest> {
    super::reify_flow::plan_flows(graph, sorted, sorted_p, true)
        .into_iter()
        .map(|f| FlowManifest {
            name: f.name,
            commands: f
                .cmds
                .into_iter()
                .map(|c| FlowCommand { decider_id: c.decider_id, when: c.when, outcome: c.outcome })
                .collect(),
            views: f
                .views
                .into_iter()
                .map(|v| FlowView { projector_id: v.projector_id, view: v.view })
                .collect(),
        })
        .collect()
}

fn screens(graph: &DomainGraph, sorted_p: &[&Projector]) -> Vec<ScreenManifest> {
    super::reify_screen::testable_steps(graph)
        .into_iter()
        .map(|step| ScreenManifest {
            step_id: step.id.clone(),
            surfaces: step.surfaces.iter().map(|s| s.projection.clone()).collect(),
            offers: step.offers.iter().map(|o| o.command.clone()).collect(),
            degraded_states: step
                .state_meanings
                .iter()
                .filter(|m| m.waiver.is_none() && m.state != "present")
                .map(|m| (m.projection.clone(), m.state.clone()))
                .collect(),
            present_fixture: super::reify_screen::present_state(step, sorted_p),
        })
        .collect()
}

fn types(fields: &Fields) -> FieldTypes {
    fields
        .iter()
        .map(|(k, ty)| {
            (
                k.clone(),
                ty.map(|t| match t {
                    CsTy::Bool => "bool",
                    CsTy::Long => "long",
                    CsTy::Str => "string",
                }),
            )
        })
        .collect()
}
