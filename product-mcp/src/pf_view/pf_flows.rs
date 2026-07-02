//! Live projection of §3.2 event-model flows into the swimlane shape the Flows
//! view consumes. A `Flow` only stores its member `steps` (node ids); the lane
//! assignment (Triggers/UI · Commands·Views · per-aggregate event streams) and
//! the causal column order are *computed* here from the graph relationships —
//! trigger→command→event spines and event→view→ui-step crossings.

use std::collections::{BTreeMap, HashMap, HashSet};

use product_core::pf::ids::NodeKind;
use product_core::pf::model::DomainGraph;
use serde_json::{json, Value};

/// Project every flow keyed by id → the swimlane layout object.
pub fn project_flows(g: &DomainGraph) -> Value {
    let mut map = serde_json::Map::new();
    for f in &g.flows {
        map.insert(f.id.clone(), project_one(g, f));
    }
    Value::Object(map)
}

fn project_one(g: &DomainGraph, f: &product_core::pf::model_ui::Flow) -> Value {
    let steps: HashSet<&str> = f.steps.iter().map(|s| s.as_str()).collect();
    let event_agg = event_streams(g, f);
    let has_command = f.steps.iter().any(|id| g.kind_of(id) == Some(NodeKind::Command));
    let lanes = build_lanes(g, &event_agg);
    let edges = build_edges(g, &steps);
    let col = layer(&f.steps, &edges);
    let cols = col.values().copied().max().map(|m| m + 1).unwrap_or(1);
    let nodes = build_nodes(g, f, &col, &event_agg);
    let edge_json: Vec<Value> =
        edges.iter().map(|(a, b, t)| json!({ "from": a, "to": b, "type": t })).collect();

    json!({
        "system": f.system.clone().unwrap_or_default(),
        "name": f.label,
        "pattern": if has_command { "Command + View" } else { "View" },
        "conformance": "realised",
        "lanes": lanes,
        "cols": cols,
        "nodes": nodes,
        "edges": edge_json,
        "meta": command_meta(g, f, &steps),
    })
}

/// Each member event id → the aggregate (entity) it changes — the stream lane.
fn event_streams(g: &DomainGraph, f: &product_core::pf::model_ui::Flow) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for id in &f.steps {
        if let Some(ev) = g.events.iter().find(|e| &e.id == id) {
            out.insert(id.clone(), ev.changes.clone());
        }
    }
    out
}

/// The two rails plus one stream lane per aggregate the flow's events touch.
fn build_lanes(g: &DomainGraph, event_agg: &BTreeMap<String, String>) -> Vec<Value> {
    let mut lanes = vec![
        json!({ "id": "ui", "label": "Triggers / UI", "kind": "rail" }),
        json!({ "id": "cmdview", "label": "Commands · Views", "kind": "rail" }),
    ];
    let mut streams: Vec<String> = event_agg.values().cloned().collect();
    streams.sort();
    streams.dedup();
    for agg in &streams {
        let label = g.entities.iter().find(|e| &e.id == agg).map(|e| e.label.clone()).unwrap_or_else(|| agg.clone());
        lanes.push(json!({ "id": agg, "label": label, "kind": "stream" }));
    }
    lanes
}

/// Spines (trigger→cmd, cmd→event, ui→cmd) and crossings (event→view, view→ui).
fn build_edges(g: &DomainGraph, steps: &HashSet<&str>) -> Vec<(String, String, &'static str)> {
    let mut edges: Vec<(String, String, &'static str)> = Vec::new();
    for t in g.triggers.iter().filter(|t| steps.contains(t.id.as_str())) {
        if steps.contains(t.issues.as_str()) { edges.push((t.id.clone(), t.issues.clone(), "spine")); }
    }
    for c in g.commands.iter().filter(|c| steps.contains(c.id.as_str())) {
        for ev in c.emits.iter().filter(|ev| steps.contains(ev.as_str())) {
            edges.push((c.id.clone(), ev.clone(), "spine"));
        }
    }
    for rm in g.read_models.iter().filter(|r| steps.contains(r.id.as_str())) {
        for ev in rm.projects.iter().filter(|ev| steps.contains(ev.as_str())) {
            edges.push((ev.clone(), rm.id.clone(), "cross"));
        }
    }
    for u in g.wireframe_steps.iter().filter(|w| steps.contains(w.id.as_str())) {
        for s in u.surfaces.iter().filter(|s| steps.contains(s.projection.as_str())) {
            edges.push((s.projection.clone(), u.id.clone(), "cross"));
        }
        for o in u.offers.iter().filter(|o| steps.contains(o.command.as_str())) {
            edges.push((u.id.clone(), o.command.clone(), "spine"));
        }
    }
    edges
}

/// One renderable node per member step, placed in its lane + causal column.
fn build_nodes(g: &DomainGraph, f: &product_core::pf::model_ui::Flow, col: &HashMap<String, usize>, event_agg: &BTreeMap<String, String>) -> Vec<Value> {
    f.steps
        .iter()
        .filter_map(|id| {
            let (kind, label) = node_kind_label(g, id)?;
            let lane = match kind {
                "trigger" | "ui-step" => "ui".to_string(),
                "event" => event_agg.get(id).cloned().unwrap_or_else(|| "cmdview".to_string()),
                _ => "cmdview".to_string(),
            };
            Some(json!({
                "id": id, "kind": kind, "label": label,
                "col": col.get(id).copied().unwrap_or(0), "lane": lane, "sub": id,
            }))
        })
        .collect()
}

/// The renderable kind + label for a member node id, or `None` if unknown.
fn node_kind_label(g: &DomainGraph, id: &str) -> Option<(&'static str, String)> {
    match g.kind_of(id)? {
        NodeKind::Trigger => Some(("trigger", g.triggers.iter().find(|t| t.id == id).map(|t| t.label.clone())?)),
        NodeKind::Command => Some(("command", g.commands.iter().find(|c| c.id == id).map(|c| c.label.clone())?)),
        NodeKind::Event => Some(("event", g.events.iter().find(|e| e.id == id).map(|e| e.label.clone())?)),
        NodeKind::ReadModel => Some(("view", g.read_models.iter().find(|r| r.id == id).map(|r| r.label.clone())?)),
        NodeKind::WireframeStep => Some(("ui-step", g.wireframe_steps.iter().find(|w| w.id == id).map(|w| w.label.clone())?)),
        _ => None,
    }
}

/// Longest-path column index per node over the (from→to) edge list.
fn layer(steps: &[String], edges: &[(String, String, &'static str)]) -> HashMap<String, usize> {
    let mut col: HashMap<String, usize> = steps.iter().map(|s| (s.clone(), 0usize)).collect();
    // Relax |V| times — enough for any DAG; cycles simply stop improving.
    for _ in 0..steps.len().max(1) {
        let mut changed = false;
        for (a, b, _) in edges {
            let na = col.get(a).copied().unwrap_or(0);
            let nb = col.get(b).copied().unwrap_or(0);
            if nb < na + 1 {
                col.insert(b.clone(), na + 1);
                changed = true;
            }
        }
        if !changed { break; }
    }
    col
}

/// Per-command meta (context · guards · in/out) surfaced on selection.
fn command_meta(g: &DomainGraph, f: &product_core::pf::model_ui::Flow, steps: &HashSet<&str>) -> Value {
    let mut meta = serde_json::Map::new();
    for c in g.commands.iter().filter(|c| steps.contains(c.id.as_str())) {
        let out: Vec<String> = c.emits.iter().map(|e| format!("→ {e} (emits)")).collect();
        let ins: Vec<String> = g
            .triggers
            .iter()
            .filter(|t| t.issues == c.id && steps.contains(t.id.as_str()))
            .map(|t| format!("← {} (triggers)", t.id))
            .collect();
        // guards: invariants the aggregate's Decider rejects (best-effort).
        let guards = g
            .invariants
            .iter()
            .filter(|i| i.applies_to.as_deref() == Some(&c.targets))
            .map(|i| i.id.clone())
            .collect::<Vec<_>>()
            .join(", ");
        meta.insert(c.id.clone(), json!({ "context": c.context, "guards": guards, "out": out, "in": ins }));
    }
    let _ = f;
    Value::Object(meta)
}
