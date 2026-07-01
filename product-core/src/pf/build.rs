//! SPMC build-context assembly for a deliverable (§5).
//!
//! Assembles the *frozen Context* an agent needs to realise one delivery
//! feature: the What subgraph it ships, the How to apply by pointer, the Decider
//! oracle its behaviour must match, and the acceptance it must satisfy. The
//! agent re-derives nothing — the hard reasoning is upstream (§5).

use super::bundle::{bundle_many, covered};
use super::decider::Decider;
use super::deliverable::Deliverable;
use super::how::HowContract;
use super::model::DomainGraph;
use super::feature::Feature;

/// Assemble the SPMC frozen context (markdown) for a deliverable.
pub fn assemble(
    d: &Deliverable,
    feature: &Feature,
    graph: &DomainGraph,
    how: Option<&HowContract>,
    deciders: &[Decider],
    product: &str,
) -> String {
    let scope = covered(graph, &feature.anchors, feature.depth());
    let mut out = String::new();
    out.push_str(&format!("# Build Context: {} — feature `{}`\n\n", d.id, d.feature));
    out.push_str("⟦Ω:SPMC⟧ Frozen context. Produce one artifact; reference the What/How by pointer; do not re-decide them.\n\n---\n\n");

    out.push_str("## What — the feature to realise\n\n");
    match bundle_many(graph, &feature.anchors, feature.depth(), product) {
        Some(b) => out.push_str(&b),
        None => out.push_str("_(feature resolves to no nodes)_\n"),
    }
    out.push_str("\n---\n\n");

    out.push_str("## How — apply these by pointer\n\n");
    render_how(how, &mut out);
    out.push_str("\n---\n\n");

    out.push_str("## Behaviour — the Decider oracle (your code must compute the same)\n\n");
    render_deciders(deciders, &scope, &mut out);
    out.push_str("\n---\n\n");

    out.push_str("## Acceptance — what makes this done\n\n");
    if d.acceptance.is_empty() {
        out.push_str("_(no acceptance criteria declared)_\n");
    }
    for a in &d.acceptance {
        out.push_str(&format!("- [{}] {}: {}\n", a.status, a.id, a.statement));
    }
    out
}

fn render_how(how: Option<&HowContract>, out: &mut String) {
    let Some(h) = how else {
        out.push_str("_(no How contract loaded)_\n");
        return;
    };
    if !h.principles.is_empty() {
        out.push_str("**Principles** (obey):\n");
        for p in &h.principles {
            out.push_str(&format!("- {}: {}\n", p.id, p.statement));
        }
    }
    if !h.patterns.is_empty() {
        out.push_str("\n**Patterns** (apply):\n");
        for p in &h.patterns {
            out.push_str(&format!("- {}: {}\n", p.id, p.shape));
        }
    }
    out.push_str(&format!("\n**Application contract**: {} ({})\n", h.application_contract.id, h.application_contract.language));
}

fn render_deciders(deciders: &[Decider], scope: &std::collections::BTreeSet<String>, out: &mut String) {
    let in_scope: Vec<&Decider> = deciders.iter().filter(|d| scope.contains(&d.decides_for)).collect();
    if in_scope.is_empty() {
        out.push_str("_(no Decider over an in-scope aggregate — trivial behaviour)_\n");
        return;
    }
    for dec in in_scope {
        out.push_str(&format!("### Decider `{}` (decides for {})\n", dec.id, dec.decides_for));
        for s in &dec.scenarios {
            let given: Vec<&str> = s.given.iter().map(|e| e.id()).collect();
            let then = match (&s.then.reject, &s.then.emit) {
                (Some(r), _) => format!("reject {r}"),
                (None, Some(e)) => format!("emit {:?}", e.iter().map(|x| x.id()).collect::<Vec<_>>()),
                _ => "—".to_string(),
            };
            out.push_str(&format!("- {}: given {:?}, when {}, then {}\n", s.name, given, s.when.id(), then));
        }
    }
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;
