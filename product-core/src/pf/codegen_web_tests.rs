//! Unit tests for the web codegen backend — on-system pages, tokens not literals.

use super::*;
use crate::pf::model_ui::{Flow, Offer, Surface, WireframeStep};

const MANIFEST: &str = r##"
design_system:
  id: acme
  version: "1.0"
  themes: [light]
  components:
    - id: value-block
      tokens: [color.fg]
    - id: primary-button
      tokens: [color.accent, space.inset]
  reification:
    - { aio: display-value, when: {}, cio: value-block }
    - { aio: trigger-action, when: {}, cio: primary-button }
  tokens:
    - { id: color.fg, type: color, values: { light: "#111" } }
    - { id: color.accent, type: color, values: { light: "#36f" } }
    - { id: space.inset, type: dimension, values: { light: "12px" } }
"##;

fn opts_with_ds() -> ReifyOptions {
    let m = crate::pf::manifest::parse_ds(MANIFEST).expect("parse");
    ReifyOptions {
        product: "shop".into(),
        namespace: "Shop".into(),
        what_version: "1.0".into(),
        oracle_only: true,
        design_system: Some(crate::pf::reify_ds::DsSpec::from_source(m, MANIFEST)),
    }
}

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.wireframe_steps = vec![
        WireframeStep {
            id: "review-order".into(),
            label: "Review order".into(),
            intent: Some("Confirm before paying".into()),
            surfaces: vec![Surface { projection: "OrderSummary".into(), aio: "display-value".into() }],
            offers: vec![Offer { command: "PlaceOrder".into(), aio: "trigger-action".into() }],
            transitions_to: vec!["confirmation".into()],
            ..Default::default()
        },
        WireframeStep { id: "confirmation".into(), label: "Confirmation".into(), ..Default::default() },
    ];
    g.flows = vec![Flow {
        id: "checkout".into(),
        label: "Checkout".into(),
        steps: vec!["review-order".into(), "confirmation".into()],
        ..Default::default()
    }];
    g
}

fn find<'a>(plan: &'a ReifyPlan, path: &str) -> &'a str {
    &plan.files.iter().find(|f| f.path == path).unwrap_or_else(|| panic!("{path} missing")).content
}

#[test]
fn plan_emits_pages_tokens_and_pinned_json() {
    let plan = plan_web(&graph(), &[], &[], &opts_with_ds()).expect("plan");
    let page = find(&plan, "pages/review-order.g.html");
    assert!(page.contains("class=\"cio-value-block\"") && page.contains("data-projection=\"OrderSummary\""), "{page}");
    assert!(page.contains("class=\"cio-primary-button\"") && page.contains("data-command=\"PlaceOrder\""), "{page}");
    assert!(page.contains("href=\"confirmation.g.html\""), "nav from transitions_to:\n{page}");
    let css = find(&plan, "ds.g.css");
    assert!(css.contains(".cio-primary-button") && css.contains("background-color: var(--color-accent);"), "{css}");
    assert!(css.contains("padding: var(--space-inset);"), "{css}");
    assert!(!css.contains("#36f"), "literals stay in tokens.g.css only:\n{css}");
    let tokens = find(&plan, "tokens.g.css");
    assert!(tokens.contains("--color-accent: #36f;"), "{tokens}");
    let index = find(&plan, "index.g.html");
    assert!(index.contains("Checkout") && index.contains("pages/review-order.g.html"), "{index}");
    let prov = find(&plan, "provenance.g.json");
    assert!(prov.contains("\"design_system\"") && prov.contains("\"id\": \"acme\""), "{prov}");
}

#[test]
fn plan_without_bound_design_system_is_rejected() {
    let opts = ReifyOptions { design_system: None, ..opts_with_ds() };
    let e = match plan_web(&graph(), &[], &[], &opts) { Err(e) => e, Ok(_) => panic!("expected error") };
    assert!(format!("{e}").contains("design-system bind"), "{e}");
}

#[test]
fn coverage_gap_fails_the_web_plan() {
    let mut g = graph();
    g.wireframe_steps[0].offers[0].aio = "range-select".into();
    let e = match plan_web(&g, &[], &[], &opts_with_ds()) { Err(e) => e, Ok(_) => panic!("expected error") };
    assert!(format!("{e}").contains("cannot realise"), "{e}");
}
