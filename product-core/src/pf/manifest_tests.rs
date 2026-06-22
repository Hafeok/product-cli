//! Unit tests for the §11.3 design-system manifest profile (canonical YAML).

use super::*;
use crate::pf::model::{ContextOfUse, DomainGraph, Offer, Surface, WireframeStep};

const WHOLE: &str = r#"
design_system:
  id: "acme"
  version: "1.0"
  wcag_target: "2.2-AA"
  contexts_supported:
    form_factor: [phone, tablet]
    modality: [touch, pointer]
  components:
    - id: searchable-list
      tokens: [color.accent]
      satisfies:
        - { criterion: "1.3.1", level: A, via: machine }
    - id: primary-button
      tokens: [color.accent]
      satisfies:
        - { criterion: "2.5.8", level: AA, via: machine }
  reification:
    - aio: single-select
      when: { form_factor: phone, options: many }
      cio: searchable-list
      rationale: "thumb-reachable"
    - aio: trigger-action
      when: { emphasis: primary }
      cio: primary-button
  tokens:
    - { id: color.accent, type: color }
"#;

#[test]
fn whole_manifest_validates() {
    let m = parse_ds(WHOLE).expect("parses");
    assert!(validate_ds(&m).is_empty(), "should be whole: {:?}", validate_ds(&m));
}

#[test]
fn dangling_reification_cio_fails() {
    let m = parse_ds(&WHOLE.replace("cio: searchable-list", "cio: ghost-cio")).expect("parses");
    let f = validate_ds(&m);
    assert!(f.iter().any(|s| s.contains("ghost-cio") && s.contains("absent")), "{f:?}");
}

#[test]
fn undeclared_token_and_fake_criterion_fail() {
    let src = WHOLE
        .replace("tokens: [color.accent]\n      satisfies:\n        - { criterion: \"1.3.1\"",
                 "tokens: [color.ghost]\n      satisfies:\n        - { criterion: \"9.9.9\"");
    let m = parse_ds(&src).expect("parses");
    let f = validate_ds(&m);
    assert!(f.iter().any(|s| s.contains("color.ghost")), "token: {f:?}");
    assert!(f.iter().any(|s| s.contains("9.9.9")), "criterion: {f:?}");
}

fn step_using(aio: &str) -> WireframeStep {
    WireframeStep {
        id: "S".into(), label: "S".into(),
        surfaces: vec![Surface { projection: "P".into(), aio: aio.into() }],
        offers: vec![Offer { command: "C".into(), aio: "trigger-action".into() }],
        ..Default::default()
    }
}

#[test]
fn coupling_covers_referenced_aios_over_declared_contexts() {
    let m = parse_ds(WHOLE).expect("parses");
    let mut g = DomainGraph::default();
    g.contexts_of_use.push(ContextOfUse {
        id: "phone".into(), label: "P".into(),
        dimension: Some("form_factor".into()), value: Some("phone".into()),
    });
    g.wireframe_steps.push(step_using("single-select"));
    // single-select reifies on phone, trigger-action (emphasis predicate) is a wildcard on form_factor.
    assert!(couple_ds(&m, &g).is_empty(), "should couple: {:?}", couple_ds(&m, &g));
    // A step referencing an AIO with no rule → non-conforming for that context.
    g.wireframe_steps.push(step_using("date-entry"));
    let f = couple_ds(&m, &g);
    assert!(f.iter().any(|s| s.contains("date-entry") && s.contains("phone")), "{f:?}");
}
