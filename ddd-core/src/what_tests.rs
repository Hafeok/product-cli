//! What-adapter classification plus the governance join, over hand-built graphs.

use super::*;
use product_core::pf::model::{Command, ContextMapping, Entity, Event, ReadModel};
use product_core::pf::model_product::{Journey, QualityDemand};
use product_core::pf::model_ui::{System, Trigger};

/// Add a §3.2.0 Translation carrying `view` (which projects `event`) and
/// issuing `command` across a boundary.
fn with_translation(g: &mut DomainGraph, view: &str, event: &str, command: &str) {
    g.read_models.push(ReadModel {
        id: view.into(),
        projects: vec![event.into()],
        ..Default::default()
    });
    g.triggers.push(Trigger {
        id: "translate-in".into(),
        source: "automated".into(),
        issues: command.into(),
        watches: Some(view.into()),
        translates_from: Some("payments-api".into()),
        ..Default::default()
    });
}

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
        bindings: Vec::new(),
        notes: None,
    }
}

fn surface_of(g: &DomainGraph) -> Vec<(String, String)> {
    classify(g)
        .into_iter()
        .filter(|e| e.surface)
        .map(|e| (e.kind, e.id))
        .collect()
}

#[test]
fn boundary_kinds_are_surface_whatever_they_connect_to() {
    let surface = surface_of(&graph());
    for (kind, id) in [
        ("system", "payments-api"),
        ("context-mapping", "billing-to-ledger"),
        ("journey-crossing", "checkout#translate-to-ledger"),
        ("quality-demand", "auth-latency"),
    ] {
        assert!(
            surface.contains(&(kind.to_string(), id.to_string())),
            "{kind}/{id} missing from {surface:?}"
        );
    }
}

/// dec/ddd/what-published-qualifier: without a Translation carrying them,
/// events and commands are internal — the rule DDD-what-02 got wrong.
#[test]
fn an_event_or_command_no_translation_carries_is_internal() {
    let surface = surface_of(&graph());
    for kind in ["event", "command"] {
        assert!(
            !surface.iter().any(|(k, _)| k == kind),
            "{kind} must not be surface without a crossing: {surface:?}"
        );
    }
    let classified = classify(&graph());
    let ev = classified.iter().find(|e| e.kind == "event").expect("event");
    assert_eq!(ev.visibility, INTERNAL);
}

#[test]
fn a_translation_publishes_its_view_its_command_and_the_events_it_projects() {
    let mut g = graph();
    with_translation(&mut g, "OrderConfirmation", "PaymentAuthorized", "AuthorizePayment");
    let surface = surface_of(&g);
    for (kind, id) in [
        ("read-model", "OrderConfirmation"),
        ("event", "PaymentAuthorized"),
        ("command", "AuthorizePayment"),
    ] {
        assert!(
            surface.contains(&(kind.to_string(), id.to_string())),
            "{kind}/{id} missing from {surface:?}"
        );
    }
    // The view a Translation watches is surface; the plain one is not.
    let plain = classify(&g)
        .into_iter()
        .find(|e| e.id == "PaymentList")
        .expect("PaymentList");
    assert!(!plain.surface);
    assert_eq!(plain.visibility, INTERNAL);
}

#[test]
fn published_set_follows_the_translation_edges_only() {
    let mut g = graph();
    with_translation(&mut g, "OrderConfirmation", "PaymentAuthorized", "AuthorizePayment");
    let published = published_set(&g);
    assert!(published.contains("OrderConfirmation"));
    assert!(published.contains("PaymentAuthorized"));
    assert!(published.contains("AuthorizePayment"));
    // A non-Translation trigger publishes nothing.
    g.triggers.push(Trigger {
        id: "user-pays".into(),
        source: "user".into(),
        issues: "PayNow".into(),
        ..Default::default()
    });
    assert!(!published_set(&g).contains("PayNow"));
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
    // Four boundary kinds; the event/command/view need a Translation.
    assert_eq!(r.escapes.len(), 4, "{:?}", r.escapes);
    assert_eq!(r.surface_total(), 4);
    // Coverage: internals are counted, not silently dropped.
    assert_eq!(r.internal, 4);
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
    assert_eq!(r.escapes.len(), 3);
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
        visibility: PUBLISHED.into(),
        signature: "(amount)".into(),
        decorators: Vec::new(),
        exported: false,
        extra: Default::default(),
        sel_line: 0,
        sel_char: 0,
    };
    let (surface, rule, _) =
        decide(WHAT_POLICY, ChangeKind::SignatureChanged, &facts, PUBLISHED);
    assert!(surface);
    assert_eq!(rule, Some("pf-event-signature"));
    // Removal has no row yet: the table names what forms a boundary.
    let (removed, _, _) = decide(WHAT_POLICY, ChangeKind::Removed, &facts, PUBLISHED);
    assert!(!removed);
    // An internal event's payload change is not a boundary event.
    let (internal, _, _) =
        decide(WHAT_POLICY, ChangeKind::SignatureChanged, &facts, INTERNAL);
    assert!(!internal);
}

#[test]
fn an_empty_graph_is_clean_with_nothing_classified() {
    let r = report(&DddStore::default(), &DomainGraph::default(), "acme");
    assert!(r.is_clean());
    assert_eq!(r.classified, 0);
}
