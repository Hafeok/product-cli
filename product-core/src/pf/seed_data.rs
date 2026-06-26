//! Turtle parsing for the §3.1 data-side nodes — reference sets, shapes,
//! datasets. Peer of [`super::turtle_data`]; split from [`super::seed`].

use std::collections::HashMap;

use oxigraph::store::Store;

use super::model::*;
use super::seed::{lit, local, multi, opt, select};
use crate::error::Result;

/// Parse every data-side kind into `g`.
pub(super) fn parse_data(store: &Store, g: &mut DomainGraph) -> Result<()> {
    parse_reference_sets(store, g)?;
    parse_data_shapes(store, g)?;
    parse_datasets(store, g)?;
    Ok(())
}

fn parse_reference_sets(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let values = multi(store, "pf:ReferenceSet", "pf:referenceValue")?;
    for row in select(store, "?s ?label ?concept",
        "?s a pf:ReferenceSet . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:referenceDataFor ?concept }")? {
        let id = local(row.get("s"));
        g.reference_sets.push(ReferenceSet {
            values: values.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: opt(row.get("label")), concept: local(row.get("concept")),
        });
    }
    Ok(())
}

fn parse_data_shapes(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let required = multi(store, "pf:DataShape", "pf:requiredField")?;
    let mut enums: HashMap<String, Vec<EnumConstraint>> = HashMap::new();
    for row in select(store, "?s ?field ?set",
        "?s a pf:DataShape ; pf:enumConstraint ?b . ?b pf:field ?field ; pf:fromReferenceSet ?set")? {
        enums.entry(local(row.get("s"))).or_default()
            .push(EnumConstraint { field: lit(row.get("field")), reference_set: local(row.get("set")) });
    }
    let mut types: HashMap<String, Vec<TypeConstraint>> = HashMap::new();
    for row in select(store, "?s ?field ?dt",
        "?s a pf:DataShape ; pf:typeConstraint ?b . ?b pf:field ?field ; pf:datatype ?dt")? {
        types.entry(local(row.get("s"))).or_default()
            .push(TypeConstraint { field: lit(row.get("field")), datatype: lit(row.get("dt")) });
    }
    for row in select(store, "?s ?label ?target",
        "?s a pf:DataShape . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:shapeTarget ?target }")? {
        let id = local(row.get("s"));
        g.data_shapes.push(DataShape {
            required: required.get(&id).cloned().unwrap_or_default(),
            enums: enums.remove(&id).unwrap_or_default(),
            types: types.remove(&id).unwrap_or_default(),
            id: id.clone(), label: opt(row.get("label")), target: local(row.get("target")),
        });
    }
    Ok(())
}

fn parse_datasets(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label ?shape ?source",
        "?s a pf:ProductionDataset . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:conformsToShape ?shape } OPTIONAL { ?s pf:dataSource ?source }")? {
        g.production_datasets.push(ProductionDataset {
            id: local(row.get("s")), label: opt(row.get("label")),
            shape: local(row.get("shape")), source: lit(row.get("source")),
        });
    }
    Ok(())
}
