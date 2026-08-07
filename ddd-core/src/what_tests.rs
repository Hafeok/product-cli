//! What-adapter classification plus the governance join, over hand-built graphs.

use super::*;
use product_core::pf::model::{Command, ContextMapping, Entity, Event, ReadModel};
use product_core::pf::model_product::{Journey, QualityDemand};
use product_core::pf::model_ui::System;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.systems.push(System { id: "payments-api".into(), kind: "service".into(), ..Default::default() });
    g.context_mappings.push(ContextMapping {
        id: "billing-to-ledger".into(),
        concept_a: "billing".into(),
        concept_b: "ledger".into(),
        ..Default::default()
    });
    g.events.push(Event {
        id: "PaymentAuthorized".into(),
        context: "billing".into(),
        ..Default::default()
    });
    g.commands.push(Command {
        id: "AuthorizePayment".into(),
        context: "billing".into(),
        ..Default::default()
    });
    g.quality_demands.push(QualityDemand {
        id: "auth-latency".into(),
        bound: "p99 <= 200ms".into(),
        scopes: "payments-api".into(),
        ..Default::default()
    });
    g.journeys.push(Journey {
        id: "checkout".into(),
        crosses_via: vec!["translate-to-ledger".into()],
        ..Default::default()
    });
    // Internal elaboration — classified, never surface.
    g.entities.push(Entity { id: "Payment".into(), context: "billing".into(), ..Default::default() });
    g.read_models.push(ReadModel { id: "PaymentList".into(), ..Default::default() });
    g
}

fn seam(id: &str, element: &str) -> crate::seam::SeamDeclaration {
    crate::seam::SeamDeclaration {
        format: 1,
        id: id.into(),
        boundary: "b".into(),
        verdict_knowledge: "v".into(),
        contract_location: what_ref(element),
        obligations: Vec::new(),
        metadata: Default::default(),
        notes: None,
    }
}

#[test]
fn boundaries_and_the_published_contract_are_surface() {
    let classified = classify(&graph());
    let surface: Vec<(&str, &str)> = classified
        .iter()
        .filter(|e| e.surface)
        .map(|e| (e.kind.as_str(), e.id.as_str()))
        .collect();
    for expect in [
        ("system", "payments-api"),
        ("context-mapping", "billing-to-ledger"),
        ("journey-crossing", "checkout#translate-to-ledger"),
        ("event", "PaymentAuthorized"),
        ("command", "AuthorizePayment"),
        ("quality-demand", "auth-latency"),
    ] {
        assert!(surface.contains(&expect), "{expect:?} missing from {surface:?}");
    }
}

#[test]
fn internal_elaboration_is_classified_but_never_surface() {
    let classified = classify(&graph());
    for kind in ["entity", "read-model"] {
        let el = classified.iter().find(|e| e.kind == kind).expect(kind);
        assert!(!el.surface, "{kind} must not be surface");
        assert_eq!(el.rule, None, "no row should claim {kind}");
    }
}

#[test]
fn every_surface_element_names_the_row_that_decided_it() {
    for el in classify(&graph()).iter().filter(|e| e.surface) {
        assert!(el.rule.is_some(), "{} has no rule", el.id);
        assert!(el.rule_claim.is_some(), "{} has no claim", el.id);
    }
}

#[test]
fn an_ungoverned_graph_reports_every_surface_element_as_an_escape() {
    let store = DddStore::default();
    let r = report(&store, &graph(), "acme");
    assert!(r.governed.is_empty());
    assert_eq!(r.escapes.len(), 6, "{:?}", r.escapes);
    assert_eq!(r.surface_total(), 6);
    // Coverage: internals are counted, not silently dropped.
    assert_eq!(r.internal, 2);
    assert_eq!(r.classified, 8);
    assert!(!r.is_clean());
}

#[test]
fn a_seam_naming_the_element_moves_it_out_of_the_escapes() {
    let mut store = DddStore::default();
    store.seams.push(seam("seam/what/payments-api", "payments-api"));
    let r = report(&store, &graph(), "acme");
    assert_eq!(r.governed.len(), 1);
    assert_eq!(r.governed[0].id, "payments-api");
    assert_eq!(r.governed[0].governed_by.as_deref(), Some("seam/what/payments-api"));
    assert_eq!(r.escapes.len(), 5);
}

#[test]
fn a_seam_pointing_at_a_file_does_not_govern_a_what_element() {
    let mut store = DddStore::default();
    let mut s = seam("seam/csharp/thing", "payments-api");
    s.contract_location = "PaymentsApi.cs#Refund".into();
    store.seams.push(s);
    assert_eq!(report(&store, &graph(), "acme").governed.len(), 0);
}

#[test]
fn a_payload_change_is_surface_only_under_the_signature_row() {
    let facts = SymbolFacts {
        name: "PaymentAuthorized".into(),
        container: "billing".into(),
        kind: "event".into(),
        visibility: String::new(),
        signature: "(amount)".into(),
        decorators: Vec::new(),
        exported: false,
        extra: Default::default(),
        sel_line: 0,
        sel_char: 0,
    };
    let (surface, rule, _) = decide(WHAT_POLICY, ChangeKind::SignatureChanged, &facts, "");
    assert!(surface);
    assert_eq!(rule, Some("pf-event-signature"));
    // Removal has no row yet: the table names what forms a boundary.
    let (removed, _, _) = decide(WHAT_POLICY, ChangeKind::Removed, &facts, "");
    assert!(!removed);
}

#[test]
fn an_empty_graph_is_clean_with_nothing_classified() {
    let r = report(&DddStore::default(), &DomainGraph::default(), "acme");
    assert!(r.is_clean());
    assert_eq!(r.classified, 0);
}
