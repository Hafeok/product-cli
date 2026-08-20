//! The declared rows: classes, subclass edges, typed properties.
//!
//! High assurance because the language semantics say what these are — the
//! server returns a declaration and the mapping is a rule, not a judgement.
//! Each row is gated on the operations it needs: if one did not answer, the
//! row yields nothing *and says so*, which is the difference between a
//! codebase with no hierarchy and an instrument that did not reply.

use std::collections::BTreeSet;

use super::derivation::Derivation;
use super::facts::{CorpusFacts, MemberKind};
use super::mint::{class_iri, property_iri};
use super::propose::{cite, Proposer};
use super::rows::{row, Outcome, Row};
use super::triple::{ProposedAssertion, Term, Triple};
use super::vocab::{
    self, OWL_CLASS, OWL_DATATYPE_PROPERTY, OWL_OBJECT_PROPERTY, RDFS_DOMAIN, RDFS_RANGE,
    RDFS_SUBCLASS_OF, RDF_TYPE,
};

/// What a declared row produced, beside its assertions.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeclaredNotes {
    /// Supertypes named by an edge but not declared in the loaded subset —
    /// the restorable-subgraph limit, made countable.
    pub supertypes_outside_subset: BTreeSet<String>,

    /// Properties whose type resolved to neither a declared type nor a
    /// mapped primitive, so they carry no range.
    pub unranged_properties: Vec<String>,
    /// Properties whose declaration carried a nullability marker. Read, and
    /// deliberately not mapped: expressing it needs OWL cardinality
    /// restrictions, which are outside this row's stated output.
    pub nullable_properties: usize,
    /// Properties derived from a member the server reported as a *field*.
    /// The row cannot tell implementation from structure, so it proposes
    /// both; naming them lets a reviewer batch what the row could not split.
    pub field_derived: Vec<String>,
}

/// Gate a row on the operations it needs, returning the missing ones.
pub fn gate(facts: &CorpusFacts, r: &Row) -> Option<Outcome> {
    let missing: Vec<String> = r
        .operations
        .iter()
        .filter(|op| !facts.answered(op))
        .map(|op| (*op).to_string())
        .collect();
    (!missing.is_empty()).then_some(Outcome::Unavailable { operations: missing })
}

/// Row `classes`: every declared type is an `owl:Class` candidate.
pub fn classes(base: &str, facts: &CorpusFacts, p: &Proposer) -> (Vec<ProposedAssertion>, Outcome) {
    let Some(r) = row("classes") else { return (Vec::new(), Outcome::FoundNothing) };
    if let Some(blocked) = gate(facts, r) {
        return (Vec::new(), blocked);
    }
    let mut out = Vec::new();
    for decl in &facts.declarations {
        let triple = Triple::new(class_iri(base, &decl.name), RDF_TYPE, Term::iri(OWL_CLASS));
        out.push(p.propose(
            r,
            triple,
            vec![format!("{} ({})", cite(&decl.file, decl.start_line), decl.kind.as_str())],
            Derivation::by("declared-type", r.operations).in_container(decl.kind, &decl.file),
        ));
    }
    settle(out)
}

/// Row `subclass`: a declared base list becomes `rdfs:subClassOf`.
pub fn subclass(
    base: &str,
    facts: &CorpusFacts,
    p: &Proposer,
    notes: &mut DeclaredNotes,
) -> (Vec<ProposedAssertion>, Outcome) {
    let Some(r) = row("subclass") else { return (Vec::new(), Outcome::FoundNothing) };
    if let Some(blocked) = gate(facts, r) {
        return (Vec::new(), blocked);
    }
    let declared = facts.declared_names();
    let mut out = Vec::new();
    for edge in &facts.hierarchy {
        if !declared.contains(edge.sup.as_str()) {
            notes.supertypes_outside_subset.insert(edge.sup.clone());
        }
        let triple = Triple::new(
            class_iri(base, &edge.sub),
            RDFS_SUBCLASS_OF,
            Term::iri(class_iri(base, &edge.sup)),
        );
        let sub = facts.declarations.iter().find(|d| d.name == edge.sub);
        let mut derivation = Derivation::by("declared-supertype", r.operations);
        if let Some(d) = sub {
            derivation = derivation.in_container(d.kind, &d.file);
        }
        out.push(p.propose(r, triple, evidence_for(facts, &edge.sub), derivation));
    }
    settle(out)
}

/// Row `properties`: a declared member becomes a property with a range.
///
/// A member whose type is a declared type of this corpus is an **object**
/// property; one whose type is a mapped primitive is a **datatype**
/// property; anything else is a property with no range and a note.
pub fn properties(
    base: &str,
    facts: &CorpusFacts,
    p: &Proposer,
    notes: &mut DeclaredNotes,
) -> (Vec<ProposedAssertion>, Outcome) {
    let Some(r) = row("properties") else { return (Vec::new(), Outcome::FoundNothing) };
    if let Some(blocked) = gate(facts, r) {
        return (Vec::new(), blocked);
    }
    let declared = facts.declared_names();
    let mut out = Vec::new();
    for decl in &facts.declarations {
        for prop in &decl.properties {
            let iri = property_iri(base, &decl.name, &prop.name);
            let evidence = vec![cite(&decl.file, decl.start_line), prop.type_name.clone()];
            if prop.nullable() {
                notes.nullable_properties += 1;
            }
            if prop.kind == MemberKind::Field {
                notes.field_derived.push(format!("{}.{}", decl.name, prop.name));
            }
            let (kind, range) = classify(base, prop.bare_type(), &declared);
            if range.is_none() {
                notes.unranged_properties.push(format!("{}.{}", decl.name, prop.name));
            }
            let member = Derivation::by("declared-member", r.operations)
                .in_container(decl.kind, &decl.file)
                .of_member(prop.kind);
            out.push(p.propose(
                r,
                Triple::new(iri.clone(), RDF_TYPE, Term::iri(kind)),
                evidence.clone(),
                member.clone(),
            ));
            out.push(p.propose(
                r,
                Triple::new(iri.clone(), RDFS_DOMAIN, Term::iri(class_iri(base, &decl.name))),
                evidence.clone(),
                member.clone(),
            ));
            if let Some(range) = range {
                let ranged = Derivation::by("member-range", r.operations)
                    .in_container(decl.kind, &decl.file)
                    .of_member(prop.kind);
                out.push(p.propose(
                    r,
                    Triple::new(iri, RDFS_RANGE, Term::iri(range)),
                    evidence,
                    ranged,
                ));
            }
        }
    }
    settle(out)
}

/// Which property kind a member's type makes it, with the range if any.
fn classify(base: &str, bare: &str, declared: &BTreeSet<&str>) -> (&'static str, Option<String>) {
    if declared.contains(bare) {
        return (OWL_OBJECT_PROPERTY, Some(class_iri(base, bare)));
    }
    match vocab::primitive_datatype(bare) {
        Some(dt) => (OWL_DATATYPE_PROPERTY, Some(dt)),
        None => (OWL_DATATYPE_PROPERTY, None),
    }
}

/// Where a declared type is declared, for a citation.
fn evidence_for(facts: &CorpusFacts, name: &str) -> Vec<String> {
    facts
        .declarations
        .iter()
        .find(|d| d.name == name)
        .map(|d| vec![cite(&d.file, d.start_line)])
        .unwrap_or_default()
}

/// Deduplicate, then read the outcome off what survived. An empty result
/// past the gate is `FoundNothing`: every operation answered, the codebase
/// has none of this shape.
pub fn settle(proposals: Vec<ProposedAssertion>) -> (Vec<ProposedAssertion>, Outcome) {
    let out = dedup(proposals);
    let outcome =
        if out.is_empty() { Outcome::FoundNothing } else { Outcome::Fired { triples: out.len() } };
    (out, outcome)
}

/// Collapse proposals that mint the same id — the same triple read twice is
/// one assertion, and one file.
pub fn dedup(mut proposals: Vec<ProposedAssertion>) -> Vec<ProposedAssertion> {
    proposals.sort_by(|a, b| a.id.cmp(&b.id));
    proposals.dedup_by(|a, b| a.id == b.id);
    proposals
}

#[cfg(test)]
#[path = "declared_tests.rs"]
mod tests;
