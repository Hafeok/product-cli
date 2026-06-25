//! Unit tests for the render-contract projection.

use super::*;
use crate::pf::model::*;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.read_models.push(ReadModel { id: "Cart".into(), label: "Cart".into(), projects: vec!["O".into()], states: vec!["empty".into(), "present".into()] });
    g.application_roots.push(ApplicationRoot { id: "root".into(), label: Some("Root".into()), navigates_from_root: vec!["Review".into()] });
    g.wireframe_steps.push(WireframeStep {
        id: "Review".into(), label: "Review".into(), intent: Some("Confirm before paying".into()),
        surfaces: vec![Surface { projection: "Cart".into(), aio: "display-collection".into() }],
        offers: vec![Offer { command: "Pay".into(), aio: "trigger-action".into() }],
        transitions_to: vec!["Done".into()],
        content_refs: vec![ContentRef { key: "cart.empty".into(), role: "empty-message".into() }],
        ..Default::default()
    });
    g.flows.push(Flow { id: "checkout".into(), label: "Checkout".into(), steps: vec!["Review".into()], entry_page: Some("Review".into()), ..Default::default() });
    g
}

#[test]
fn projects_root_flow_and_screens() {
    let c = build(&graph(), "checkout", "Shop", None, None).expect("builds");
    assert_eq!(c.contract_version, "preview-0");
    assert_eq!(c.flow.entry.as_deref(), Some("Review"));
    assert_eq!(c.root.as_ref().unwrap().destinations[0].to, "Review");
    let screen = &c.screens[0];
    assert_eq!(screen.projection.as_deref(), Some("Cart"));
    assert_eq!(screen.state_space, vec!["empty", "present"]);
    // a display element + a control element, each AIO-typed.
    assert!(screen.elements.iter().any(|e| e.role == "display" && e.aio == "display-collection"));
    let control = screen.elements.iter().find(|e| e.role == "control").unwrap();
    assert_eq!(control.issues.as_deref(), Some("Pay"));
    assert_eq!(control.transitions_to.as_deref(), Some("Done"));
}

#[test]
fn resolves_content_for_locale() {
    let mut g = graph();
    g.content_stores.push(ContentStore {
        id: "cs".into(), locales: vec!["en".into()],
        resolutions: vec![Resolution { key: "cart.empty".into(), locale: "en".into(), value: "Empty".into() }],
        ..Default::default()
    });
    let c = build(&g, "checkout", "Shop", None, Some("en".into())).expect("builds");
    assert_eq!(c.content_store.get("cart.empty").map(|v| v.value.as_str()), Some("Empty"));
}

#[test]
fn unknown_flow_is_named() {
    let err = build(&graph(), "ghost", "Shop", None, None).unwrap_err();
    assert!(err.contains("ghost"), "{err}");
}
