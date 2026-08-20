//! The §5.2 extraction table as data — one row per code structure.
//!
//! Each row names the operations it needs, the assurance grade its output
//! carries, and the provenance a Reading taken through it gets. A row is
//! never silently empty: it either fired, found nothing, could not be
//! derived from the declaration subset at all, or was gated off because an
//! operation did not answer. Those four outcomes are different findings
//! and the report keeps them apart.

use serde::Serialize;

/// The assurance grade a row's output carries (§5.2's third column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Grade {
    /// Declared by the language: the semantics say what it is.
    High,
    /// Structural, or inferred from usage.
    Mid,
    /// Heuristic.
    Low,
    /// Not obtainable from the declaration surface at any grade.
    NotDerivable,
}

impl Grade {
    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::High => "high",
            Grade::Mid => "mid",
            Grade::Low => "low",
            Grade::NotDerivable => "not-derivable",
        }
    }
}

/// The §4.1 provenance vocabulary, track-owned per `g-dec-01`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provenance {
    Controlled,
    Observed,
    Inferred,
    Institutional,
}

impl Provenance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provenance::Controlled => "controlled",
            Provenance::Observed => "observed",
            Provenance::Inferred => "inferred",
            Provenance::Institutional => "institutional",
        }
    }
}

/// One row of the extraction table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Row {
    /// Stable id, used in the assurance field so a reviewer can query by row.
    pub id: &'static str,
    /// The code structure this row reads (§5.2 column 1).
    pub structure: &'static str,
    /// What it proposes (§5.2 column 2).
    pub output: &'static str,
    pub grade: Grade,
    pub provenance: Provenance,
    /// The declaration-level operations this row cannot fire without.
    pub operations: &'static [&'static str],
    /// The method, recorded in every Reading's assurance field.
    pub derivation: &'static str,
}

/// Every row of the table, in the spec's order.
pub const ROWS: [Row; 7] = [
    Row {
        id: "classes",
        structure: "classes, records, entities, aggregates",
        output: "owl:Class candidates",
        grade: Grade::High,
        provenance: Provenance::Observed,
        operations: &["documentSymbol"],
        derivation: "mapping rule over declared types",
    },
    Row {
        id: "subclass",
        structure: "inheritance, interface implementation",
        output: "rdfs:subClassOf",
        grade: Grade::High,
        provenance: Provenance::Observed,
        operations: &["documentSymbol", "typeHierarchy"],
        derivation: "mapping rule over typeHierarchy, both directions",
    },
    Row {
        id: "properties",
        structure: "typed properties",
        output: "datatype and object properties with ranges",
        grade: Grade::High,
        provenance: Provenance::Observed,
        operations: &["documentSymbol"],
        derivation: "mapping rule over declared member types",
    },
    Row {
        id: "foreign-keys",
        structure: "foreign keys, relational structure",
        output: "object properties between entities",
        grade: Grade::NotDerivable,
        provenance: Provenance::Observed,
        operations: &[],
        derivation: "none — outside the declaration surface",
    },
    Row {
        id: "usage-relations",
        structure: "composition and reference",
        output: "relations between declared types",
        grade: Grade::Mid,
        provenance: Provenance::Inferred,
        operations: &["documentSymbol", "references"],
        derivation: "SPARQL CONSTRUCT over raw reference edges, to fixpoint",
    },
    Row {
        id: "modules",
        structure: "namespaces, modules, bounded contexts",
        output: "domain axis registries",
        grade: Grade::Mid,
        provenance: Provenance::Observed,
        operations: &["documentSymbol"],
        derivation: "mapping rule over declared module containment",
    },
    Row {
        id: "synonyms",
        structure: "naming, comments, string usage",
        output: "synonyms, skos:altLabel",
        grade: Grade::Low,
        provenance: Provenance::Inferred,
        operations: &["documentSymbol"],
        derivation: "lexical rule: identifier word split; no model",
    },
];

/// Why the foreign-key row proposes nothing, stated so the gap is a limit
/// rather than a silence.
pub const FOREIGN_KEY_REASON: &str = "relational structure lives in ORM \
configuration and generated migrations, outside the declaration surface the \
extractor reads; naming it here keeps the absence reported rather than silent, \
and synthesising it from those sources is out of G0 scope";

/// What became of a row on one corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum Outcome {
    /// Proposed this many assertions.
    Fired { triples: usize },
    /// Every operation answered; the codebase has nothing of this shape.
    FoundNothing,
    /// Structurally unobtainable from the declaration subset.
    NotDerivable { reason: &'static str },
    /// Gated off: an operation the row needs did not answer.
    Unavailable { operations: Vec<String> },
}

impl Outcome {
    /// Did this row contribute assertions?
    pub fn fired(&self) -> usize {
        match self {
            Outcome::Fired { triples } => *triples,
            _ => 0,
        }
    }

    /// A one-line statement of the outcome, for the report.
    pub fn describe(&self) -> String {
        match self {
            Outcome::Fired { triples } => format!("fired — {triples} assertion(s)"),
            Outcome::FoundNothing => {
                "found nothing — every operation answered, the codebase has none".to_string()
            }
            Outcome::NotDerivable { .. } => "not derivable from the declaration subset".to_string(),
            Outcome::Unavailable { operations } => {
                format!("UNAVAILABLE — {} did not answer", operations.join(", "))
            }
        }
    }
}

/// The row with the given id.
pub fn row(id: &str) -> Option<&'static Row> {
    ROWS.iter().find(|r| r.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_ids_are_unique() {
        let mut ids: Vec<&str> = ROWS.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), before);
    }

    /// A row whose provenance is `inferred` must not claim high assurance:
    /// inference is not declaration, and grading it as one would be the
    /// exact overclaim per-triple review is meant to catch.
    #[test]
    fn no_inferred_row_claims_declared_assurance() {
        for r in ROWS.iter().filter(|r| r.provenance == Provenance::Inferred) {
            assert_ne!(r.grade, Grade::High, "{}", r.id);
        }
    }

    /// Every row that can fire names at least one operation, so gating is
    /// possible; the not-derivable row names none, because no operation
    /// would help.
    #[test]
    fn only_the_not_derivable_row_needs_no_operation() {
        for r in &ROWS {
            let needs = !r.operations.is_empty();
            assert_eq!(needs, r.grade != Grade::NotDerivable, "{}", r.id);
        }
    }

    #[test]
    fn outcomes_read_differently_from_one_another() {
        let described = [
            Outcome::Fired { triples: 3 }.describe(),
            Outcome::FoundNothing.describe(),
            Outcome::NotDerivable { reason: FOREIGN_KEY_REASON }.describe(),
            Outcome::Unavailable { operations: vec!["typeHierarchy".into()] }.describe(),
        ];
        let mut seen: Vec<&String> = described.iter().collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 4);
        assert_eq!(Outcome::FoundNothing.fired(), 0);
    }
}
