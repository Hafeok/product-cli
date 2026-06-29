//! View projection of the What graph — a `{nodes, edges}` shape for rendering.
//!
//! Mirrors the field-walk of [`super::turtle`], but instead of RDF triples it
//! emits a flat node/edge list tagged for layout: every node carries a `model`
//! lane (`domain` for §3.1 structure, `event` for §3.2 behaviour), and every
//! edge is marked a `bridge` when it crosses from the event lane into the
//! domain lane (behaviour acting on structure). Nodes additionally carry the
//! per-kind detail the live web views render (entity fields, system platforms,
//! product purpose, …). Pure — no I/O.

use std::collections::HashMap;

use serde::Serialize;

use super::model::DomainGraph;

/// The `domain` lane — §3.1 structural kinds.
pub const DOMAIN: &str = "domain";
/// The `event` lane — §3.2 behavioural kinds.
pub const EVENT: &str = "event";

fn is_false(b: &bool) -> bool {
    !*b
}

/// One renderable node. `model` is the lane; `context` is the owning bounded
/// context (empty when the kind carries none, e.g. flows and ui-steps; a
/// system reuses it to carry its `kind`, e.g. "application"). The trailing
/// fields are per-kind detail surfaced for the live views: `fields` are the
/// body lines (entity attributes, an invariant statement, a system's quality
/// demands), `purpose` the one-line gloss (product / system), `tags` the chips
/// (a system's platforms + interaction classes, a product's version, a
/// trigger's source), and `aggregate` flags an entity that is an aggregate root.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub model: String,
    pub context: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub aggregate: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// One renderable edge. `bridge` is true when it runs event -> domain. `card`
/// carries a relation's cardinality; `label` its human label (relations,
/// ownership, references), both empty when the edge kind carries none.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub bridge: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub card: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
}

/// A bounded context, surfaced so the client can label its lane clusters and
/// render the Systems-map domain cards (its `glossary` is the ubiquitous
/// language; `purpose` the one-line gloss).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ViewContext {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glossary: Vec<String>,
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
        fields: Vec::new(),
        aggregate: false,
        purpose: String::new(),
        tags: Vec::new(),
    }
}

/// A pending edge (resolved to a [`ViewEdge`] once every lane is known).
#[derive(Default)]
struct RawEdge {
    from: String,
    to: String,
    kind: String,
    card: String,
    label: String,
}

fn re(from: &str, to: &str, kind: &str) -> RawEdge {
    RawEdge { from: from.into(), to: to.into(), kind: kind.into(), ..Default::default() }
}

/// Project a [`DomainGraph`] into a renderable [`ViewGraph`].
///
/// Structural kinds (entity, value-object, invariant, system, product, journey,
/// plus relation/mapping edges) land in the domain lane; behavioural kinds
/// (command, event, read-model, ui-step, trigger, flow) in the event lane.
/// Bridges are computed last, once every node's lane is known, so the
/// `event->domain` direction is enforced uniformly.
pub fn to_view_graph(g: &DomainGraph) -> ViewGraph {
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<RawEdge> = Vec::new();
    push_domain(g, &mut nodes, &mut edges);
    push_event(g, &mut nodes, &mut edges);
    push_product(g, &mut nodes, &mut edges);

    let lane: HashMap<&str, &str> = nodes.iter().map(|n| (n.id.as_str(), n.model.as_str())).collect();
    let contexts = g
        .contexts
        .iter()
        .map(|c| ViewContext {
            id: c.id.clone(),
            label: c.label.clone(),
            purpose: c.purpose.clone().unwrap_or_default(),
            glossary: c.glossary.clone(),
        })
        .collect();
    let edges = edges
        .into_iter()
        .map(|e| {
            let bridge = lane.get(e.from.as_str()) == Some(&EVENT) && lane.get(e.to.as_str()) == Some(&DOMAIN);
            ViewEdge { from: e.from, to: e.to, kind: e.kind, bridge, card: e.card, label: e.label }
        })
        .collect();

    ViewGraph { nodes, edges, contexts }
}

/// One `"name: type"` line per attribute (the type is optional), identity first.
fn attr_lines(e: &super::model::Entity) -> Vec<String> {
    let mut out: Vec<String> = e
        .attributes
        .iter()
        .map(|a| match &a.ty {
            Some(t) => format!("{}: {}", a.name, t),
            None => a.name.clone(),
        })
        .collect();
    if let Some(id) = &e.identity {
        out.insert(0, format!("identity: {id}"));
    }
    out
}

/// §3.1 structure — entities, value objects, invariants, relations, mappings.
fn push_domain(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for e in &g.entities {
        let mut n = node(&e.id, &e.label, "entity", DOMAIN, &e.context);
        n.aggregate = e.is_aggregate_root;
        n.fields = attr_lines(e);
        nodes.push(n);
    }
    for vo in &g.value_objects {
        let mut n = node(&vo.id, &vo.label, "value-object", DOMAIN, &vo.context);
        if let Some(d) = &vo.definition {
            n.fields = vec![d.clone()];
        }
        nodes.push(n);
    }
    for i in &g.invariants {
        let ctx = i.context.clone().unwrap_or_default();
        // The label is the short id; the statement is the body line, so the ER
        // view reads "CART-1" over its rule rather than the whole sentence.
        let mut n = node(&i.id, &i.id, "invariant", DOMAIN, &ctx);
        n.fields = vec![i.statement.clone()];
        nodes.push(n);
        if let Some(target) = &i.applies_to {
            edges.push(re(&i.id, target, "applies-to"));
        }
    }
    for r in &g.relations {
        let label = r.label.clone().unwrap_or_default();
        edges.push(RawEdge {
            from: r.from.clone(),
            to: r.to.clone(),
            kind: "relation".into(),
            card: r.cardinality.clone(),
            label,
        });
    }
    for m in &g.context_mappings {
        edges.push(RawEdge {
            from: m.concept_a.clone(),
            to: m.concept_b.clone(),
            kind: "mapping".into(),
            label: m.kind.clone().unwrap_or_default(),
            ..Default::default()
        });
    }
}

/// §3.2 behaviour — commands, events, read models, ui-steps, triggers, flows.
fn push_event(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for c in &g.commands {
        nodes.push(node(&c.id, &c.label, "command", EVENT, &c.context));
        edges.push(re(&c.id, &c.targets, "targets"));
        for ev in &c.emits {
            edges.push(re(&c.id, ev, "emits"));
        }
    }
    for ev in &g.events {
        nodes.push(node(&ev.id, &ev.label, "event", EVENT, &ev.context));
        edges.push(re(&ev.id, &ev.changes, "changes"));
    }
    for rm in &g.read_models {
        let mut n = node(&rm.id, &rm.label, "read-model", EVENT, "");
        n.fields = rm.states.clone();
        nodes.push(n);
        for p in &rm.projects {
            edges.push(re(&rm.id, p, "projects"));
        }
    }
    for w in &g.wireframe_steps {
        nodes.push(node(&w.id, &w.label, "ui-step", EVENT, ""));
        if let Some(d) = &w.displays {
            edges.push(re(&w.id, d, "displays"));
        }
        if let Some(t) = &w.triggers {
            edges.push(re(&w.id, t, "triggers"));
        }
        for t in &w.transitions_to {
            edges.push(re(&w.id, t, "transitions"));
        }
    }
    for f in &g.flows {
        nodes.push(node(&f.id, &f.label, "flow", EVENT, ""));
        for s in &f.steps {
            edges.push(re(&f.id, s, "step"));
        }
        if let Some(sys) = &f.system {
            edges.push(re(&f.id, sys, "system-of"));
        }
    }
    // §3.2.0 — Triggers initiate commands (the top of the Command pattern);
    // an automated Trigger watches a View (Automation/Translation).
    for t in &g.triggers {
        let mut n = node(&t.id, &t.label, "trigger", EVENT, "");
        if !t.source.is_empty() {
            n.tags = vec![t.source.clone()];
        }
        nodes.push(n);
        edges.push(re(&t.id, &t.issues, "issues"));
        if let Some(w) = &t.watches {
            edges.push(re(&t.id, w, "watches"));
        }
    }
}

/// An ownership/reference edge carrying a human label (no cardinality).
fn owns(from: &str, to: &str, kind: &str, label: &str) -> RawEdge {
    RawEdge { from: from.into(), to: to.into(), kind: kind.into(), label: label.into(), ..Default::default() }
}

/// Quality demands grouped by the element they scope (a node's body lines).
fn demand_lines(g: &DomainGraph) -> HashMap<&str, Vec<String>> {
    let mut demands: HashMap<&str, Vec<String>> = HashMap::new();
    for q in &g.quality_demands {
        let line = if q.bound.is_empty() { q.label.clone() } else { format!("{}: {}", q.label, q.bound) };
        demands.entry(q.scopes.as_str()).or_default().push(line);
    }
    demands
}

/// §3.0–§3.6 product boundary — products, systems, journeys, quality demands.
/// These sit *above* the per-system What; the Systems map renders them.
fn push_product(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    let demands = demand_lines(g);
    push_systems(g, &demands, nodes, edges);
    push_products(g, &demands, nodes, edges);
    push_journeys(g, nodes, edges);
}

/// §3.2.5 — Systems own flows; reference (not own) whole domains. The system
/// `kind` rides on the otherwise-unused `context` field.
fn push_systems(g: &DomainGraph, demands: &HashMap<&str, Vec<String>>, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for s in &g.systems {
        let mut n = node(&s.id, &s.label, "system", DOMAIN, &s.kind);
        n.purpose = s.purpose.clone();
        n.tags = s.target_classes.iter().chain(s.target_platforms.iter()).cloned().collect();
        n.fields = demands.get(s.id.as_str()).cloned().unwrap_or_default();
        nodes.push(n);
        for d in &s.references_domain {
            edges.push(owns(&s.id, d, "references", "references"));
        }
    }
}

/// §3.0 — the product owns domains + systems (the root of the What).
fn push_products(g: &DomainGraph, demands: &HashMap<&str, Vec<String>>, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for p in &g.products {
        let mut n = node(&p.id, &p.label, "product", DOMAIN, "");
        n.purpose = p.purpose.clone();
        if let Some(v) = &p.version {
            n.tags.push(format!("version {v}"));
        }
        n.fields = demands.get(p.id.as_str()).cloned().unwrap_or_default();
        nodes.push(n);
        for d in &p.owns_domain {
            edges.push(owns(&p.id, d, "owns-domain", "owns"));
        }
        for sid in &p.owns_system {
            edges.push(owns(&p.id, sid, "owns-system", "owns"));
        }
    }
}

/// §3.0.1 — a journey composes single-system flows across Translations.
fn push_journeys(g: &DomainGraph, nodes: &mut Vec<ViewNode>, edges: &mut Vec<RawEdge>) {
    for j in &g.journeys {
        nodes.push(node(&j.id, &j.label, "journey", DOMAIN, ""));
        for f in &j.composes_flow {
            edges.push(re(&j.id, f, "composes"));
        }
        for t in &j.crosses_via {
            edges.push(re(&j.id, t, "crosses"));
        }
    }
}

#[cfg(test)]
#[path = "viz_tests.rs"]
mod tests;
