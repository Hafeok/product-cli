//! Unit tests for the design-system reify stage (§11.2 as a plan-time gate).

use super::*;
use crate::pf::model::DomainGraph;
use crate::pf::model_ui::{ContextOfUse, Offer, Surface, WireframeStep};

const MANIFEST: &str = r##"
design_system:
  id: acme
  version: "1.0"
  themes: [light, dark]
  components:
    - id: searchable-list
      tokens: [color.fg]
      satisfies:
        - { criterion: "1.3.1", level: A, via: machine }
    - id: segmented-control
      tokens: [color.fg]
    - id: primary-button
      tokens: [color.accent]
      satisfies:
        - { criterion: "2.5.8", level: AA, via: machine }
  reification:
    - { aio: single-select, when: { form_factor: phone }, cio: searchable-list }
    - { aio: single-select, when: { form_factor: tablet }, cio: segmented-control }
    - { aio: trigger-action, when: {}, cio: primary-button }
  tokens:
    - { id: color.fg, type: color, values: { light: "#111", dark: "#eee" } }
    - { id: color.accent, type: color, values: { light: "#36f", dark: "#69f" } }
"##;

fn spec() -> DsSpec {
    let m = crate::pf::manifest::parse_ds(MANIFEST).expect("parse");
    DsSpec::from_source(m, MANIFEST)
}

fn ctx(id: &str, dim: &str, value: &str) -> ContextOfUse {
    ContextOfUse {
        id: id.to_string(),
        label: id.to_string(),
        dimension: Some(dim.to_string()),
        value: Some(value.to_string()),
    }
}

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts_of_use = vec![ctx("phone", "form_factor", "phone"), ctx("tablet", "form_factor", "tablet")];
    g.wireframe_steps = vec![WireframeStep {
        id: "pick-shipping".to_string(),
        label: "Pick shipping".to_string(),
        surfaces: vec![Surface { projection: "ShippingOptions".to_string(), aio: "single-select".to_string() }],
        offers: vec![Offer { command: "ConfirmShipping".to_string(), aio: "trigger-action".to_string() }],
        ..Default::default()
    }];
    g
}

#[test]
fn resolves_per_context_component_maps() {
    let r = resolve(&spec(), &graph()).expect("resolve");
    assert_eq!(r.id, "acme");
    let comps = &r.screens["pick-shipping"];
    // single-select × {phone, tablet} + trigger-action × {phone, tablet}.
    assert_eq!(comps.len(), 4);
    let phone_select = comps.iter().find(|c| c.aio == "single-select" && c.context == "phone").expect("phone");
    assert_eq!(phone_select.cio, "searchable-list");
    assert_eq!(phone_select.binds, "ShippingOptions");
    assert_eq!(phone_select.wcag, vec!["1.3.1"]);
    let tablet_select = comps.iter().find(|c| c.aio == "single-select" && c.context == "tablet").expect("tablet");
    assert_eq!(tablet_select.cio, "segmented-control");
    let offer = comps.iter().find(|c| c.role == "offer" && c.context == "phone").expect("offer");
    assert_eq!(offer.cio, "primary-button");
    assert_eq!(offer.tokens, vec!["color.accent"]);
}

#[test]
fn coverage_gap_fails_the_plan() {
    let mut g = graph();
    g.wireframe_steps[0].surfaces[0].aio = "range-select".to_string(); // no rule
    let e = resolve(&spec(), &g).expect_err("gap");
    let msg = format!("{e}");
    assert!(msg.contains("range-select") && msg.contains("cannot realise"), "{msg}");
}

#[test]
fn unwhole_manifest_fails_the_plan() {
    let mut s = spec();
    s.manifest.design_system.reification[0].cio = "ghost".to_string();
    let e = resolve(&s, &graph()).expect_err("unwhole");
    assert!(format!("{e}").contains("ghost"), "{e}");
}

#[test]
fn graph_without_contexts_resolves_wildcard_rules_only() {
    let mut g = graph();
    g.contexts_of_use.clear();
    g.wireframe_steps[0].surfaces.clear(); // single-select has no wildcard rule
    let r = resolve(&spec(), &g).expect("resolve");
    let comps = &r.screens["pick-shipping"];
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].context, "any");
    assert_eq!(comps[0].cio, "primary-button");
}

#[test]
fn ds_json_pins_both_hashes_and_carries_token_values() {
    let s = spec();
    let r = resolve(&s, &graph()).expect("resolve");
    let json = ds_json(&r, &s, "abc123");
    let v: serde_json::Value = serde_json::from_str(&json).expect("json");
    assert_eq!(v["design_system"]["id"], "acme");
    assert_eq!(v["graph_hash"], "sha256:abc123");
    assert!(v["design_system"]["hash"].as_str().unwrap_or_default().starts_with("sha256:"));
    assert_eq!(v["tokens"][0]["values"]["dark"], "#eee");
    assert_eq!(v["screens"]["pick-shipping"][0]["cio"], "searchable-list");
}

#[test]
fn tokens_css_puts_first_theme_on_root_and_others_behind_data_theme() {
    let css = tokens_css(&spec());
    assert!(css.contains(":root {"), "{css}");
    assert!(css.contains("--color-accent: #36f;"), "{css}");
    assert!(css.contains("[data-theme=\"dark\"]"), "{css}");
    assert!(css.contains("--color-fg: #eee;"), "{css}");
}

#[test]
fn themeless_tokens_render_as_valueless_declarations() {
    let mut s = spec();
    s.manifest.design_system.themes.clear();
    let css = tokens_css(&s);
    assert!(css.contains("--color-fg: initial;"), "{css}");
}

#[test]
fn catalog_cs_bakes_the_resolutions() {
    let s = spec();
    let r = resolve(&s, &graph()).expect("resolve");
    let cs = catalog_cs("// hdr\n", "Shop", &r);
    assert!(cs.contains("interface IDesignSystemProvider"), "{cs}");
    assert!(cs.contains("new(\"pick-shipping\", \"single-select\", \"ShippingOptions\", \"phone\", \"searchable-list\")"), "{cs}");
}
