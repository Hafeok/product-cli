//! §3.1 data-side conformance rules over the structure/data split.
//!
//! Splits like the rest of the checker: small presence checks (a shape needs a
//! target, a dataset needs a shape + source) run in-loop on the fragment, while
//! the cross-references (a reference set's concept exists, a shape targets a
//! real entity, a dataset's shape exists) run over the whole graph — so a node
//! can be authored before the concept it points at, then caught by `validate`.

use super::model::{DataShape, DomainGraph, ProductionDataset, ReferenceSet};
use super::validate::Violation;

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message: message.to_string(), severity: "violation".to_string() }
}

/// In-loop presence checks for a reference set (no cross-reference).
pub fn check_reference_set(rs: &ReferenceSet, out: &mut Vec<Violation>) {
    if rs.concept.trim().is_empty() {
        out.push(v(&rs.id, "concept",
            "§3.1 Reference data must declare the concept it is constitutive of (reference_data_for)."));
    }
    if rs.values.iter().all(|x| x.trim().is_empty()) {
        out.push(v(&rs.id, "values",
            "§3.1 A reference set must declare at least one value (the closed set behaviour draws from)."));
    }
}

/// The datatypes a type constraint may declare (the engine's machine gates).
const DATATYPES: [&str; 5] = ["string", "integer", "number", "boolean", "date"];

/// In-loop presence checks for a data shape (no cross-reference).
pub fn check_data_shape(s: &DataShape, out: &mut Vec<Violation>) {
    if s.target.trim().is_empty() {
        out.push(v(&s.id, "target",
            "§3.1 A data shape must target an entity (the structure it makes checkable)."));
    }
    for c in &s.types {
        if !DATATYPES.contains(&c.datatype.as_str()) {
            out.push(v(&s.id, "types",
                "§3.1 A datatype constraint must be one of: string, integer, number, boolean, date."));
        }
    }
}

/// In-loop presence checks for a production dataset (no cross-reference).
pub fn check_dataset(d: &ProductionDataset, out: &mut Vec<Violation>) {
    if d.shape.trim().is_empty() {
        out.push(v(&d.id, "shape",
            "§3.1 A production dataset must name the shape it conforms_to_shape (the oracle relation)."));
    }
    if d.source.trim().is_empty() {
        out.push(v(&d.id, "source",
            "§3.1 A production dataset must point at its populated records (a JSON source)."));
    }
}

/// Whole-graph cross-reference checks: every data-side pointer resolves.
pub fn data_cross_refs(g: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    let is_concept = |id: &str| g.entities.iter().any(|e| e.id == id) || g.value_objects.iter().any(|n| n.id == id);
    for rs in &g.reference_sets {
        if !rs.concept.trim().is_empty() && !is_concept(&rs.concept) {
            out.push(v(&rs.id, "concept",
                "§3.1 Reference data must be constitutive of a real entity or value object."));
        }
    }
    for s in &g.data_shapes {
        if !s.target.trim().is_empty() && !g.entities.iter().any(|e| e.id == s.target) {
            out.push(v(&s.id, "target", "§3.1 A data shape must target a declared entity."));
        }
        for c in &s.enums {
            if !g.reference_sets.iter().any(|rs| rs.id == c.reference_set) {
                out.push(v(&s.id, "enums",
                    "§3.1 A field's enum constraint must reference a declared reference set."));
            }
        }
    }
    for d in &g.production_datasets {
        if !d.shape.trim().is_empty() && !g.data_shapes.iter().any(|s| s.id == d.shape) {
            out.push(v(&d.id, "shape", "§3.1 A production dataset must conform to a declared data shape."));
        }
    }
    out
}

#[cfg(test)]
#[path = "rules_data_tests.rs"]
mod tests;
