//! Unit tests for strict What-conformance (graph-level completeness).

use super::*;
use crate::pf::model::*;

fn base() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "C".into(), label: "C".into(), ..Default::default() });
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "C".into(), definition: "d".into(), ..Default::default() });
    g.events.push(Event { id: "Placed".into(), label: "Placed".into(), context: "C".into(), changes: "Order".into() });
    g.commands.push(Command { id: "Place".into(), label: "Place".into(), context: "C".into(), targets: "Order".into(), emits: vec!["Placed".into()] });
    g
}

fn has(v: &[Violation], path: &str) -> bool {
    v.iter().any(|x| x.path == path)
}

#[test]
fn command_pattern_wants_a_trigger_and_an_event() {
    let mut g = base();
    // No trigger issues Place → Command-pattern finding; emits is non-empty so no event finding.
    let v = pattern_conformance(&g);
    assert!(has(&v, "trigger"), "{v:?}");
    assert!(!v.iter().any(|x| x.path == "emits"), "{v:?}");

    // Add a trigger and the command-pattern finding clears.
    g.triggers.push(Trigger { id: "t".into(), label: "T".into(), source: "user".into(), issues: "Place".into(), ..Default::default() });
    assert!(!has(&pattern_conformance(&g), "trigger"));

    // A command with no event is a finding.
    g.commands.push(Command { id: "Note".into(), label: "Note".into(), context: "C".into(), targets: "Order".into(), emits: vec![] });
    g.triggers.push(Trigger { id: "t2".into(), label: "T2".into(), source: "user".into(), issues: "Note".into(), ..Default::default() });
    assert!(pattern_conformance(&g).iter().any(|x| x.focus == "Note" && x.path == "emits"));
}

#[test]
fn flow_must_be_owned_and_view_must_be_consumed() {
    let mut g = base();
    g.flows.push(Flow { id: "f".into(), label: "F".into(), steps: vec![], ..Default::default() });
    assert!(has(&pattern_conformance(&g), "system"), "unowned flow is a finding");

    g.read_models.push(ReadModel { id: "Cart".into(), label: "Cart".into(), projects: vec!["Order".into()], ..Default::default() });
    assert!(pattern_conformance(&g).iter().any(|x| x.focus == "Cart" && x.path == "consumed"), "unconsumed view is a finding");

    // A UI step that surfaces the view clears the consumption finding.
    g.wireframe_steps.push(WireframeStep { id: "step".into(), label: "S".into(), surfaces: vec![Surface { projection: "Cart".into(), aio: "display-collection".into() }], ..Default::default() });
    assert!(!pattern_conformance(&g).iter().any(|x| x.focus == "Cart" && x.path == "consumed"));
}

#[test]
fn unreifiable_seam_flags_a_step_in_a_targeted_class() {
    let mut g = base();
    g.wireframe_steps.push(WireframeStep { id: "gallery".into(), label: "Gallery".into(), surfaces: vec![Surface { projection: "Cart".into(), aio: "display-collection".into() }], ..Default::default() });
    g.unreifiable_rules.push(UnreifiableRule { id: "u".into(), aio: "display-collection".into(), class: "tui".into(), rationale: Some("no grid form".into()) });

    // No system targets tui yet → no seam finding (the gap is merely recorded).
    assert!(!pattern_conformance(&g).iter().any(|x| x.focus == "gallery" && x.path == "aio"));

    // A system that targets tui makes the step a §4.5 finding.
    g.systems.push(System { id: "sys".into(), label: "S".into(), kind: "cli".into(), purpose: "tool".into(), target_classes: vec!["tui".into()], ..Default::default() });
    assert!(pattern_conformance(&g).iter().any(|x| x.focus == "gallery" && x.path == "aio"));
}
