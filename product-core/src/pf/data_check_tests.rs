//! Unit tests for the §6.3 data-conformance engine.

use super::*;
use crate::pf::model::*;
use serde_json::json;

fn graph() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.entities.push(Entity { id: "Order".into(), label: "Order".into(), context: "Sales".into(), definition: "an order".into(), ..Default::default() });
    g.reference_sets.push(ReferenceSet {
        id: "ShippingMethods".into(),
        label: None,
        concept: "Order".into(),
        values: vec!["standard".into(), "express".into()],
    });
    g.data_shapes.push(DataShape {
        id: "OrderShape".into(),
        target: "Order".into(),
        required: vec!["id".into(), "total".into()],
        enums: vec![EnumConstraint { field: "shipping".into(), reference_set: "ShippingMethods".into() }],
        types: vec![TypeConstraint { field: "total".into(), datatype: "integer".into() }],
        ..Default::default()
    });
    g.production_datasets.push(ProductionDataset {
        id: "OrdersLive".into(),
        label: None,
        shape: "OrderShape".into(),
        source: "orders.json".into(),
    });
    g
}

#[test]
fn clean_data_has_zero_divergence() {
    let g = graph();
    let records = vec![
        json!({ "id": "o1", "total": 10, "shipping": "standard" }),
        json!({ "id": "o2", "total": 20, "shipping": "express" }),
    ];
    let v = check_dataset(&g, "OrdersLive", &records).expect("verdict");
    assert!(v.conformant());
    assert_eq!(v.total, 2);
    assert_eq!(v.violating, 0);
    assert_eq!(v.divergence_rate, 0.0);
}

#[test]
fn missing_required_field_is_caught() {
    let g = graph();
    // The §3.1 example: a field that is null/absent in production rows.
    let records = vec![
        json!({ "id": "o1", "shipping": "standard" }),       // total missing
        json!({ "id": "o2", "total": null, "shipping": "express" }), // total null
    ];
    let v = check_dataset(&g, "OrdersLive", &records).expect("verdict");
    assert_eq!(v.violating, 2);
    assert_eq!(v.divergence_rate, 1.0);
    assert!(v.findings.iter().all(|f| f.kind == "missing-required" && f.field == "total"));
}

#[test]
fn enum_value_the_schema_never_declared_is_caught() {
    let g = graph();
    // The §3.1 example: an enum value the declared set never sanctioned.
    let records = vec![
        json!({ "id": "o1", "total": 10, "shipping": "standard" }),
        json!({ "id": "o2", "total": 20, "shipping": "drone" }),
    ];
    let v = check_dataset(&g, "OrdersLive", &records).expect("verdict");
    assert_eq!(v.violating, 1);
    assert_eq!(v.divergence_rate, 0.5);
    let f = v.findings.iter().find(|f| f.field == "shipping").expect("finding");
    assert_eq!(f.kind, "not-in-reference-set");
    assert_eq!(f.record, 1);
}

#[test]
fn empty_dataset_is_zero_divergence_not_a_panic() {
    let g = graph();
    let v = check_dataset(&g, "OrdersLive", &[]).expect("verdict");
    assert_eq!(v.total, 0);
    assert_eq!(v.divergence_rate, 0.0);
    assert!(v.conformant());
}

#[test]
fn unknown_dataset_errs() {
    let g = graph();
    assert!(check_dataset(&g, "ghost", &[]).is_err());
}

#[test]
fn wrong_datatype_is_caught() {
    let g = graph(); // OrderShape declares total: integer
    let records = vec![
        json!({ "id": "o1", "total": 10, "shipping": "standard" }),
        json!({ "id": "o2", "total": "twelve", "shipping": "express" }),
    ];
    let v = check_dataset(&g, "OrdersLive", &records).expect("verdict");
    assert_eq!(v.violating, 1);
    let f = v.findings.iter().find(|f| f.kind == "not-of-type").expect("type finding");
    assert_eq!(f.field, "total");
    assert_eq!(f.record, 1);
}

#[test]
fn trend_classifies_movement_against_the_previous_run() {
    assert_eq!(classify_trend(None, 0.2), DivergenceTrend::First);
    assert_eq!(classify_trend(Some(0.1), 0.3), DivergenceTrend::Rising);
    assert_eq!(classify_trend(Some(0.3), 0.1), DivergenceTrend::Falling);
    assert_eq!(classify_trend(Some(0.20), 0.205), DivergenceTrend::Stable);
}
