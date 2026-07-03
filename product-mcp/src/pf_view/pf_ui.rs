//! Live projection of the §3.2 UI model into the shapes the UI views consume:
//! the page graph (§3.2.4), the UI-step spec sheets (§3.2.1), and the screen
//! contract (§3.2.1/§4.5). Derived from the graph's wireframe steps,
//! application roots, flows and AIOs. Sample projected data (the screen
//! renderer's fill) is not in the graph, so the contract's `scenario` is empty.

use product_core::pf::model::DomainGraph;
use product_core::pf::model_ui::{Flow, WireframeStep};
use serde_json::{json, Map, Value};

/// The flow (if any) a ui-step belongs to.
fn flow_of<'a>(g: &'a DomainGraph, step: &str) -> Option<&'a Flow> {
    g.flows.iter().find(|f| f.steps.iter().any(|s| s == step))
}

/// The WCAG criteria an AIO carries (inherited by steps that use it).
fn aio_wcag(g: &DomainGraph, aio: &str) -> Vec<String> {
    g.aios.iter().find(|a| a.id == aio).map(|a| a.must_satisfy.clone()).unwrap_or_default()
}

/// The ui-steps that belong to a system (via its flows).
fn steps_of_system<'a>(g: &'a DomainGraph, sys: &str) -> Vec<&'a WireframeStep> {
    let step_ids: std::collections::BTreeSet<&str> = g
        .flows
        .iter()
        .filter(|f| f.system.as_deref() == Some(sys))
        .flat_map(|f| f.steps.iter().map(|s| s.as_str()))
        .collect();
    g.wireframe_steps.iter().filter(|w| step_ids.contains(w.id.as_str())).collect()
}

// --- §3.2.4 the page graph -------------------------------------------------

pub fn project_page_graph(g: &DomainGraph) -> Value {
    let systems: Vec<Value> = g
        .systems
        .iter()
        .map(|sys| {
            let steps = steps_of_system(g, &sys.id);
            let root = sys.root.clone().unwrap_or_default();
            let pages: Vec<Value> = steps.iter().map(|u| json!({
                "id": u.id, "name": u.label,
                "flow": flow_of(g, &u.id).map(|f| f.id.clone()).unwrap_or_default(),
                "specced": true,
            })).collect();
            let mut edges: Vec<Value> = Vec::new();
            if let Some(ar) = g.application_roots.iter().find(|a| a.id == root) {
                for dest in &ar.navigates_from_root {
                    edges.push(json!({ "from": root, "to": dest }));
                }
            }
            for u in &steps {
                for t in &u.transitions_to {
                    edges.push(json!({ "from": u.id, "to": t }));
                }
            }
            let flows: Vec<Value> = g.flows.iter().filter(|f| f.system.as_deref() == Some(&sys.id))
                .map(|f| json!({ "id": f.id, "name": f.label, "entry": f.entry_page.clone().unwrap_or_default() })).collect();
            json!({ "id": sys.id, "name": sys.label, "root": root, "globalActions": [], "pages": pages, "edges": edges, "flows": flows })
        })
        .collect();
    json!({ "systems": systems })
}

// --- §3.2.1 the ui-step spec sheets ----------------------------------------

pub fn project_step_specs(g: &DomainGraph) -> Value {
    let mut m = Map::new();
    for u in &g.wireframe_steps {
        let mut inherited = Map::new();
        for s in &u.surfaces {
            for w in aio_wcag(g, &s.aio) { inherited.insert(w, json!(s.aio)); }
        }
        for o in &u.offers {
            for w in aio_wcag(g, &o.command) { inherited.insert(w, json!("action")); }
        }
        let transitions: Vec<Value> = u.transitions_to.iter().map(|t| json!({ "on": "", "to": t })).collect();
        m.insert(u.id.clone(), json!({
            "emphasis": u.intent.clone().unwrap_or_default(),
            "transitions": transitions,
            "inheritedWcag": Value::Object(inherited),
            "stepWcag": {},
            "intentReliance": 0,
        }));
    }
    Value::Object(m)
}

// --- §3.2.1/§4.5 the screen contract ---------------------------------------

pub fn project_contract(g: &DomainGraph) -> Value {
    let screens: Vec<Value> = g.wireframe_steps.iter().map(|u| screen_json(g, u)).collect();
    let start = g.wireframe_steps.first().map(|u| u.id.clone()).unwrap_or_default();
    let product = g.products.first().map(|p| p.label.clone()).unwrap_or_default();
    let flows: Vec<Value> = g.flows.iter().map(|f| json!({
        "id": f.id, "entry": f.entry_page.clone().unwrap_or_default(),
        "pages": f.steps.iter().filter(|s| g.wireframe_steps.iter().any(|w| &w.id == *s)).collect::<Vec<_>>(),
    })).collect();
    json!({
        "contract_version": "live", "title": product, "context": {}, "locale": "en",
        "content_store": {}, "start": start,
        "root": { "destinations": [], "global_actions": [] },
        "flows": flows, "screens": screens,
        "scenario": { "given": [], "projected": {} },
    })
}

fn screen_json(g: &DomainGraph, u: &WireframeStep) -> Value {
    let mut states: Vec<String> = vec!["present".to_string()];
    let mut meanings = Map::new();
    for s in &u.state_meanings {
        if !states.contains(&s.state) { states.push(s.state.clone()); }
        if let Some(m) = &s.meaning { meanings.insert(s.state.clone(), json!(m)); }
    }
    let mut elements: Vec<Value> = u.surfaces.iter().map(|s| json!({
        "aio": s.aio, "role": "", "binds": s.projection, "wcag": aio_wcag(g, &s.aio),
    })).collect();
    elements.extend(u.offers.iter().map(|o| json!({
        "aio": o.aio, "role": "", "issues": o.command, "wcag": aio_wcag(g, &o.command),
    })));
    json!({
        "id": u.id, "name": u.label, "intent": u.intent.clone().unwrap_or_default(),
        "content": {}, "projection": u.surfaces.first().map(|s| s.projection.clone()).unwrap_or_default(),
        "state_space": states, "state_meanings": Value::Object(meanings),
        "elements": elements,
    })
}
