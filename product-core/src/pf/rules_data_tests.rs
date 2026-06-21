//! Unit tests for the §3.1 data-side conformance rules.

use super::*;
use crate::pf::model::{DataShape, EnumConstraint, Entity, ProductionDataset, ReferenceSet};

fn entity(g: &mut DomainGraph, id: &str) {
    g.entities.push(Entity { id: id.into(), label: id.into(), context: "C".into(), definition: "d".into(), ..Default::default() });
}

#[test]
fn reference_set_needs_concept_and_values() {
    let rs = ReferenceSet { id: "RS".into(), label: None, concept: "".into(), values: vec![] };
    let mut out = Vec::new();
    check_reference_set(&rs, &mut out);
    assert_eq!(out.len(), 2, "missing concept + values, got {out:?}");
}

#[test]
fn data_shape_needs_target() {
    let s = DataShape { id: "S".into(), target: "".into(), ..Default::default() };
    let mut out = Vec::new();
    check_data_shape(&s, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "target");
}

#[test]
fn dataset_needs_shape_and_source() {
    let d = ProductionDataset { id: "D".into(), label: None, shape: "".into(), source: "".into() };
    let mut out = Vec::new();
    check_dataset(&d, &mut out);
    assert_eq!(out.len(), 2);
}

#[test]
fn cross_refs_catch_dangling_pointers() {
    let mut g = DomainGraph::default();
    entity(&mut g, "Order");
    // concept points at a non-existent entity
    g.reference_sets.push(ReferenceSet { id: "RS".into(), label: None, concept: "Ghost".into(), values: vec!["a".into()] });
    // shape targets a real entity but enum points at a non-existent set
    g.data_shapes.push(DataShape { id: "S".into(), label: None, target: "Order".into(), required: vec![], enums: vec![EnumConstraint { field: "f".into(), reference_set: "NoSet".into() }] });
    // dataset references a non-existent shape
    g.production_datasets.push(ProductionDataset { id: "D".into(), label: None, shape: "NoShape".into(), source: "x.json".into() });
    let out = data_cross_refs(&g);
    assert_eq!(out.len(), 3, "one dangling ref each, got {out:?}");
}

#[test]
fn cross_refs_pass_when_pointers_resolve() {
    let mut g = DomainGraph::default();
    entity(&mut g, "Order");
    g.reference_sets.push(ReferenceSet { id: "RS".into(), label: None, concept: "Order".into(), values: vec!["a".into()] });
    g.data_shapes.push(DataShape { id: "S".into(), label: None, target: "Order".into(), required: vec![], enums: vec![EnumConstraint { field: "f".into(), reference_set: "RS".into() }] });
    g.production_datasets.push(ProductionDataset { id: "D".into(), label: None, shape: "S".into(), source: "x.json".into() });
    assert!(data_cross_refs(&g).is_empty());
}
