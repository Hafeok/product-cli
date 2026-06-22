//! The §3.1 data-side node model — reference sets, validatable shapes, datasets.
//!
//! These are the "data" half of the structure/data split: constitutive
//! reference data that is part of the What, the SHACL-property shapes the
//! structure is made checkable as, and the production datasets that serve as
//! the oracle data conformance (§6.3) validates the shapes against.

use serde::{Deserialize, Serialize};

/// §3.1 — constitutive **reference data**: a named, governed set of values the
/// behaviour depends on (valid shipping methods, tax categories). It is part of
/// the What; `concept` is the entity/value-object it is reference data for.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ReferenceSet {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The concept (`reference_data_for`) this set is constitutive of.
    pub concept: String,
    /// The declared members — the closed set the behaviour may reference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

/// §3.1 — one field constraint within a [`DataShape`]: a `field` that may be
/// `required` (present + non-null) and/or constrained to a [`ReferenceSet`]'s
/// membership (`reference_set`). The SHACL-property side of the structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EnumConstraint {
    pub field: String,
    pub reference_set: String,
}

/// §3.1 — a datatype constraint: a `field` whose value must be of `datatype`
/// (`string` · `integer` · `number` · `boolean` · `date`). Catches type drift
/// the structure side cannot otherwise see.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TypeConstraint {
    pub field: String,
    pub datatype: String,
}

/// §3.1 — a validatable **shape** over an entity: the structure side made
/// machine-checkable. `target` is the entity it shapes; `required` lists the
/// fields production records must carry; `enums` constrains fields to a declared
/// reference set; `types` constrains fields to a datatype. Production data is
/// validated against it as an oracle (§6.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DataShape {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The entity this shape constrains (`conforms_to_shape` target).
    pub target: String,
    /// Fields every conforming record must carry, present and non-null.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    /// Fields whose value must be a member of a declared reference set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<EnumConstraint>,
    /// Fields whose value must be of a declared datatype.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeConstraint>,
}

/// §3.1 — a **production dataset**: the oracle the structure is checked against.
/// Not specification; `shape` names the [`DataShape`] it `conforms_to_shape`,
/// and `source` points at the populated records (a JSON file of objects).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProductionDataset {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The shape this dataset is validated against (the oracle relation).
    pub shape: String,
    /// Path to the populated records — a JSON array of objects.
    pub source: String,
}
