//! Context-bundle assembly over the What graph.
//!
//! The domain analog of `context::bundle_feature`: given a focus node (an
//! entity, bounded context, flow, …) and a traversal depth, gather the
//! reachable slice of the captured What graph and render an LLM-ready markdown
//! bundle — the focus node in full, then its neighbourhood grouped by kind.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::query;

/// Assemble a context bundle for `id` to `depth` hops. `None` if `id` is absent.
pub fn bundle(graph: &DomainGraph, id: &str, depth: usize, product: &str) -> Option<String> {
    bundle_many(graph, &[id.to_string()], depth, product)
}

/// Assemble one bundle over the union of several focus nodes — a saved slice of
/// the event model (§7.1). Absent anchors are skipped; `None` if none resolve.
pub fn bundle_many(graph: &DomainGraph, anchors: &[String], depth: usize, product: &str) -> Option<String> {
    let present: Vec<&String> = anchors.iter().filter(|a| graph.kind_of(a).is_some()).collect();
    if present.is_empty() {
        return None;
    }
    // Union of each anchor's reachable set, in discovery order.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut nodes: Vec<String> = Vec::new();
    for a in &present {
        for id in reachable(graph, a, depth) {
            if seen.insert(id.clone()) {
                nodes.push(id);
            }
        }
    }
    let focus: BTreeSet<&str> = present.iter().map(|s| s.as_str()).collect();

    let mut out = String::new();
    let title = present.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
    let focus_desc = present
        .iter()
        .map(|a| format!("{a}:{}", graph.kind_of(a).map(|k| k.class_name()).unwrap_or("?")))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("# Domain Context Bundle: {title}\n\n"));
    out.push_str("⟦Ω:WhatBundle⟧{\n");
    out.push_str(&format!("  product≜{product}:Product\n"));
    out.push_str(&format!("  focus≜{focus_desc}\n"));
    out.push_str(&format!("  depth≜{depth}\n"));
    out.push_str(&format!("  nodes≜{}\n}}\n\n---\n\n", nodes.len()));

    // Focus node(s), in full.
    out.push_str("## Focus\n\n");
    for a in &present {
        out.push_str(&render_node(graph, a));
        out.push('\n');
    }
    out.push_str("---\n\n");

    render_neighbourhood(graph, &focus, &nodes, &mut out);
    Some(out)
}

/// The set of nodes covered by a slice — the union of each anchor's reachable
/// set to `depth` hops. (§7 delivery: a slice's in-scope elements.)
pub fn covered(graph: &DomainGraph, anchors: &[String], depth: usize) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for a in anchors {
        if graph.kind_of(a).is_some() {
            set.extend(reachable(graph, a, depth));
        }
    }
    set
}

/// The directed dependencies of a node — the other nodes it references (its
/// context, targets, emitted events, …). Used by the §7.2 "cut is closed" check.
pub fn dependencies(graph: &DomainGraph, id: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(e) = graph.entities.iter().find(|n| n.id == id) { out.push(e.context.clone()); }
    if let Some(vo) = graph.value_objects.iter().find(|n| n.id == id) { out.push(vo.context.clone()); }
    if let Some(r) = graph.relations.iter().find(|n| n.id == id) { out.push(r.from.clone()); out.push(r.to.clone()); }
    if let Some(i) = graph.invariants.iter().find(|n| n.id == id) {
        out.extend(i.context.clone());
        out.extend(i.applies_to.clone());
    }
    if let Some(m) = graph.context_mappings.iter().find(|n| n.id == id) { out.push(m.concept_a.clone()); out.push(m.concept_b.clone()); }
    if let Some(c) = graph.commands.iter().find(|n| n.id == id) { out.push(c.context.clone()); out.push(c.targets.clone()); out.extend(c.emits.clone()); }
    if let Some(ev) = graph.events.iter().find(|n| n.id == id) { out.push(ev.context.clone()); out.push(ev.changes.clone()); }
    if let Some(rm) = graph.read_models.iter().find(|n| n.id == id) { out.extend(rm.projects.clone()); }
    if let Some(w) = graph.wireframe_steps.iter().find(|n| n.id == id) { out.extend(w.triggers.clone()); out.extend(w.displays.clone()); }
    if let Some(f) = graph.flows.iter().find(|n| n.id == id) { out.extend(f.steps.clone()); }
    out.retain(|s| !s.is_empty());
    out
}

/// BFS the undirected What graph from `id`, returning reachable ids (incl. id)
/// in discovery order, bounded by `depth` hops.
fn reachable(graph: &DomainGraph, id: &str, depth: usize) -> Vec<String> {
    let adj = adjacency(graph);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((id.to_string(), 0));
    seen.insert(id.to_string());
    while let Some((node, d)) = queue.pop_front() {
        order.push(node.clone());
        if d == depth {
            continue;
        }
        if let Some(neigh) = adj.get(&node) {
            for n in neigh {
                if seen.insert(n.clone()) {
                    queue.push_back((n.clone(), d + 1));
                }
            }
        }
    }
    order
}

/// Build the undirected adjacency map over every typed edge in the graph.
fn adjacency(graph: &DomainGraph) -> BTreeMap<String, BTreeSet<String>> {
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut link = |a: &str, b: &str| {
        if !a.is_empty() && !b.is_empty() {
            adj.entry(a.to_string()).or_default().insert(b.to_string());
            adj.entry(b.to_string()).or_default().insert(a.to_string());
        }
    };
    for e in &graph.entities { link(&e.id, &e.context); }
    for vo in &graph.value_objects { link(&vo.id, &vo.context); }
    for r in &graph.relations { link(&r.id, &r.from); link(&r.id, &r.to); }
    for i in &graph.invariants {
        if let Some(c) = &i.context { link(&i.id, c); }
        if let Some(a) = &i.applies_to { link(&i.id, a); }
    }
    for m in &graph.context_mappings { link(&m.id, &m.concept_a); link(&m.id, &m.concept_b); }
    for c in &graph.commands {
        link(&c.id, &c.context);
        link(&c.id, &c.targets);
        for ev in &c.emits { link(&c.id, ev); }
    }
    for ev in &graph.events { link(&ev.id, &ev.context); link(&ev.id, &ev.changes); }
    for rm in &graph.read_models {
        for p in &rm.projects { link(&rm.id, p); }
    }
    for w in &graph.wireframe_steps {
        if let Some(t) = &w.triggers { link(&w.id, t); }
        if let Some(d) = &w.displays { link(&w.id, d); }
    }
    for f in &graph.flows {
        for s in &f.steps { link(&f.id, s); }
    }
    adj
}

/// Render the reachable nodes (minus the focus set) grouped by kind.
fn render_neighbourhood(graph: &DomainGraph, focus: &BTreeSet<&str>, reachable: &[String], out: &mut String) {
    for kind in super::ids::ALL_KINDS {
        let ids: Vec<&String> = reachable
            .iter()
            .filter(|id| !focus.contains(id.as_str()) && graph.kind_of(id) == Some(kind))
            .collect();
        if ids.is_empty() {
            continue;
        }
        out.push_str(&format!("## {}\n\n", section_title(kind)));
        for id in ids {
            out.push_str(&render_node(graph, id));
            out.push('\n');
        }
        out.push_str("---\n\n");
    }
}

fn section_title(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::BoundedContext => "Bounded contexts",
        NodeKind::Entity => "Entities",
        NodeKind::ValueObject => "Value objects",
        NodeKind::Relation => "Relations",
        NodeKind::Invariant => "Invariants",
        NodeKind::ContextMapping => "Context mappings",
        NodeKind::Command => "Commands",
        NodeKind::Event => "Events",
        NodeKind::ReadModel => "Read models",
        NodeKind::WireframeStep => "Wireframe steps",
        NodeKind::Flow => "Flows",
    }
}

/// Render a single node as a markdown block: a `### kind id — label` heading
/// followed by its non-empty fields.
fn render_node(graph: &DomainGraph, id: &str) -> String {
    let Some(value) = query::node_value(graph, id) else {
        return format!("### {id} (missing)\n");
    };
    let kind = graph.kind_of(id).map(|k| k.cli_name()).unwrap_or("node");
    let label = value.get("label").and_then(|l| l.as_str()).unwrap_or("");
    let mut out = format!("### {kind} `{id}`{}\n\n", if label.is_empty() { String::new() } else { format!(" — {label}") });
    if let serde_json::Value::Object(map) = &value {
        for (k, v) in map {
            if k == "id" || k == "label" {
                continue;
            }
            if let Some(s) = render_field(v) {
                out.push_str(&format!("- {k}: {s}\n"));
            }
        }
    }
    out
}

/// Render a field value compactly; `None` for empty/absent so it is skipped.
fn render_field(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Array(a) if !a.is_empty() => {
            let parts: Vec<String> = a.iter().filter_map(render_field).collect();
            (!parts.is_empty()).then(|| parts.join(", "))
        }
        serde_json::Value::Object(o) if !o.is_empty() => {
            Some(serde_json::to_string(v).unwrap_or_else(|_| format!("{o:?}")))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "bundle_tests.rs"]
mod tests;
