//! The What adapter — the framework graph as a governed contract surface.
//!
//! The C#/Bicep adapters need a language server because a compiler owns the
//! symbol table. `product-core` already owns the What graph as typed data,
//! so this adapter reads kinds, containers and signatures straight off
//! `DomainGraph` — no host, no LSP, same [`PolicyRow`] mechanism.
//!
//! The table below is the falsifiable part (§9's design rule): it names
//! which framework elements form a boundary. Boundaries and the published
//! event/command contract are surface; entities, value objects, views and
//! UI steps are internal elaboration and stay out of the way.

use product_core::pf::model::DomainGraph;
use serde::Serialize;

use crate::store::DddStore;
use crate::surface::{decide, ChangeKind, PolicyRow, SymbolFacts};

/// The contract-surface policy table for the What graph. Row order matters:
/// the first match decides.
pub const WHAT_POLICY: &[PolicyRow] = &[
    PolicyRow {
        id: "pf-add-system",
        changes: &[ChangeKind::Added],
        kinds: &["system"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a system (§3.2.5) is a deployment, team and contract boundary",
    },
    PolicyRow {
        id: "pf-add-context-mapping",
        changes: &[ChangeKind::Added],
        kinds: &["context-mapping"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a context mapping (§3.1) declares how two models translate — it is the seam itself",
    },
    PolicyRow {
        id: "pf-journey-crossing",
        changes: &[ChangeKind::Added],
        kinds: &["journey-crossing"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a journey crossing a system boundary (§3.0.1) must be absorbed by a Translation",
    },
    PolicyRow {
        id: "pf-add-event",
        changes: &[ChangeKind::Added],
        kinds: &["event"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a domain event is the published contract other contexts read",
    },
    PolicyRow {
        id: "pf-event-signature",
        changes: &[ChangeKind::SignatureChanged],
        kinds: &["event", "command"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a payload field added or removed changes a wire contract consumers depend on",
    },
    PolicyRow {
        id: "pf-add-command",
        changes: &[ChangeKind::Added],
        kinds: &["command"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a command is an entry point to a Decider; its payload is a wire contract",
    },
    PolicyRow {
        id: "pf-quality-demand",
        changes: &[ChangeKind::Added],
        kinds: &["quality-demand"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a quality demand (§3.6) binds runtime behaviour or a How element",
    },
];

/// The `contract_location` a declaration uses to name a What element, so
/// the join is one convention across the report and the interceptor.
pub fn what_ref(element_id: &str) -> String {
    format!("what:{element_id}")
}

/// One classified element of the What graph.
#[derive(Debug, Clone, Serialize)]
pub struct WhatElement {
    pub kind: String,
    pub id: String,
    /// The context or system the element sits in; empty at product level.
    pub container: String,
    pub signature: String,
    pub surface: bool,
    /// The policy row that decided, when one matched.
    pub rule: Option<&'static str>,
    pub rule_claim: Option<&'static str>,
    /// The declaration governing it, when one names it.
    pub governed_by: Option<String>,
}

/// A surface element with nothing in `.ddd/` naming it.
#[derive(Debug, Clone, Serialize)]
pub struct WhatEscape {
    pub kind: String,
    pub id: String,
    pub container: String,
    pub rule: Option<&'static str>,
    pub rule_claim: Option<&'static str>,
}

/// The What-governance report, stating its coverage as well as its
/// findings (`dec/ddd/report-coverage-explicit`).
#[derive(Debug, Default, Serialize)]
pub struct WhatReport {
    pub product: String,
    /// Every element the table classified, surface or not.
    pub classified: usize,
    /// Elements the table called internal elaboration.
    pub internal: usize,
    /// Surface elements with a declaration naming them.
    pub governed: Vec<WhatElement>,
    /// Surface elements with none.
    pub escapes: Vec<WhatEscape>,
}

impl WhatReport {
    pub fn is_clean(&self) -> bool {
        self.escapes.is_empty()
    }

    /// Surface elements found, governed or not.
    pub fn surface_total(&self) -> usize {
        self.governed.len() + self.escapes.len()
    }
}

/// Classify one graph and join it against the governance store.
pub fn report(store: &DddStore, graph: &DomainGraph, product: &str) -> WhatReport {
    let mut out = WhatReport { product: product.to_string(), ..Default::default() };
    for mut el in classify(graph) {
        out.classified += 1;
        if !el.surface {
            out.internal += 1;
            continue;
        }
        el.governed_by = governing_declaration(store, &el.id);
        match el.governed_by.is_some() {
            true => out.governed.push(el),
            false => out.escapes.push(WhatEscape {
                kind: el.kind,
                id: el.id,
                container: el.container,
                rule: el.rule,
                rule_claim: el.rule_claim,
            }),
        }
    }
    out.governed.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
    out.escapes.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
    out
}

/// The seam declaration naming this element, if any.
fn governing_declaration(store: &DddStore, element_id: &str) -> Option<String> {
    let want = what_ref(element_id);
    store.seams.iter().find(|s| s.contract_location == want).map(|s| s.id.clone())
}

/// Walk every element the table has an opinion about, in graph order.
pub fn classify(graph: &DomainGraph) -> Vec<WhatElement> {
    let mut out = Vec::new();
    for s in &graph.systems {
        out.push(element("system", &s.id, "", &format!("{} system", s.kind)));
    }
    for m in &graph.context_mappings {
        let sig = format!("{} <-> {}", m.concept_a, m.concept_b);
        out.push(element("context-mapping", &m.id, "", &sig));
    }
    for j in &graph.journeys {
        for via in &j.crosses_via {
            let id = format!("{}#{via}", j.id);
            out.push(element("journey-crossing", &id, &j.id, &format!("crosses via {via}")));
        }
    }
    for e in &graph.events {
        out.push(element("event", &e.id, &e.context, &fields_of(&e.fields)));
    }
    for c in &graph.commands {
        out.push(element("command", &c.id, &c.context, &fields_of(&c.fields)));
    }
    for q in &graph.quality_demands {
        out.push(element("quality-demand", &q.id, &q.scopes, &q.bound));
    }
    // Internal elaboration: classified so the report can state coverage,
    // never surface (no table row claims them).
    for en in &graph.entities {
        out.push(element("entity", &en.id, &en.context, ""));
    }
    for v in &graph.value_objects {
        out.push(element("value-object", &v.id, &v.context, ""));
    }
    for r in &graph.read_models {
        out.push(element("read-model", &r.id, "", ""));
    }
    for w in &graph.wireframe_steps {
        out.push(element("ui-step", &w.id, "", ""));
    }
    out
}

/// A payload signature: the declared field names, in order.
fn fields_of(fields: &[product_core::pf::model::Attribute]) -> String {
    let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    format!("({})", names.join(", "))
}

/// Run one element through the table. Addition is the only change the
/// static report can see; the signature row waits for the interceptor.
fn element(kind: &str, id: &str, container: &str, signature: &str) -> WhatElement {
    let facts = SymbolFacts {
        name: id.to_string(),
        container: container.to_string(),
        kind: kind.to_string(),
        visibility: String::new(),
        signature: signature.to_string(),
        decorators: Vec::new(),
        exported: false,
        extra: Default::default(),
        sel_line: 0,
        sel_char: 0,
    };
    let (surface, rule, rule_claim) = decide(WHAT_POLICY, ChangeKind::Added, &facts, "");
    WhatElement {
        kind: kind.to_string(),
        id: id.to_string(),
        container: container.to_string(),
        signature: signature.to_string(),
        surface,
        rule,
        rule_claim,
        governed_by: None,
    }
}

#[cfg(test)]
#[path = "what_tests.rs"]
mod tests;
