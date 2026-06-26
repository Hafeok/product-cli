//! Turtle emission for the §3.1 data-side nodes — reference sets, shapes,
//! datasets. Split from [`super::turtle`] for the 400-line gate.

use super::model;
use super::turtle::lit;

/// §3.1 — constitutive reference data: the closed set of values a concept allows.
pub(super) fn emit_reference_set(out: &mut String, rs: &model::ReferenceSet) {
    out.push_str(&format!("d:{} a pf:ReferenceSet ;\n  pf:referenceDataFor d:{}", rs.id, rs.concept));
    if let Some(l) = &rs.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for v in &rs.values {
        out.push_str(&format!(" ;\n  pf:referenceValue {}", lit(v)));
    }
    out.push_str(" .\n\n");
}

/// §3.1 — a validatable shape over an entity (required fields, enum + type
/// constraints), emitted with constraints as blank nodes.
pub(super) fn emit_data_shape(out: &mut String, s: &model::DataShape) {
    out.push_str(&format!("d:{} a pf:DataShape ;\n  pf:shapeTarget d:{}", s.id, s.target));
    if let Some(l) = &s.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    for r in &s.required {
        out.push_str(&format!(" ;\n  pf:requiredField {}", lit(r)));
    }
    for c in &s.enums {
        out.push_str(&format!(
            " ;\n  pf:enumConstraint [ pf:field {} ; pf:fromReferenceSet d:{} ]",
            lit(&c.field), c.reference_set
        ));
    }
    for c in &s.types {
        out.push_str(&format!(
            " ;\n  pf:typeConstraint [ pf:field {} ; pf:datatype {} ]",
            lit(&c.field), lit(&c.datatype)
        ));
    }
    out.push_str(" .\n\n");
}

/// §3.1 — a production dataset: the oracle the shape is checked against.
pub(super) fn emit_dataset(out: &mut String, d: &model::ProductionDataset) {
    out.push_str(&format!(
        "d:{} a pf:ProductionDataset ;\n  pf:conformsToShape d:{} ;\n  pf:dataSource {}",
        d.id, d.shape, lit(&d.source)
    ));
    if let Some(l) = &d.label {
        out.push_str(&format!(" ;\n  rdfs:label {}", lit(l)));
    }
    out.push_str(" .\n\n");
}
