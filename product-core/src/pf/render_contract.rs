//! Render contract (PREVIEW) — a read-only projection of the What page-graph a
//! renderer consumes (`preview/render-contract.schema.md`, FT-146).
//!
//! Derived, never authored: the application root, a flow's screens (Abstract UI
//! typed by AIO with inherited WCAG obligations), and resolved content. The
//! `render(contract, manifest?)` other half is §11 (FT-141); this is "what there
//! is to render", emitted at generic wireframe fidelity.

use super::ids::CORE_AIO_CRITERIA;
use super::model::DomainGraph;

#[derive(Debug, serde::Serialize)]
pub struct RenderContract {
    pub contract_version: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub content_store: std::collections::BTreeMap<String, ContentValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Root>,
    pub flow: FlowOut,
    pub screens: Vec<Screen>,
}

#[derive(Debug, serde::Serialize)]
pub struct ContentValue {
    pub role: String,
    pub value: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Root {
    pub destinations: Vec<Destination>,
}

#[derive(Debug, serde::Serialize)]
pub struct Destination {
    pub to: String,
    pub label: String,
}

#[derive(Debug, serde::Serialize)]
pub struct FlowOut {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    pub pages: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Screen {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub state_space: Vec<String>,
    pub elements: Vec<Element>,
}

#[derive(Debug, serde::Serialize)]
pub struct Element {
    pub aio: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitions_to: Option<String>,
    pub wcag: Vec<String>,
}

/// WCAG criteria an AIO inherits (§3.2.3), plus the step's own `must_satisfy`.
fn aio_wcag(aio: &str, extra: &[String]) -> Vec<String> {
    let mut out: Vec<String> = CORE_AIO_CRITERIA
        .iter()
        .find(|(a, _)| *a == aio)
        .map(|(_, cs)| cs.iter().map(|c| c.to_string()).collect())
        .unwrap_or_default();
    out.extend(extra.iter().cloned());
    out.sort();
    out.dedup();
    out
}

/// Project the render contract for `flow_id`. Errors name the missing element.
pub fn build(
    graph: &DomainGraph,
    flow_id: &str,
    title: &str,
    context: Option<String>,
    locale: Option<String>,
) -> Result<RenderContract, String> {
    let flow = graph
        .flows
        .iter()
        .find(|f| f.id == flow_id)
        .ok_or_else(|| format!("no flow with id {flow_id:?} in the graph"))?;

    let root = graph.application_roots.first().map(|r| Root {
        destinations: r.navigates_from_root.iter()
            .map(|to| Destination { to: to.clone(), label: to.clone() })
            .collect(),
    });

    let screens = flow.steps.iter()
        .filter_map(|step_id| graph.wireframe_steps.iter().find(|s| &s.id == step_id))
        .map(|step| build_screen(graph, step))
        .collect();

    let content_store = locale.as_ref().map(|loc| resolve_content(graph, flow, loc)).unwrap_or_default();

    Ok(RenderContract {
        contract_version: "preview-0",
        title: title.to_string(),
        context,
        locale,
        content_store,
        root,
        flow: FlowOut { id: flow.id.clone(), entry: flow.entry_page.clone(), pages: flow.steps.clone() },
        screens,
    })
}

fn build_screen(graph: &DomainGraph, step: &super::model::WireframeStep) -> Screen {
    let mut elements = Vec::new();
    for s in &step.surfaces {
        elements.push(Element {
            aio: s.aio.clone(),
            role: "display".into(),
            binds: Some(s.projection.clone()),
            issues: None,
            transitions_to: None,
            wcag: aio_wcag(&s.aio, &step.must_satisfy),
        });
    }
    for o in &step.offers {
        elements.push(Element {
            aio: o.aio.clone(),
            role: "control".into(),
            binds: None,
            issues: Some(o.command.clone()),
            transitions_to: step.transitions_to.first().cloned(),
            wcag: aio_wcag(&o.aio, &step.must_satisfy),
        });
    }
    let projection = step.surfaces.first().map(|s| s.projection.clone());
    let state_space = projection.as_ref()
        .and_then(|p| graph.read_models.iter().find(|rm| &rm.id == p))
        .map(|rm| rm.states.clone())
        .unwrap_or_default();
    Screen { id: step.id.clone(), intent: step.intent.clone(), projection, state_space, elements }
}

/// Resolve every content key the flow's steps reference, in `locale`.
fn resolve_content(
    graph: &DomainGraph,
    flow: &super::model::Flow,
    locale: &str,
) -> std::collections::BTreeMap<String, ContentValue> {
    let mut out = std::collections::BTreeMap::new();
    let steps = flow.steps.iter().filter_map(|id| graph.wireframe_steps.iter().find(|s| &s.id == id));
    for step in steps {
        for r in &step.content_refs {
            if let Some(v) = lookup(graph, &r.key, locale) {
                out.insert(r.key.clone(), ContentValue { role: r.role.clone(), value: v });
            }
        }
    }
    out
}

fn lookup(graph: &DomainGraph, key: &str, locale: &str) -> Option<String> {
    graph.content_stores.iter()
        .flat_map(|s| s.resolutions.iter())
        .find(|r| r.key == key && r.locale == locale)
        .map(|r| r.value.clone())
}

#[cfg(test)]
#[path = "render_contract_tests.rs"]
mod tests;
