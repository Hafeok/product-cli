//! Tests for the work-unit (SPMC) checker.

use super::*;
use crate::pf::how::HowContract;
use crate::pf::model::*;
use crate::pf::validate::Violation;
use crate::pf::work_unit::*;

const EXAMPLE: &str = include_str!("../../../schema/examples/work-unit.example.yaml");
const HOW: &str = include_str!("../../../schema/examples/how-contract.example.yaml");

fn blocking(vs: &[Violation]) -> usize {
    vs.iter().filter(|v| v.severity == "violation").count()
}

#[test]
fn bundled_example_has_no_blocking_violations() {
    let w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    assert_eq!(blocking(&validate_work_unit(&w, None, None)), 0);
}

#[test]
fn unfrozen_context_is_a_violation() {
    let mut w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    w.context.frozen = false;
    assert!(validate_work_unit(&w, None, None).iter().any(|x| x.path == "context.frozen"));
}

#[test]
fn empty_derived_from_is_a_violation() {
    let mut w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    w.context.derived_from.clear();
    assert!(validate_work_unit(&w, None, None).iter().any(|x| x.path == "context.derived_from"));
}

#[test]
fn domain_pointer_cross_checks_the_what_graph() {
    let mut w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    w.context.derived_from = vec!["domain:Ghost".into()];
    let empty = DomainGraph::default();
    assert!(validate_work_unit(&w, Some(&empty), None).iter().any(|x| x.message.contains("Ghost")));
}

#[test]
fn domain_pointer_resolves_when_present() {
    let mut w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    w.context.derived_from = vec!["domain:Task".into()];
    w.trace = None;
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), ..Default::default() });
    g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "d".into(), ..Default::default() });
    assert!(!validate_work_unit(&w, Some(&g), None).iter().any(|x| x.message.contains("Task")));
}

#[test]
fn applied_but_unenforced_principle_warns_on_trace_truth() {
    let mut w = WorkUnit::from_yaml(EXAMPLE).expect("parse");
    w.applies = vec!["feature-cohesion".into()];
    w.trace = None;
    let mut how = HowContract::from_yaml(HOW).expect("how");
    // strip enforcement of feature-cohesion → trace lie
    for p in how.principles.iter_mut() {
        if p.id == "feature-cohesion" { p.enforced_by.clear(); }
    }
    let vs = validate_work_unit(&w, None, Some(&how));
    assert!(vs.iter().any(|x| x.path == "trace" && x.message.contains("trace must be true")));
}
