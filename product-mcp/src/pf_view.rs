//! Live `window.PF` projection — the graph, projected into the shape the 1.7.0
//! explorer UI consumes, served at `/api/pf`.
//!
//! The explorer (`assets/ui/`) reads a global `window.PF` object that its bundled
//! `data*.js` builds as a curated demo. This module projects the **live** graph
//! (`DomainGraph` + the `.product/` delivery + How artifacts) into that same
//! shape for the fields the graph genuinely carries — product/systems/domains/
//! journeys (§3.0), the domain ER (§3.1), event-model flows (§3.2), deciders
//! (§3.3), delivery + versions (§7), and the How's blueprint / DeployableUnits /
//! why-cascade (§4). The UI merges these live fields over its demo defaults, so
//! narrative fields the framework graph does not capture keep their sample value.
//!
//! Everything is derived per request (no cache); the client re-fetches on the
//! `/api/events` SSE, exactly as the legacy view does.

use std::path::{Path, PathBuf};

use product_core::pf::deployable_unit::DeployableUnit;
use product_core::pf::model::DomainGraph;
use serde_json::{json, Map, Value};

mod conformance;
mod pf_build;
mod pf_flows;
mod pf_how;
mod pf_repo;
mod pf_ui;

/// The `.product` base for a product's How/Delivery/Build artifacts: a
/// per-product `.product/products/<product>/` if it exists (so acme's blueprint,
/// deliverables, DeployableUnits, … are its own), else the shared `.product/`
/// (back-compat for the self-hosted product-cli, whose artifacts live at root).
pub(crate) fn scoped_base(repo_root: &Path, product: &str) -> PathBuf {
    let scoped = repo_root.join(".product").join("products").join(product);
    if scoped.is_dir() { scoped } else { repo_root.join(".product") }
}

/// Read + parse every `*.yaml` in `<base>/<dir>` through `parse`, sorted by
/// filename. `base` is the scoped `.product` dir. Shared by the projectors.
pub(crate) fn load_all<T>(
    base: &Path,
    dir: &str,
    parse: impl Fn(&str) -> product_core::error::Result<T>,
) -> Vec<T> {
    let d = base.join(dir);
    let mut paths: Vec<_> = match std::fs::read_dir(&d) {
        Ok(it) => it.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml")).collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths.iter().filter_map(|p| std::fs::read_to_string(p).ok()).filter_map(|t| parse(&t).ok()).collect()
}

/// Build the live `window.PF` field map for `product` from the graph + its
/// (per-product-scoped) `.product/` artifacts.
pub fn build_pf_view(graph: &DomainGraph, repo_root: &Path, product: &str) -> Value {
    let base = scoped_base(repo_root, product);
    let conf = conformance::Conformance::compute(graph, &base);
    let mut out = Map::new();
    out.insert("product".into(), project_product(graph, &conf));
    out.insert("domains".into(), project_domains(graph, &conf));
    out.insert("systems".into(), project_systems(graph, &conf));
    out.insert("journeys".into(), project_journeys(graph));
    out.insert("domain".into(), project_domain_er(graph));
    out.insert("aios".into(), project_aios(graph));
    out.insert("wcag".into(), project_wcag(graph));
    out.insert("pageGraph".into(), pf_ui::project_page_graph(graph));
    out.insert("stepSpecs".into(), pf_ui::project_step_specs(graph));
    out.insert("contract".into(), pf_ui::project_contract(graph));
    out.insert("flows".into(), pf_flows::project_flows(graph, &conf));
    out.insert("deciders".into(), pf_how::project_deciders(&base, &conf));
    out.insert("projectors".into(), pf_how::project_projectors(&base, &conf));
    out.insert("scenarios".into(), pf_how::project_scenarios(&base, &conf));
    out.insert("delivery".into(), pf_how::project_delivery(graph, &base, &conf));
    out.insert("how".into(), pf_how::project_how(graph, &base, &conf));
    out.insert("workUnits".into(), pf_build::project_work_units(&base));
    out.insert("repoTree".into(), pf_repo::project_repo_tree(repo_root, &base, product));
    // Markers for the UI: live-connected, the current product, and every product
    // available (the product picker shows when there is more than one).
    out.insert("_live".into(), json!(true));
    out.insert("_product".into(), json!(product));
    out.insert("_products".into(), json!(list_products(repo_root)));
    Value::Object(out)
}

/// Every product with a captured What graph (dir names under author-domain/).
pub(crate) fn list_products(repo_root: &Path) -> Vec<String> {
    let dir = repo_root.join(".product").join("author-domain");
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(it) => it.flatten().filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok()).collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

// --- §3.0 product / systems / domains / journeys ---------------------------

fn project_product(g: &DomainGraph, conf: &conformance::Conformance) -> Value {
    let Some(p) = g.products.first() else { return Value::Null };
    let direction = p.version.as_deref().map(|v| format!("What {v}")).unwrap_or_default();
    let quality = g
        .quality_demands
        .iter()
        .find(|q| q.kind == "architectural")
        .map(|q| q.bound.clone())
        .unwrap_or_default();
    json!({
        "id": p.id,
        "name": p.label,
        "purpose": p.purpose,
        "direction": direction,
        "quality": quality,
        "ownsDomains": p.owns_domain,
        "ownsSystems": p.owns_system,
        "conformance": conf.level(&p.id),
    })
}

fn project_domains(g: &DomainGraph, conf: &conformance::Conformance) -> Value {
    let v: Vec<Value> = g
        .contexts
        .iter()
        .map(|c| {
            // language: the context's glossary, else the labels of its entities.
            let mut language: Vec<String> = c.glossary.clone();
            if language.is_empty() {
                language = g.entities.iter().filter(|e| e.context == c.id).map(|e| e.label.clone()).collect();
            }
            json!({
                "id": c.id,
                "name": c.label,
                "sub": "bounded context · §3.1",
                "language": language,
                "conformance": conf.level(&c.id),
            })
        })
        .collect();
    Value::Array(v)
}

fn project_systems(g: &DomainGraph, conf: &conformance::Conformance) -> Value {
    let v: Vec<Value> = g
        .systems
        .iter()
        .map(|s| {
            let demands: Vec<String> = g
                .quality_demands
                .iter()
                .filter(|q| q.scopes == s.id)
                .map(|q| q.bound.clone())
                .collect();
            let flows: Vec<String> =
                g.flows.iter().filter(|f| f.system.as_deref() == Some(&s.id)).map(|f| f.id.clone()).collect();
            json!({
                "id": s.id,
                "name": s.label,
                "kind": s.kind,
                "purpose": s.purpose,
                "cls": s.target_classes.first().cloned().unwrap_or_default(),
                "platforms": s.target_platforms,
                "references": s.references_domain,
                "demands": demands,
                "flows": flows,
                "conformance": conf.level(&s.id),
            })
        })
        .collect();
    Value::Array(v)
}

fn project_journeys(g: &DomainGraph) -> Value {
    let v: Vec<Value> = g
        .journeys
        .iter()
        .map(|j| {
            let flow_system = |fid: &str| -> Option<String> {
                g.flows.iter().find(|f| f.id == fid).and_then(|f| f.system.clone())
            };
            let from_flow = j.composes_flow.first().cloned().unwrap_or_default();
            let to_flow = j.composes_flow.get(1).cloned().unwrap_or_default();
            json!({
                "id": j.id,
                "name": j.label,
                "from": { "system": flow_system(&from_flow).unwrap_or_default(), "flow": from_flow, "label": "" },
                "to": { "system": flow_system(&to_flow).unwrap_or_default(), "flow": to_flow, "label": "" },
                "translation": j.crosses_via.join(", "),
            })
        })
        .collect();
    Value::Array(v)
}

// --- §3.1 the domain ER graph (the primary context) ------------------------

/// The ER graph for the primary bounded context — the one owning the most
/// entities (the UI's Domain view renders a single `PF.domain`).
fn project_domain_er(g: &DomainGraph) -> Value {
    let Some(ctx) = primary_context(g) else { return json!({"contextId": "", "nodes": [], "edges": []}) };
    let nodes = domain_nodes(g, &ctx);
    let in_ctx: std::collections::HashSet<&str> =
        nodes.iter().filter_map(|n| n.get("id").and_then(|v| v.as_str())).collect();
    let mut edges: Vec<Value> = Vec::new();
    for r in &g.relations {
        if in_ctx.contains(r.from.as_str()) || in_ctx.contains(r.to.as_str()) {
            edges.push(json!({ "from": r.from, "to": r.to, "label": r.label.clone().unwrap_or_default(), "card": r.cardinality }));
        }
    }
    for i in g.invariants.iter().filter(|i| i.context.as_deref() == Some(&ctx)) {
        if let Some(target) = &i.applies_to {
            edges.push(json!({ "from": i.id, "to": target, "label": "governs", "kind": "inv" }));
        }
    }
    json!({ "contextId": ctx, "nodes": nodes, "edges": edges })
}

/// The §3.1 nodes (entities, value objects, invariants) of one bounded context.
fn domain_nodes(g: &DomainGraph, ctx: &str) -> Vec<Value> {
    let mut nodes: Vec<Value> = Vec::new();
    for e in g.entities.iter().filter(|e| e.context == ctx) {
        let fields: Vec<String> = e
            .attributes
            .iter()
            .map(|a| match &a.ty { Some(t) => format!("{}: {}", a.name, t), None => a.name.clone() })
            .collect();
        let kind = if e.is_aggregate_root { "aggregate" } else { "entity" };
        nodes.push(json!({ "id": e.id, "kind": kind, "label": e.label, "sub": kind, "fields": fields }));
    }
    for vo in g.value_objects.iter().filter(|v| v.context == ctx) {
        nodes.push(json!({
            "id": vo.id, "kind": "value-object", "label": vo.label, "sub": "value object",
            "fields": vo.definition.clone().map(|d| vec![d]).unwrap_or_default(),
        }));
    }
    for i in g.invariants.iter().filter(|i| i.context.as_deref() == Some(ctx)) {
        nodes.push(json!({ "id": i.id, "kind": "invariant", "label": i.id, "sub": "invariant", "fields": [i.statement.clone()] }));
    }
    nodes
}

// --- §3.2.2 AIOs + §3.2.3 WCAG ---------------------------------------------

/// The AIO catalog: each interaction object with its meaning, inherited WCAG,
/// and per-context reification (from the §4.5 reification rules).
fn project_aios(g: &DomainGraph) -> Value {
    let cio_label = |id: &str| g.cios.iter().find(|c| c.id == id).and_then(|c| c.label.clone()).unwrap_or_else(|| id.to_string());
    let ctx_label = |id: &str| g.contexts_of_use.iter().find(|c| c.id == id).map(|c| c.label.clone()).unwrap_or_else(|| id.to_string());
    let v: Vec<Value> = g
        .aios
        .iter()
        .map(|a| {
            let mut reify = Map::new();
            for r in g.reification_rules.iter().filter(|r| r.aio == a.id) {
                reify.insert(ctx_label(&r.context), json!(cio_label(&r.cio)));
            }
            json!({
                "id": a.id, "means": a.means.clone().unwrap_or_default(), "typedOver": "",
                "wcag": a.must_satisfy, "reify": Value::Object(reify),
            })
        })
        .collect();
    Value::Array(v)
}

/// The WCAG criteria as a `{ id → {name, level, vtype} }` map (§3.2.3).
fn project_wcag(g: &DomainGraph) -> Value {
    let mut m = Map::new();
    for w in &g.wcag_criteria {
        m.insert(w.id.clone(), json!({
            "name": w.label.clone().unwrap_or_else(|| w.id.clone()),
            "level": w.level.clone().unwrap_or_default(),
            "vtype": w.verification.clone().unwrap_or_else(|| "machine".to_string()),
        }));
    }
    Value::Object(m)
}

/// The bounded context owning the most entities (falls back to the first).
fn primary_context(g: &DomainGraph) -> Option<String> {
    g.contexts
        .iter()
        .max_by_key(|c| g.entities.iter().filter(|e| e.context == c.id).count())
        .map(|c| c.id.clone())
}

/// Load the DeployableUnits under `<base>/deployable-units/`.
pub(crate) fn load_deployable_units(base: &Path) -> Vec<DeployableUnit> {
    product_core::pf::deployable_unit::load_dir(&base.join("deployable-units"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_core::pf::model::{BoundedContext, Command, Entity, Event};
    use product_core::pf::model_product::{Journey, Product};
    use product_core::pf::model_ui::{Flow, System};

    fn sample() -> DomainGraph {
        let mut g = DomainGraph::default();
        g.products.push(Product {
            id: "acme".into(), label: "Acme".into(), purpose: "sell".into(),
            owns_domain: vec!["ordering".into()], owns_system: vec!["shop".into()],
            version: Some("1.1".into()),
        });
        g.contexts.push(BoundedContext { id: "ordering".into(), label: "Ordering".into(), glossary: vec!["Order".into()], ..Default::default() });
        g.entities.push(Entity { id: "order".into(), label: "Order".into(), context: "ordering".into(), is_aggregate_root: true, ..Default::default() });
        g.systems.push(System { id: "shop".into(), label: "Shop".into(), kind: "application".into(), purpose: "buy".into(), references_domain: vec!["ordering".into()], ..Default::default() });
        g.flows.push(Flow { id: "flow-checkout".into(), label: "Checkout".into(), steps: vec!["t".into(), "cmd".into(), "ev".into()], system: Some("shop".into()), ..Default::default() });
        g.commands.push(Command { id: "cmd".into(), label: "Place".into(), context: "ordering".into(), targets: "order".into(), emits: vec!["ev".into()] });
        g.events.push(Event { id: "ev".into(), label: "Placed".into(), context: "ordering".into(), changes: "order".into() });
        g.journeys.push(Journey { id: "j1".into(), label: "Order to fulfil".into(), product: "acme".into(), composes_flow: vec!["flow-checkout".into()], crosses_via: vec!["tr".into()] });
        g
    }

    #[test]
    fn projects_the_live_window_pf_shape() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let pf = build_pf_view(&sample(), tmp.path(), "acme");
        // The top-level keys the UI reads are all present.
        for k in ["product", "domains", "systems", "journeys", "domain", "flows", "deciders", "delivery", "how"] {
            assert!(pf.get(k).is_some(), "missing PF field: {k}");
        }
        assert_eq!(pf["product"]["name"], "Acme");
        assert_eq!(pf["product"]["ownsSystems"][0], "shop");
        assert_eq!(pf["systems"][0]["id"], "shop");
        assert_eq!(pf["systems"][0]["flows"][0], "flow-checkout");
        assert_eq!(pf["domains"][0]["name"], "Ordering");
        assert_eq!(pf["_live"], true);
    }

    #[test]
    fn flow_layout_assigns_lanes_and_causal_columns() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let pf = build_pf_view(&sample(), tmp.path(), "acme");
        let flow = &pf["flows"]["flow-checkout"];
        assert_eq!(flow["system"], "shop");
        // trigger→command→event spine puts the event to the right of the command.
        let nodes = flow["nodes"].as_array().expect("nodes");
        let col = |id: &str| nodes.iter().find(|n| n["id"] == id).and_then(|n| n["col"].as_u64());
        assert!(col("cmd") < col("ev"), "event should be causally after its command");
    }
}
