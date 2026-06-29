//! Round-trip tests for Turtle seed parsing.

use super::*;
use crate::pf::turtle::to_turtle;

fn sample() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), purpose: Some("track work".into()), glossary: vec![] });
    g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "a unit of work".into(), identity: Some("id".into()), is_aggregate_root: true, attributes: vec![] });
    g.relations.push(Relation { id: "rel".into(), label: Some("owns".into()), from: "Task".into(), to: "Task".into(), cardinality: "one-to-many".into(), rationale: "self ref".into() });
    g.events.push(Event { id: "Done".into(), label: "Done".into(), context: "Tasks".into(), changes: "Task".into() });
    g.commands.push(Command { id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Done".into()] });
    g.read_models.push(ReadModel { id: "Open".into(), label: "Open".into(), projects: vec!["Task".into(), "Done".into()], ..Default::default() });
    g.flows.push(Flow { id: "Flow".into(), label: "Complete a task".into(), steps: vec!["Complete".into(), "Done".into(), "Open".into()], ..Default::default() });
    g
}

#[test]
fn turtle_seed_round_trips() {
    let original = sample();
    let ttl = to_turtle(&original, "demo");
    let parsed = from_turtle(&ttl).expect("parse seed");

    assert_eq!(parsed.contexts.len(), 1);
    assert_eq!(parsed.entities.len(), 1);
    let task = &parsed.entities[0];
    assert_eq!(task.id, "Task");
    assert_eq!(task.context, "Tasks");
    assert!(task.is_aggregate_root);
    assert_eq!(parsed.events[0].changes, "Task");
    assert_eq!(parsed.commands[0].targets, "Task");
    assert_eq!(parsed.commands[0].emits, vec!["Done".to_string()]);
    let mut projects = parsed.read_models[0].projects.clone();
    projects.sort();
    assert_eq!(projects, vec!["Done".to_string(), "Task".to_string()]);
    assert_eq!(parsed.flows[0].steps.len(), 3);
}

#[test]
fn seeded_graph_is_conformant() {
    let ttl = to_turtle(&sample(), "demo");
    let parsed = from_turtle(&ttl).expect("parse seed");
    assert_eq!(crate::pf::validate::validate_graph(&parsed), vec![]);
}

#[test]
fn malformed_turtle_errs() {
    assert!(from_turtle("this is not turtle <<<").is_err());
}

#[test]
fn system_round_trips_with_flow_ownership() {
    let mut g = DomainGraph::default();
    g.systems.push(System {
        id: "sys-shop".into(), label: "Acme Shop".into(), kind: "application".into(),
        purpose: "consumer e-commerce".into(), target_platforms: vec!["ios".into(), "web".into()],
        target_classes: vec!["gui".into()], root: Some("root-shop".into()),
        references_domain: vec![],
    });
    g.flows.push(Flow { id: "checkout".into(), label: "Checkout".into(), steps: vec![], system: Some("sys-shop".into()), ..Default::default() });

    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    let s = &parsed.systems[0];
    assert_eq!(s.id, "sys-shop");
    assert_eq!(s.kind, "application");
    assert_eq!(s.purpose, "consumer e-commerce");
    assert_eq!(s.root.as_deref(), Some("root-shop"));
    let mut platforms = s.target_platforms.clone();
    platforms.sort();
    assert_eq!(platforms, vec!["ios".to_string(), "web".to_string()]);
    assert_eq!(s.target_classes, vec!["gui".to_string()]);
    assert_eq!(parsed.flows[0].system.as_deref(), Some("sys-shop"));
}

#[test]
fn triggers_round_trip() {
    let mut g = DomainGraph::default();
    g.triggers.push(Trigger { id: "t-user".into(), label: "Place".into(), source: "user".into(), issues: "PlaceOrder".into(), ..Default::default() });
    g.triggers.push(Trigger {
        id: "t-auto".into(), label: "Restock".into(), source: "automated".into(),
        issues: "Restock".into(), watches: Some("LowStock".into()), translates_from: Some("sys-wms".into()),
    });
    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    assert_eq!(parsed.triggers.len(), 2);
    let user = parsed.triggers.iter().find(|t| t.id == "t-user").expect("user");
    assert_eq!(user.source, "user");
    assert_eq!(user.issues, "PlaceOrder");
    let auto = parsed.triggers.iter().find(|t| t.id == "t-auto").expect("auto");
    assert_eq!(auto.source, "automated");
    assert_eq!(auto.watches.as_deref(), Some("LowStock"));
    assert_eq!(auto.translates_from.as_deref(), Some("sys-wms"));
}

/// A graph with one node of every kind, every field populated — the spec for
/// a lossless Turtle round-trip. If `to_turtle`/`from_turtle` drop any field,
/// the `assert_eq!` below diffs it.
fn maximal() -> DomainGraph {
    let mut g = DomainGraph::default();
    max_structure(&mut g);
    max_behaviour(&mut g);
    max_ui(&mut g);
    max_data(&mut g);
    g
}

fn max_structure(g: &mut DomainGraph) {
    g.contexts.push(BoundedContext { id: "ctx".into(), label: "Context".into(), purpose: Some("a purpose".into()), glossary: vec!["term-a".into(), "term-b".into()] });
    g.entities.push(Entity {
        id: "ent".into(), label: "Entity".into(), context: "ctx".into(), definition: "an entity".into(),
        identity: Some("id".into()), is_aggregate_root: true,
        attributes: vec![Attribute { name: "email".into(), ty: Some("string".into()) }, Attribute { name: "name".into(), ty: None }],
    });
    g.value_objects.push(ValueObject { id: "vo".into(), label: "Money".into(), context: "ctx".into(), definition: Some("an amount".into()) });
    g.relations.push(Relation { id: "rel".into(), label: Some("owns".into()), from: "ent".into(), to: "vo".into(), cardinality: "one-to-many".into(), rationale: "structural".into() });
    g.invariants.push(Invariant { id: "inv".into(), statement: "must hold".into(), context: Some("ctx".into()), applies_to: Some("ent".into()) });
    g.context_mappings.push(ContextMapping { id: "map".into(), concept_a: "Aaa".into(), concept_b: "Bbb".into(), kind: Some("shared-kernel".into()), rationale: "shared".into() });
}

fn max_behaviour(g: &mut DomainGraph) {
    g.commands.push(Command { id: "cmd".into(), label: "Do".into(), context: "ctx".into(), targets: "ent".into(), emits: vec!["ev".into(), "ev2".into()] });
    g.events.push(Event { id: "ev".into(), label: "Done".into(), context: "ctx".into(), changes: "ent".into() });
    g.events.push(Event { id: "ev2".into(), label: "Also".into(), context: "ctx".into(), changes: "ent".into() });
    g.read_models.push(ReadModel { id: "rm".into(), label: "View".into(), projects: vec!["ent".into(), "ev".into()], states: vec!["loading".into(), "empty".into()] });
    g.flows.push(Flow { id: "flow".into(), label: "Journey".into(), steps: vec!["step".into()], entry_page: Some("step".into()), system: Some("sys".into()) });
    g.systems.push(System { id: "sys".into(), label: "App".into(), kind: "application".into(), purpose: "do things".into(), target_platforms: vec!["web".into()], target_classes: vec!["gui".into()], root: Some("root".into()), references_domain: vec![] });
    g.triggers.push(Trigger { id: "trig".into(), label: "Init".into(), source: "automated".into(), issues: "cmd".into(), watches: Some("rm".into()), translates_from: Some("sys".into()) });
}

fn max_ui(g: &mut DomainGraph) {
    g.wireframe_steps.push(WireframeStep {
        id: "step".into(), label: "Screen".into(), intent: Some("show the thing".into()),
        surfaces: vec![Surface { projection: "rm".into(), aio: "display-collection".into() }],
        offers: vec![Offer { command: "cmd".into(), aio: "action-trigger".into() }],
        transitions_to: vec!["step2".into()],
        state_meanings: vec![
            StateMeaning { projection: "rm".into(), state: "loading".into(), meaning: Some("fetching".into()), waiver: None },
            StateMeaning { projection: "rm".into(), state: "empty".into(), meaning: None, waiver: Some("ignorable".into()) },
        ],
        must_satisfy: vec!["wcag-1".into()],
        content_refs: vec![ContentRef { key: "heading".into(), role: "heading".into() }],
        styles: vec!["color-primary".into()],
        triggers: Some("trig".into()), displays: Some("rm".into()),
    });
    g.aios.push(Aio { id: "aio".into(), label: "Range".into(), means: Some("pick a range".into()), must_satisfy: vec!["wcag-1".into()] });
    g.contexts_of_use.push(ContextOfUse { id: "cou".into(), label: "Mobile".into(), dimension: Some("form-factor".into()), value: Some("handset".into()) });
    g.application_roots.push(ApplicationRoot { id: "root".into(), label: Some("Home".into()), navigates_from_root: vec!["step".into()] });
    g.wcag_criteria.push(WcagCriterion { id: "wcag-1".into(), label: Some("Contrast".into()), level: Some("AA".into()), verification: Some("machine".into()), satisfied: true });
    g.attestations.push(Attestation { id: "att".into(), step: "step".into(), criterion: "wcag-1".into(), date: "2026-01-01".into(), by: "auditor".into() });
    g.content_stores.push(ContentStore { id: "store".into(), label: Some("Copy".into()), locales: vec!["en".into(), "da".into()], resolutions: vec![Resolution { key: "heading".into(), locale: "en".into(), value: "Hello".into() }] });
    g.design_systems.push(DesignSystem { id: "ds".into(), label: Some("DS".into()), cios: vec!["cio".into()], tokens: vec!["color-primary".into()] });
    g.cios.push(Cio { id: "cio".into(), label: Some("Button".into()) });
    g.tokens.push(Token { id: "color-primary".into(), kind: Some("color".into()) });
    g.reification_rules.push(ReificationRule { id: "rr".into(), aio: "aio".into(), context: "cou".into(), cio: "cio".into(), rationale: Some("fits".into()) });
    g.unreifiable_rules.push(UnreifiableRule { id: "ur".into(), aio: "aio".into(), class: "tui".into(), rationale: Some("no form".into()) });
}

fn max_data(g: &mut DomainGraph) {
    g.reference_sets.push(ReferenceSet { id: "refset".into(), label: Some("Methods".into()), concept: "ent".into(), values: vec!["a".into(), "b".into()] });
    g.data_shapes.push(DataShape {
        id: "shape".into(), label: Some("Shape".into()), target: "ent".into(), required: vec!["email".into()],
        enums: vec![EnumConstraint { field: "method".into(), reference_set: "refset".into() }],
        types: vec![TypeConstraint { field: "email".into(), datatype: "string".into() }],
    });
    g.production_datasets.push(ProductionDataset { id: "ds-prod".into(), label: Some("Prod".into()), shape: "shape".into(), source: "data.json".into() });
}

#[test]
fn full_graph_round_trips_losslessly() {
    let g = maximal();
    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    let mut expected = g.clone();
    crate::pf::seed_canon::canonicalize(&mut expected);
    assert_eq!(parsed, expected, "every field must survive a Turtle round-trip");
    // re-export of the parsed graph is byte-stable (canonical order is a fixpoint).
    assert_eq!(to_turtle(&parsed, "demo"), to_turtle(&expected, "demo"));
}

#[test]
fn unreifiable_rules_round_trip() {
    let mut g = DomainGraph::default();
    g.unreifiable_rules.push(UnreifiableRule {
        id: "u-gallery".into(), aio: "display-collection".into(), class: "tui".into(),
        rationale: Some("no faithful character-grid form".into()),
    });
    let parsed = from_turtle(&to_turtle(&g, "demo")).expect("parse seed");
    assert_eq!(parsed.unreifiable_rules.len(), 1);
    let u = &parsed.unreifiable_rules[0];
    assert_eq!(u.aio, "display-collection");
    assert_eq!(u.class, "tui");
    assert_eq!(u.rationale.as_deref(), Some("no faithful character-grid form"));
}
