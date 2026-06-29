//! Turtle emission for the §3.0–§3.6 product-boundary nodes.
//!
//! Split from [`super::turtle`] for the 400-line gate. Predicates follow the
//! sibling camelCase convention (`pf:ownsDomain`, `pf:journeyOf`, …); the §9
//! derivation contract names the same edges in snake_case.

use super::model;
use super::turtle::lit;

/// §3.0 — the product root: the domains and systems it owns, plus its
/// What-version (§7.3).
pub(super) fn emit_product(out: &mut String, p: &model::Product) {
    out.push_str(&format!("d:{} a pf:Product ;\n  rdfs:label {}", p.id, lit(&p.label)));
    if !p.purpose.is_empty() {
        out.push_str(&format!(" ;\n  pf:purpose {}", lit(&p.purpose)));
    }
    for d in &p.owns_domain {
        out.push_str(&format!(" ;\n  pf:ownsDomain d:{}", d));
    }
    for s in &p.owns_system {
        out.push_str(&format!(" ;\n  pf:ownsSystem d:{}", s));
    }
    if let Some(v) = &p.version {
        out.push_str(&format!(" ;\n  pf:versionedAs {}", lit(v)));
    }
    out.push_str(" .\n\n");
}

/// §3.0.1 — a journey: the product it belongs to, the flows it composes, and the
/// Translation crossings it goes through.
pub(super) fn emit_journey(out: &mut String, j: &model::Journey) {
    out.push_str(&format!("d:{} a pf:Journey ;\n  rdfs:label {}", j.id, lit(&j.label)));
    if !j.product.is_empty() {
        out.push_str(&format!(" ;\n  pf:journeyOf d:{}", j.product));
    }
    for f in &j.composes_flow {
        out.push_str(&format!(" ;\n  pf:composesFlow d:{}", f));
    }
    for t in &j.crosses_via {
        out.push_str(&format!(" ;\n  pf:crossesVia d:{}", t));
    }
    out.push_str(" .\n\n");
}

/// §3.6 — a quality demand: its kind, bound, the element it scopes, and the
/// telemetry source or How-side contract it is checked against. The `boundedBy`
/// edge runs from the scoped element to the demand (§9).
pub(super) fn emit_quality_demand(out: &mut String, q: &model::QualityDemand) {
    out.push_str(&format!("d:{} a pf:QualityDemand ;\n  rdfs:label {}", q.id, lit(&q.label)));
    if !q.kind.is_empty() {
        out.push_str(&format!(" ;\n  pf:demandKind {}", lit(&q.kind)));
    }
    if !q.bound.is_empty() {
        out.push_str(&format!(" ;\n  pf:bound {}", lit(&q.bound)));
    }
    if !q.scopes.is_empty() {
        out.push_str(&format!(" ;\n  pf:scopes d:{}", q.scopes));
    }
    if let Some(m) = &q.measured_by {
        out.push_str(&format!(" ;\n  pf:measuredBy {}", lit(m)));
    }
    if let Some(c) = &q.constrains {
        out.push_str(&format!(" ;\n  pf:constrains d:{}", c));
    }
    out.push_str(" .\n\n");
    if !q.scopes.is_empty() {
        out.push_str(&format!("d:{} pf:boundedBy d:{} .\n\n", q.scopes, q.id));
    }
}
