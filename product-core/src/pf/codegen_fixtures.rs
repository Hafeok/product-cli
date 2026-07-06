//! Shared test fixtures for the codegen slice tests.

#![allow(dead_code)] // each consumer uses a subset

use std::collections::BTreeMap;

use crate::pf::codegen::{plan_csharp, GenFile, ReifyOptions, ReifyPlan};
use crate::pf::decider::Decider;
use crate::pf::model::DomainGraph;
use crate::pf::decider_logic::{
    CommandRef, DecideRule, DeciderLogic, EventRef, EvolveRule, Expectation, Guard, Scalar,
    Scenario,
};
use crate::pf::model::{Attribute, Entity, ReferenceSet, ValueObject};


pub fn payload(pairs: &[(&str, Scalar)]) -> BTreeMap<String, Scalar> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

pub fn fixture_graph() -> DomainGraph {
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
    behaviour_nodes(&mut g);
    g
}

fn behaviour_nodes(g: &mut DomainGraph) {
    g.commands.push(crate::pf::model::Command {
        fields: vec![],
        id: "PlaceOrder".into(),
        label: "Place order".into(),
        context: "Catalog".into(),
        targets: "Order".into(),
        emits: vec!["OrderPlaced".into()],
    });
    g.events.push(crate::pf::model::Event {
        fields: vec![],
        id: "OrderPlaced".into(),
        label: "Order placed".into(),
        context: "Catalog".into(),
        changes: "Order".into(),
    });
    g.flows.push(crate::pf::model_ui::Flow {
        id: "flow-buy".into(),
        label: "Buy a book".into(),
        steps: vec!["PlaceOrder".into(), "OrderPlaced".into(), "OrderSummary".into()],
        entry_page: None,
        system: None,
    });
    g.wireframe_steps.push(checkout_step());
}

fn checkout_step() -> crate::pf::model_ui::WireframeStep {
    use crate::pf::model_ui::{Offer, StateMeaning, Surface};
    crate::pf::model_ui::WireframeStep {
        id: "ui-checkout".into(),
        label: "Checkout".into(),
        surfaces: vec![Surface { projection: "OrderSummary".into(), aio: "display-collection".into() }],
        offers: vec![Offer { command: "PlaceOrder".into(), aio: "trigger-action".into() }],
        state_meanings: vec![
            StateMeaning {
                projection: "OrderSummary".into(),
                state: "empty".into(),
                meaning: Some("no orders yet".into()),
                waiver: None,
            },
            StateMeaning {
                projection: "OrderSummary".into(),
                state: "failed".into(),
                meaning: None,
                waiver: Some("kiosk cannot fail".into()),
            },
        ],
        ..Default::default()
    }
}

pub fn fixture_logic() -> DeciderLogic {
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

pub fn fixture_scenarios() -> Vec<Scenario> {
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

pub fn fixture_decider() -> Decider {
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

pub fn opts() -> ReifyOptions {
    ReifyOptions {
        product: "bookstore".into(),
        namespace: "Bookstore".into(),
        what_version: "1.0".into(),
        oracle_only: false,
        design_system: None,
    }
}

pub fn fixture_projector() -> crate::pf::projector::Projector {
    use crate::pf::projector_logic::{ProjectorLogic, ProjectorScenario};
    crate::pf::projector::Projector {
        id: "ordersummary-projector".into(),
        projects_for: "OrderSummary".into(),
        folds: vec!["OrderPlaced".into()],
        over: vec!["Order".into()],
        logic: Some(ProjectorLogic {
            initial: payload(&[("count", Scalar::Int(0))]),
            apply: vec![EvolveRule {
                on: "OrderPlaced".into(),
                set: payload(&[
                    ("count", Scalar::Str("=view.count + 1".into())),
                    ("last_amount", Scalar::Str("=event.amount".into())),
                ]),
            }],
        }),
        scenarios: vec![ProjectorScenario {
            name: "one order counted".into(),
            given: vec![EventRef::Data {
                event: "OrderPlaced".into(),
                with: payload(&[("amount", Scalar::Int(5))]),
            }],
            then: payload(&[("count", Scalar::Int(1)), ("last_amount", Scalar::Int(5))]),
        }],
    }
}

pub fn plan() -> ReifyPlan {
    plan_csharp(&fixture_graph(), &[fixture_decider()], &[fixture_projector()], &opts()).expect("plan")
}

pub fn oracle_plan() -> ReifyPlan {
    let o = ReifyOptions { oracle_only: true, ..opts() };
    plan_csharp(&fixture_graph(), &[fixture_decider()], &[fixture_projector()], &o).expect("plan")
}

pub fn file<'a>(p: &'a ReifyPlan, suffix: &str) -> &'a GenFile {
    p.files
        .iter()
        .find(|f| f.path.ends_with(suffix))
        .unwrap_or_else(|| panic!("no generated file ending in {suffix}"))
}
