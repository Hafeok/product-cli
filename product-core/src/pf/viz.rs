//! View projection of the What graph — a `{nodes, edges}` shape for rendering.
//!
//! Mirrors the field-walk of [`super::turtle`], but instead of RDF triples it
//! emits a flat node/edge list tagged for a two-lane layout: every node carries
//! a `model` lane (`domain` for §3.1 structure, `event` for §3.2 behaviour),
//! and every edge is marked a `bridge` when it crosses from the event lane into
//! the domain lane (behaviour acting on structure). Pure — no I/O.

use std::collections::HashMap;

use serde::Serialize;

use super::model::DomainGraph;

/// The `domain` lane — §3.1 structural kinds.
pub const DOMAIN: &str = "domain";
/// The `event` lane — §3.2 behavioural kinds.
pub const EVENT: &str = "event";

/// One renderable node. `model` is the lane; `context` is the owning bounded
/// context (empty when the kind carries none, e.g. flows and ui-steps).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub model: String,
    pub context: String,
}

/// One renderable edge. `bridge` is true when it runs event -> domain.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub bridge: bool,
}

/// A bounded context, surfaced so the client can label its lane clusters.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewContext {
    pub id: String,
    pub label: String,
}

/// The whole projection: lane-tagged nodes, bridge-marked edges, contexts.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ViewGraph {
    pub nodes: Vec<ViewNode>,
    pub edges: Vec<ViewEdge>,
    pub contexts: Vec<ViewContext>,
}

fn node(id: &str, label: &str, kind: &str, model: &str, context: &str) -> ViewNode {
    ViewNode {
        id: id.to_string(),
        label: label.to_string(),
        kind: kind.to_string(),
        model: model.to_string(),
        context: context.to_string(),
    }
}

/// A pending edge (resolved to a [`ViewEdge`] once every lane is known).
type RawEdge = (String, String, String);

/// Project a [`DomainGraph`] into a renderable [`ViewGraph`].
///
/// Structural kinds (entity, value-object, invariant + relation/mapping edges)
/// land in the domain lane; behavioural kinds (command, event, read-model,
/// ui-step, flow) in the event lane. Bridges are computed last, once every
/// node's lane is known, so the event->domain direction is enforced uniformly.
pub fn to_view_graph(g: &DomainGraph) -> ViewGraph {
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<RawEdge> = Vec::new();
    push_domain(g, &mut nodes, &mut edges);
    push_event(g, &mut nodes, &mut edges);

    let lane: HashMap<&str, &str> = nodes.iter().map(|n| (n.id.as_str(), n.model.as_str())).collect();
    let contexts = g.contexts.iter().map(|c| ViewContext { id: c.id.clone(), label: c.label.clone() }).collect();
    let edges = edges
        .into_iter()
        .map(|(from, to, kind)| {
            let bridge = lane.get(from.as_str()) == Some(&EVENT) && lane.get(to.as_str()) == Some(&DOMAIN);
            ViewEdge { from, to, kind, bridge }
        })
        .collect();

    ViewGraph { nodes, edges, contexts }
}

/// §3.1 structure — entities, value objects, invariants, relations, mappings.
fn push_domain(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for e in &g.entities {
        nodes.push(node(&e.id, &e.label, "entity", DOMAIN, &e.context));
    }
    for vo in &g.value_objects {
        nodes.push(node(&vo.id, &vo.label, "value-object", DOMAIN, &vo.context));
    }
    for i in &g.invariants {
        let ctx = i.context.clone().unwrap_or_default();
        nodes.push(node(&i.id, &i.statement, "invariant", DOMAIN, &ctx));
        if let Some(target) = &i.applies_to {
            edges.push((i.id.clone(), target.clone(), "applies-to".into()));
        }
    }
    for r in &g.relations {
        let label = r.label.clone().unwrap_or_else(|| r.cardinality.clone());
        edges.push((r.from.clone(), r.to.clone(), format!("relation:{label}")));
    }
    for m in &g.context_mappings {
        edges.push((m.concept_a.clone(), m.concept_b.clone(), "mapping".into()));
    }
}

/// §3.2 behaviour — commands, events, read models, ui-steps, flows.
fn push_event(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for c in &g.commands {
        nodes.push(node(&c.id, &c.label, "command", EVENT, &c.context));
        edges.push((c.id.clone(), c.targets.clone(), "targets".into()));
        for ev in &c.emits {
            edges.push((c.id.clone(), ev.clone(), "emits".into()));
        }
    }
    for ev in &g.events {
        nodes.push(node(&ev.id, &ev.label, "event", EVENT, &ev.context));
        edges.push((ev.id.clone(), ev.changes.clone(), "changes".into()));
    }
    for rm in &g.read_models {
        nodes.push(node(&rm.id, &rm.label, "read-model", EVENT, ""));
        for p in &rm.projects {
            edges.push((rm.id.clone(), p.clone(), "projects".into()));
        }
    }
    for w in &g.wireframe_steps {
        nodes.push(node(&w.id, &w.label, "ui-step", EVENT, ""));
        if let Some(d) = &w.displays {
            edges.push((w.id.clone(), d.clone(), "displays".into()));
        }
        if let Some(t) = &w.triggers {
            edges.push((w.id.clone(), t.clone(), "triggers".into()));
        }
        for t in &w.transitions_to {
            edges.push((w.id.clone(), t.clone(), "transitions".into()));
        }
    }
    for f in &g.flows {
        nodes.push(node(&f.id, &f.label, "flow", EVENT, ""));
        for s in &f.steps {
            edges.push((f.id.clone(), s.clone(), "step".into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::*;

    fn sample() -> DomainGraph {
        let mut g = DomainGraph::default();
        g.contexts.push(BoundedContext { id: "ctx".into(), label: "Ctx".into(), ..Default::default() });
        g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "ctx".into(), definition: "d".into(), ..Default::default() });
        g.commands.push(Command { id: "Place".into(), label: "Place".into(), context: "ctx".into(), targets: "Order".into(), emits: vec!["Placed".into()] });
        g.events.push(Event { id: "Placed".into(), label: "Placed".into(), context: "ctx".into(), changes: "Order".into() });
        g.read_models.push(ReadModel { id: "Cart".into(), label: "Cart".into(), projects: vec!["Order".into()], ..Default::default() });
        g
    }

    #[test]
    fn tags_each_node_with_a_lane() {
        let v = to_view_graph(&sample());
        assert!(!v.nodes.is_empty());
        for n in &v.nodes {
            assert!(n.model == DOMAIN || n.model == EVENT, "{} has no lane", n.id);
        }
        let lane = |id: &str| v.nodes.iter().find(|n| n.id == id).map(|n| n.model.as_str());
        assert_eq!(lane("Order"), Some(DOMAIN));
        assert_eq!(lane("Place"), Some(EVENT));
        assert_eq!(lane("Placed"), Some(EVENT));
        assert_eq!(lane("Cart"), Some(EVENT));
    }

    #[test]
    fn bridges_run_event_to_domain_only() {
        let v = to_view_graph(&sample());
        let bridge = |from: &str, to: &str| {
            v.edges.iter().find(|e| e.from == from && e.to == to).map(|e| e.bridge)
        };
        // command -> entity, event -> entity, read-model -> entity all bridge.
        assert_eq!(bridge("Place", "Order"), Some(true));
        assert_eq!(bridge("Placed", "Order"), Some(true));
        assert_eq!(bridge("Cart", "Order"), Some(true));
        // command -> event stays inside the event lane.
        assert_eq!(bridge("Place", "Placed"), Some(false));
        // no edge ever runs domain -> event.
        for e in &v.edges {
            if e.bridge {
                let lane = |id: &str| v.nodes.iter().find(|n| n.id == id).map(|n| n.model.as_str());
                assert_eq!(lane(&e.from), Some(EVENT));
                assert_eq!(lane(&e.to), Some(DOMAIN));
            }
        }
    }
}
