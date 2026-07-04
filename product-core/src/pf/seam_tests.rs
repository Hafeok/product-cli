//! Unit tests for the §6.3 seam verification.

use super::*;
use crate::pf::model::*;

fn base() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Catalog".into(), label: "Catalog".into(), ..Default::default() });
    g.read_models.push(ReadModel { id: "OrderSummary".into(), label: "Order".into(), projects: vec!["O".into()], ..Default::default() });
    g.commands.push(Command { fields: vec![], id: "ConfirmOrder".into(), label: "Confirm".into(), context: "Catalog".into(), targets: "O".into(), emits: vec!["E".into()] });
    g
}

fn agreeing_step() -> WireframeStep {
    WireframeStep {
        id: "ReviewOrder".into(), label: "Review".into(),
        surfaces: vec![Surface { projection: "OrderSummary".into(), aio: "display-collection".into() }],
        offers: vec![Offer { command: "ConfirmOrder".into(), aio: "trigger-action".into() }],
        ..Default::default()
    }
}

#[test]
fn a_fully_agreeing_screen_passes() {
    let mut g = base();
    g.wireframe_steps.push(agreeing_step());
    let v = seam_verdict(&g, "ReviewOrder").expect("verdict");
    assert!(v.conformant, "should pass: {:?}", v.checks.iter().filter(|c| !c.passed).collect::<Vec<_>>());
}

#[test]
fn unprojected_datum_fails_that_subcheck() {
    let mut g = base();
    let mut step = agreeing_step();
    step.surfaces[0].projection = "Nonexistent".into();
    g.wireframe_steps.push(step);
    let v = seam_verdict(&g, "ReviewOrder").expect("verdict");
    assert!(!v.conformant);
    let datum = v.checks.iter().find(|c| c.name == "datum-projected").unwrap();
    assert!(!datum.passed && datum.findings[0].contains("Nonexistent"));
}

#[test]
fn foreign_command_fails_control_subcheck() {
    let mut g = base();
    let mut step = agreeing_step();
    step.offers[0].command = "GhostCommand".into();
    g.wireframe_steps.push(step);
    let v = seam_verdict(&g, "ReviewOrder").expect("verdict");
    let control = v.checks.iter().find(|c| c.name == "control-maps-to-command").unwrap();
    assert!(!control.passed && control.findings[0].contains("GhostCommand"));
}

#[test]
fn composite_lists_each_failing_subcheck_separately() {
    let mut g = base();
    // OrderSummary can be empty; the step won't cover it (state-coverage gap) and
    // references a content key with no store (content gap) and an undeclared AIO
    // context with no reify rule (reification gap).
    g.read_models[0].states = vec!["empty".into()];
    g.contexts_of_use.push(ContextOfUse { id: "phone".into(), label: "P".into(), ..Default::default() });
    let mut step = agreeing_step();
    step.offers[0].aio = "single-select".into();
    step.content_refs = vec![ContentRef { key: "x.y".into(), role: "heading".into() }];
    g.content_stores.push(ContentStore { id: "s".into(), locales: vec!["en".into()], ..Default::default() });
    g.wireframe_steps.push(step);
    let v = seam_verdict(&g, "ReviewOrder").expect("verdict");
    assert!(!v.conformant);
    let failed: std::collections::BTreeSet<&str> =
        v.checks.iter().filter(|c| !c.passed).map(|c| c.name.as_str()).collect();
    assert!(failed.contains("state-coverage"), "state gap: {failed:?}");
    assert!(failed.contains("content-coverage"), "content gap: {failed:?}");
    assert!(failed.contains("reification-coverage"), "reify gap: {failed:?}");
}
