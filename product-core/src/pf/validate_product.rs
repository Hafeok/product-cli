//! §3.0–§3.6 presence/cardinality shapes for the product-boundary node family.
//!
//! These mirror the per-node checks in [`super::validate`] for the product root
//! (§3.0), the journey (§3.0.1), and the quality demand (§3.6). Graph-level
//! completeness (journey conformance — every crossing a Translation) lives in
//! [`super::rules_pattern`], run only under `--strict`.

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::validate::Violation;

/// Run every product-boundary shape over the whole graph (the `validate_graph`
/// path), kept here so the caller stays one line.
pub(super) fn check_all(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for p in &graph.products {
        check_product(p, graph, v);
    }
    for j in &graph.journeys {
        check_journey(j, graph, v);
    }
    for q in &graph.quality_demands {
        check_quality_demand(q, graph, v);
    }
}

/// Run the shape for a single product-boundary node by id (the in-loop path),
/// kept here so `validate::check_local_shape` stays one arm.
pub(super) fn check_local(graph: &DomainGraph, id: &str, v: &mut Vec<Violation>) {
    match graph.kind_of(id) {
        Some(NodeKind::Product) => {
            if let Some(p) = graph.products.iter().find(|n| n.id == id) { check_product(p, graph, v); }
        }
        Some(NodeKind::Journey) => {
            if let Some(j) = graph.journeys.iter().find(|n| n.id == id) { check_journey(j, graph, v); }
        }
        Some(NodeKind::QualityDemand) => {
            if let Some(q) = graph.quality_demands.iter().find(|n| n.id == id) { check_quality_demand(q, graph, v); }
        }
        _ => {}
    }
}

/// §3.0 — the product owns domains and systems. References are validated when
/// present (the System precedent); presence of the owned set is a strict concern.
pub(super) fn check_product(p: &super::model::Product, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if p.purpose.trim().is_empty() {
        v.push(Violation::new(&p.id, "purpose",
            "§3.0 A product must state its purpose in one sentence (the ubiquitous language)."));
    }
    for d in &p.owns_domain {
        if !graph.is_kind(d, NodeKind::BoundedContext) {
            v.push(Violation::new(&p.id, "owns_domain",
                "§3.0 A product's owned domain must resolve to a declared bounded context."));
        }
    }
    for s in &p.owns_system {
        if !graph.is_kind(s, NodeKind::System) {
            v.push(Violation::new(&p.id, "owns_system",
                "§3.0 A product's owned system must resolve to a declared System."));
        }
    }
}

/// §3.0.1 — a journey references a product, the flows it composes, and the
/// Translation triggers it crosses via. References are resolved here; the
/// "every crossing is a Translation" rule is the strict journey-conformance check.
pub(super) fn check_journey(j: &super::model::Journey, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if !j.product.is_empty() && !graph.is_kind(&j.product, NodeKind::Product) {
        v.push(Violation::new(&j.id, "product",
            "§3.0.1 A journey's product must resolve to a declared Product."));
    }
    for f in &j.composes_flow {
        if !graph.is_kind(f, NodeKind::Flow) {
            v.push(Violation::new(&j.id, "composes_flow",
                "§3.0.1 A journey composes flows that exist — each must resolve to a declared Flow."));
        }
    }
    for t in &j.crosses_via {
        if !graph.is_kind(t, NodeKind::Trigger) {
            v.push(Violation::new(&j.id, "crosses_via",
                "§3.0.1 A journey crosses via a Translation — each crossing must resolve to a declared Trigger."));
        }
    }
}

/// §3.6 — a quality demand is one of two checkable kinds. A runtime bound names
/// the telemetry it is measured against; an architectural constraint names the
/// How-side contract it binds. A demand that can name neither is prose, not spec.
pub(super) fn check_quality_demand(q: &super::model::QualityDemand, graph: &DomainGraph, v: &mut Vec<Violation>) {
    const KINDS: [&str; 2] = ["runtime-bound", "architectural"];
    if q.kind.trim().is_empty() {
        v.push(Violation::new(&q.id, "kind",
            "§3.6 A quality demand must declare its kind (runtime-bound or architectural)."));
    } else if !KINDS.contains(&q.kind.as_str()) {
        v.push(Violation::new(&q.id, "kind",
            "§3.6 A quality demand's kind must be runtime-bound or architectural — there is no third way to check it."));
    }
    if q.bound.trim().is_empty() {
        v.push(Violation::new(&q.id, "bound",
            "§3.6 A quality demand must state a checkable bound, never prose."));
    }
    if q.scopes.trim().is_empty() {
        v.push(Violation::new(&q.id, "scopes",
            "§3.6 A quality demand is located, not listed — it must scope the element it bounds."));
    } else if graph.contains(&q.scopes) && !scopes_a_boundable_element(graph, &q.scopes) {
        v.push(Violation::new(&q.id, "scopes",
            "§3.6 A quality demand must scope a system, flow, or UI step (or a Decider)."));
    }
    if q.kind == "runtime-bound" && q.measured_by.as_deref().map(str::trim).unwrap_or("").is_empty() {
        v.push(Violation::new(&q.id, "measured_by",
            "§3.6 A runtime bound must name the telemetry source it is measured against (runtime-bound conformance, §6.3)."));
    }
    if q.kind == "architectural" && q.constrains.as_deref().map(str::trim).unwrap_or("").is_empty() {
        v.push(Violation::new(&q.id, "constrains",
            "§3.6 An architectural constraint must name the How-side contract it binds (checked at build time)."));
    }
}

/// §3.6 — the cross-artifact check: an architectural quality demand binds the How
/// (§4), so its `constrains` must resolve to a real How element (decision /
/// principle / pattern / interface / application contract). Run only when a How
/// contract is loaded alongside the graph (strict validate).
pub fn constrains_bind_how(graph: &DomainGraph, how: &super::how::HowContract) -> Vec<Violation> {
    let mut v = Vec::new();
    for q in &graph.quality_demands {
        if q.kind != "architectural" {
            continue;
        }
        if let Some(c) = q.constrains.as_deref().filter(|s| !s.trim().is_empty()) {
            if !how.has_element(c) {
                v.push(Violation::new(&q.id, "constrains", &format!(
                    "§3.6 An architectural constraint must bind a real How element — '{c}' is not a declared decision/principle/pattern/interface.")));
            }
        }
    }
    v
}

/// A scoped element is one whose conformance can carry a demand: a system, flow,
/// or UI step. (A Decider scope resolves outside the graph, so an id absent from
/// the graph is left to the caller's `contains` guard.)
fn scopes_a_boundable_element(graph: &DomainGraph, id: &str) -> bool {
    matches!(
        graph.kind_of(id),
        Some(NodeKind::System | NodeKind::Flow | NodeKind::WireframeStep | NodeKind::ReadModel)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::{Product, QualityDemand};

    fn paths(v: &[Violation]) -> Vec<&str> {
        v.iter().map(|x| x.path.as_str()).collect()
    }

    #[test]
    fn product_requires_a_purpose() {
        let g = DomainGraph::default();
        let mut v = Vec::new();
        check_product(&Product { id: "p".into(), label: "P".into(), ..Default::default() }, &g, &mut v);
        assert!(paths(&v).contains(&"purpose"), "missing purpose is a finding: {v:?}");
    }

    #[test]
    fn runtime_bound_requires_a_measured_source() {
        let g = DomainGraph::default();
        let base = QualityDemand {
            id: "qd".into(), label: "L".into(), kind: "runtime-bound".into(),
            bound: "p99<=200ms".into(), scopes: "sys".into(), ..Default::default()
        };
        let mut v = Vec::new();
        check_quality_demand(&base, &g, &mut v);
        assert!(paths(&v).contains(&"measured_by"), "runtime bound needs telemetry: {v:?}");

        let ok = QualityDemand { measured_by: Some("telemetry".into()), ..base };
        let mut v2 = Vec::new();
        check_quality_demand(&ok, &g, &mut v2);
        assert!(!paths(&v2).contains(&"measured_by"), "a measured runtime bound passes: {v2:?}");
    }

    #[test]
    fn architectural_constrains_must_bind_a_real_how_element() {
        use crate::pf::how::{HowContract, TopDecision};
        let mut g = DomainGraph::default();
        g.quality_demands.push(QualityDemand {
            id: "qd".into(), label: "L".into(), kind: "architectural".into(),
            bound: "EU only".into(), scopes: "sys".into(),
            constrains: Some("DEC-residency".into()), ..Default::default()
        });
        let mut how = HowContract::default();
        // Unknown How id → a finding.
        assert!(!constrains_bind_how(&g, &how).is_empty());
        // Declared How decision → clears.
        how.top_decisions.push(TopDecision { id: "DEC-residency".into(), ..Default::default() });
        assert!(constrains_bind_how(&g, &how).is_empty());
    }

    #[test]
    fn architectural_constraint_requires_a_how_contract() {
        let g = DomainGraph::default();
        let qd = QualityDemand {
            id: "qd".into(), label: "L".into(), kind: "architectural".into(),
            bound: "EU only".into(), scopes: "sys".into(), ..Default::default()
        };
        let mut v = Vec::new();
        check_quality_demand(&qd, &g, &mut v);
        assert!(paths(&v).contains(&"constrains"), "architectural constraint binds the How: {v:?}");
    }
}
