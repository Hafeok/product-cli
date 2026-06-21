//! Unit tests for the §4.5 reification checks.

use super::*;
use crate::pf::model::*;

fn graph_with_step_and_contexts() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts_of_use.push(ContextOfUse { id: "phone".into(), label: "Phone".into(), ..Default::default() });
    g.contexts_of_use.push(ContextOfUse { id: "tablet".into(), label: "Tablet".into(), ..Default::default() });
    g.wireframe_steps.push(WireframeStep {
        id: "Pick".into(),
        label: "Pick".into(),
        offers: vec![Offer { command: "Choose".into(), aio: "single-select".into() }],
        ..Default::default()
    });
    g
}

#[test]
fn coverage_passes_when_every_aio_context_has_a_rule() {
    let mut g = graph_with_step_and_contexts();
    for ctx in ["phone", "tablet"] {
        g.reification_rules.push(ReificationRule {
            id: format!("r-{ctx}"), aio: "single-select".into(), context: ctx.into(),
            cio: "searchable-list".into(), ..Default::default()
        });
    }
    assert!(check_reification_coverage(&g).is_empty());
}

#[test]
fn coverage_fails_on_a_missing_aio_context_pair() {
    let mut g = graph_with_step_and_contexts();
    g.reification_rules.push(ReificationRule {
        id: "r-tablet".into(), aio: "single-select".into(), context: "tablet".into(),
        cio: "segmented-control".into(), ..Default::default()
    });
    let v = check_reification_coverage(&g);
    assert!(v.iter().any(|x| x.message.contains("phone")), "should name the uncovered phone context");
}

#[test]
fn off_system_cio_is_rejected() {
    let mut g = DomainGraph::default();
    g.design_systems.push(DesignSystem { id: "ds".into(), cios: vec!["primary-button".into()], ..Default::default() });
    g.reification_rules.push(ReificationRule {
        id: "bad".into(), aio: "trigger-action".into(), context: "phone".into(),
        cio: "fancy-carousel".into(), ..Default::default()
    });
    let v = check_closed_vocabulary(&g);
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("fancy-carousel"));
}

#[test]
fn literal_style_is_rejected_token_passes() {
    let mut g = DomainGraph::default();
    g.design_systems.push(DesignSystem { id: "ds".into(), tokens: vec!["color.accent".into()], ..Default::default() });
    g.wireframe_steps.push(WireframeStep {
        id: "S".into(), label: "S".into(), styles: vec!["#3366ff".into()], ..Default::default()
    });
    assert_eq!(check_tokens_not_literals(&g).len(), 1, "literal should be rejected");
    g.wireframe_steps[0].styles = vec!["color.accent".into()];
    assert!(check_tokens_not_literals(&g).is_empty(), "token should pass");
}
