//! Unit tests for the C# reify slice — inference, emission, determinism.

use std::collections::BTreeMap;

use super::*;
use crate::pf::decider_logic::{
    CommandRef, DecideRule, DeciderLogic, EventRef, EvolveRule, Expectation, Guard, Scalar,
    Scenario,
};
use crate::pf::model::{Attribute, Entity, ReferenceSet, ValueObject};
use crate::pf::reify_ident::{method_name, pascal};

fn payload(pairs: &[(&str, Scalar)]) -> BTreeMap<String, Scalar> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn fixture_graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.entities.push(Entity {
        id: "Order".into(),
        label: "Order".into(),
        context: "Catalog".into(),
        definition: "A customer order".into(),
        identity: None,
        is_aggregate_root: true,
        attributes: vec![Attribute { name: "customer-id".into(), ty: Some("string".into()) }],
    });
    g.value_objects.push(ValueObject {
        id: "Money".into(),
        label: "Money".into(),
        context: "Catalog".into(),
        definition: Some("An amount in cents".into()),
    });
    g.reference_sets.push(ReferenceSet {
        id: "shipping-method".into(),
        label: Some("Shipping method".into()),
        concept: "Order".into(),
        values: vec!["standard".into(), "express-2day".into()],
    });
    g.commands.push(crate::pf::model::Command {
        id: "PlaceOrder".into(),
        label: "Place order".into(),
        context: "Catalog".into(),
        targets: "Order".into(),
        emits: vec!["OrderPlaced".into()],
    });
    g.events.push(crate::pf::model::Event {
        id: "OrderPlaced".into(),
        label: "Order placed".into(),
        context: "Catalog".into(),
        changes: "Order".into(),
    });
    g
}

fn fixture_logic() -> DeciderLogic {
    DeciderLogic {
        initial: payload(&[("status", Scalar::Str("new".into()))]),
        evolve: vec![EvolveRule {
            on: "OrderPlaced".into(),
            set: payload(&[
                ("status", Scalar::Str("placed".into())),
                ("amount", Scalar::Str("=event.amount".into())),
            ]),
        }],
        decide: vec![DecideRule {
            on: "PlaceOrder".into(),
            guards: vec![Guard {
                when: None,
                expr: Some("command.amount > 0".into()),
                else_reject: "inv-positive-amount".into(),
            }],
            emit: vec![EventRef::Data {
                event: "OrderPlaced".into(),
                with: payload(&[("amount", Scalar::Str("=command.amount".into()))]),
            }],
        }],
    }
}

fn fixture_scenarios() -> Vec<Scenario> {
    let place = |amount: i64| CommandRef::Data {
        command: "PlaceOrder".into(),
        with: payload(&[("amount", Scalar::Int(amount))]),
    };
    vec![
        Scenario {
            name: "order accepted".into(),
            given: vec![],
            when: place(5),
            then: Expectation::emit(vec![EventRef::Data {
                event: "OrderPlaced".into(),
                with: payload(&[("amount", Scalar::Int(5))]),
            }]),
        },
        Scenario {
            name: "non-positive rejected".into(),
            given: vec![],
            when: place(0),
            then: Expectation::reject("inv-positive-amount"),
        },
    ]
}

fn fixture_decider() -> Decider {
    Decider {
        id: "order-decider".into(),
        decides_for: "Order".into(),
        handles: vec!["PlaceOrder".into()],
        emits: vec!["OrderPlaced".into()],
        evolves_from: vec!["OrderPlaced".into()],
        rejects: vec!["inv-positive-amount".into()],
        reads: vec![],
        logic: Some(fixture_logic()),
        scenarios: fixture_scenarios(),
    }
}

fn opts() -> ReifyOptions {
    ReifyOptions {
        product: "bookstore".into(),
        namespace: "Bookstore".into(),
        what_version: "1.0".into(),
    }
}

fn plan() -> ReifyPlan {
    plan_csharp(&fixture_graph(), &[fixture_decider()], &opts()).expect("plan")
}

fn file<'a>(p: &'a ReifyPlan, suffix: &str) -> &'a GenFile {
    p.files
        .iter()
        .find(|f| f.path.ends_with(suffix))
        .unwrap_or_else(|| panic!("no generated file ending in {suffix}"))
}

#[test]
fn identifiers_pascal_case_totally() {
    assert_eq!(pascal("cmd-add-bounded-context"), "CmdAddBoundedContext");
    assert_eq!(pascal("PlaceOrder"), "PlaceOrder");
    assert_eq!(pascal("2fa-code"), "N2FaCode");
    assert_eq!(pascal(""), "X");
    assert_eq!(method_name("non-positive rejected"), "Non_positive_rejected");
}

#[test]
fn inference_recovers_payload_and_state_fields() {
    let shape = crate::pf::reify_infer::infer(&fixture_decider());
    let cmd = shape.commands.get("PlaceOrder").expect("command entry");
    assert_eq!(cmd.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
    let ev = shape.events.get("OrderPlaced").expect("event entry");
    assert_eq!(ev.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
    // status is typed from initial; amount is a plain `=event.amount` copy,
    // so it inherits the event field's type once scenarios pin it.
    assert_eq!(shape.state.get("status"), Some(&Some(crate::pf::reify_ident::CsTy::Str)));
    assert_eq!(shape.state.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
}

#[test]
fn typed_contracts_are_emitted() {
    let p = plan();
    let types = &file(&p, "Order/OrderTypes.g.cs").content;
    assert!(types.contains("public sealed record PlaceOrder(long? Amount = null) : IOrderCommand"));
    assert!(types.contains("public string WireId => \"PlaceOrder\";"));
    assert!(types.contains("public sealed record OrderPlaced(long? Amount = null) : IOrderEvent"));
    assert!(types.contains("if (Amount is not null) d[\"amount\"] = Amount;"));
    assert!(types.contains("public string Status { get; init; } = \"placed\";") == false);
    assert!(types.contains("public string Status { get; init; } = \"new\";"));
    let domain = &file(&p, "Domain.g.cs").content;
    assert!(domain.contains("public sealed record Order(string? CustomerId = null);"));
    assert!(domain.contains("public sealed record Money;"));
    assert!(domain.contains("public enum ShippingMethod"));
    assert!(domain.contains("Express2Day,"));
}

#[test]
fn decider_frame_and_wire_codec_are_emitted() {
    let p = plan();
    let frame = &file(&p, "OrderDecider.g.cs").content;
    assert!(frame.contains("public static partial DecisionResult Decide(OrderState state, IOrderCommand command);"));
    assert!(frame.contains("public static partial OrderState Evolve(OrderState state, IOrderEvent evt);"));
    assert!(frame.contains("public static readonly string[] Handles = { \"PlaceOrder\" };"));
    let wire = &file(&p, "OrderWire.g.cs").content;
    assert!(wire.contains("\"PlaceOrder\" => new PlaceOrder(Amount: PfJson.GetLong(with, \"amount\")),"));
    let program = &file(&p, "Program.g.cs").content;
    assert!(program.contains("\"order-decider\" => RunOrder(request),"));
    assert!(program.contains("static Dictionary<string, object?> RunOrder(JsonElement request)"));
}

#[test]
fn scenarios_become_xunit_facts() {
    let p = plan();
    let tests = &file(&p, "OrderScenarioTests.g.cs").content;
    assert!(tests.contains("public void Order_accepted()"));
    assert!(tests.contains("var state = OrderDecider.InitialState();"));
    assert!(tests.contains("OrderDecider.Decide(state, new PlaceOrder(Amount: 5L));"));
    assert!(tests.contains("Assert.Single(result.Events);"));
    assert!(tests.contains("Assert.Equal(5L, Assert.IsType<long>(with0[\"amount\"]));"));
    assert!(tests.contains("Assert.Equal(\"inv-positive-amount\", result.RejectedInvariant);"));
}

#[test]
fn stub_is_scaffold_only_and_excluded_from_manifest() {
    let p = plan();
    let stub = file(&p, "Order/OrderDecider.cs");
    assert!(!stub.overwrite);
    assert!(stub.content.contains("NotImplementedException"));
    let prov = &file(&p, "provenance.g.json").content;
    assert!(!prov.contains("OrderDecider.cs\""));
    assert!(prov.contains("OrderDecider.g.cs"));
}

#[test]
fn provenance_pins_the_input_hash() {
    let p = plan();
    let prov = &file(&p, "provenance.g.json").content;
    assert_eq!(recorded_hash(prov).expect("hash"), p.graph_hash);
    let cs = &file(&p, "Provenance.g.cs").content;
    assert!(cs.contains(&format!("sha256:{}", p.graph_hash)));
    assert!(cs.contains("AssemblyMetadata(\"PF.WhatVersion\", \"1.0\")"));
}

#[test]
fn generation_is_deterministic_and_hash_tracks_the_graph() {
    let a = plan();
    let b = plan();
    assert_eq!(a.graph_hash, b.graph_hash);
    for (fa, fb) in a.files.iter().zip(&b.files) {
        assert_eq!(fa.path, fb.path);
        assert_eq!(fa.content, fb.content, "non-deterministic file {}", fa.path);
    }
    let mut moved = fixture_graph();
    moved.events.push(crate::pf::model::Event {
        id: "OrderCancelled".into(),
        label: "Order cancelled".into(),
        context: "Catalog".into(),
        changes: "Order".into(),
    });
    let c = plan_csharp(&moved, &[fixture_decider()], &opts()).expect("plan");
    assert_ne!(a.graph_hash, c.graph_hash);
}

#[test]
fn duplicate_aggregate_deciders_are_rejected() {
    let mut d2 = fixture_decider();
    d2.id = "order-decider-2".into();
    let err = plan_csharp(&fixture_graph(), &[fixture_decider(), d2], &opts());
    assert!(err.is_err());
}
