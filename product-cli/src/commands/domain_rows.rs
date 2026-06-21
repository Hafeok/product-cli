//! `product domain list` row builders, split from domain.rs (400-line gate).

use product_core::pf::ids::{NodeKind, ALL_KINDS};
use product_core::pf::DomainGraph;

/// Build `(kind, id, label)` rows for `list`, honouring an optional filter.
pub(crate) fn list_rows(g: &DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = structure_rows(g, filter);
    out.extend(ui_layer_rows(g, filter));
    let _ = ALL_KINDS; // kinds enumerated across both helpers in canonical order
    out
}

/// Rows for the §3.1/§3.2 structure + behaviour node kinds.
fn structure_rows(g: &DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut push = |k: NodeKind, id: &str, label: String| {
        if filter.is_none_or(|f| f == k) {
            out.push((k.cli_name().to_string(), id.to_string(), label));
        }
    };
    for n in &g.contexts { push(NodeKind::BoundedContext, &n.id, n.label.clone()); }
    for n in &g.entities { push(NodeKind::Entity, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.value_objects { push(NodeKind::ValueObject, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.relations { push(NodeKind::Relation, &n.id, format!("{} -{}-> {}", n.from, n.cardinality, n.to)); }
    for n in &g.invariants { push(NodeKind::Invariant, &n.id, n.statement.clone()); }
    for n in &g.context_mappings { push(NodeKind::ContextMapping, &n.id, format!("{} <-> {}", n.concept_a, n.concept_b)); }
    for n in &g.commands { push(NodeKind::Command, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.events { push(NodeKind::Event, &n.id, format!("{} changes {}", n.label, n.changes)); }
    for n in &g.read_models { push(NodeKind::ReadModel, &n.id, n.label.clone()); }
    out
}

/// Rows for the §3.2.1–§3.2.4 UI layer: pages (with derived top-level marking),
/// flows (with entry page), the application root, AIOs (core + registered), and
/// contexts of use.
fn ui_layer_rows(g: &DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut push = |k: NodeKind, id: &str, label: String| {
        if filter.is_none_or(|f| f == k) {
            out.push((k.cli_name().to_string(), id.to_string(), label));
        }
    };
    // §3.2.4 — "top-level" is derived: a page with an inbound edge from the root.
    let top_level: std::collections::HashSet<&str> = g
        .application_roots
        .iter()
        .flat_map(|r| r.navigates_from_root.iter().map(String::as_str))
        .collect();
    for n in &g.wireframe_steps {
        let mark = if top_level.contains(n.id.as_str()) { " [top-level]" } else { "" };
        push(NodeKind::WireframeStep, &n.id, format!("{}{mark}", n.label));
    }
    for n in &g.flows {
        let label = match &n.entry_page {
            Some(e) => format!("{} (entry: {})", n.label, e),
            None => n.label.clone(),
        };
        push(NodeKind::Flow, &n.id, label);
    }
    for n in &g.application_roots {
        push(NodeKind::ApplicationRoot, &n.id, format!("→ {}", n.navigates_from_root.join(", ")));
    }
    // The closed-core AIO vocabulary (§3.2.2) is always recognised, shown first.
    for core in product_core::pf::ids::CORE_AIOS {
        push(NodeKind::Aio, core, "(core)".to_string());
    }
    for n in &g.aios { push(NodeKind::Aio, &n.id, n.label.clone()); }
    for n in &g.contexts_of_use {
        let label = match (&n.dimension, &n.value) {
            (Some(d), Some(v)) => format!("{} [{}={}]", n.label, d, v),
            _ => n.label.clone(),
        };
        push(NodeKind::ContextOfUse, &n.id, label);
    }
    for r in &g.reification_rules {
        push(NodeKind::ReificationRule, &r.id, format!("{} @{} → {}", r.aio, r.context, r.cio));
    }
    out.extend(plain_rows(g, filter));
    out
}

/// Rows for the §3.2.3–§4.5 kinds without bespoke labels (listed by id via the
/// canonical `ids()` table).
fn plain_rows(g: &DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    g.ids()
        .into_iter()
        .filter(|(_, k)| {
            filter.is_none_or(|f| f == *k)
                && matches!(k,
                    NodeKind::WcagCriterion | NodeKind::Attestation | NodeKind::ContentStore
                    | NodeKind::DesignSystem | NodeKind::Cio | NodeKind::Token)
        })
        .map(|(id, k)| (k.cli_name().to_string(), id, String::new()))
        .collect()
}

