//! SHACL-style shape checks over the emitted graph.
//!
//! Cross-entry referential integrity the per-file gate cannot name: a
//! supersession edge must land on a filed decision, a parent hash must name
//! a filed version of the same decision, a version's decision must have
//! been introduced by some change-set. Each shape is one SPARQL SELECT
//! returning a row per violation, run by `product-core`'s engine (OD-2) —
//! zero rows means conformant, the `sh:SPARQLConstraint` reading.

use std::collections::BTreeMap;

use crate::store::Store;

use super::turtle::emit;
use super::{GraphClass, GraphFinding};

struct Shape {
    class: GraphClass,
    select: &'static str,
    message: fn(&BTreeMap<String, String>) -> (String, String),
}

/// Strip a raw term to a readable form: `<urn:dec:x/Y>` → `dec:x/Y`,
/// literals lose their quotes.
fn term(row: &BTreeMap<String, String>, var: &str) -> String {
    let raw = row.get(var).cloned().unwrap_or_default();
    let trimmed = raw.trim_start_matches('<').trim_end_matches('>');
    let trimmed = trimmed.strip_prefix("urn:").unwrap_or(trimmed);
    trimmed.trim_matches('"').to_string()
}

const SHAPES: &[Shape] = &[
    Shape {
        class: GraphClass::G001,
        select: "SELECT ?d ?target WHERE { \
                 ?v <urn:ledger:ns#supersedes> ?target . \
                 ?v <urn:ledger:ns#ofDecision> ?d . \
                 FILTER NOT EXISTS { ?target a <urn:ledger:ns#Decision> } }",
        message: |row| {
            (
                term(row, "d"),
                format!(
                    "supersedes {}, which is not a filed decision — a supersession target must exist",
                    term(row, "target")
                ),
            )
        },
    },
    Shape {
        class: GraphClass::G002,
        select: "SELECT ?d ?parent WHERE { \
                 ?v <urn:ledger:ns#ofDecision> ?d . \
                 ?v <http://www.w3.org/ns/prov#wasRevisionOf> ?parent . \
                 FILTER NOT EXISTS { ?parent <urn:ledger:ns#ofDecision> ?d } }",
        message: |row| {
            (
                term(row, "d"),
                format!(
                    "parent {} matches no filed version of this decision — the revision chain is broken",
                    term(row, "parent")
                ),
            )
        },
    },
    Shape {
        // The same class as a dangling parent: `merged_from` is the other
        // edge of the revision DAG, and a reconciliation naming a tip
        // nobody filed closed nothing.
        class: GraphClass::G002,
        select: "SELECT ?d ?closed WHERE { \
                 ?v <urn:ledger:ns#ofDecision> ?d . \
                 ?v <urn:ledger:ns#mergedFrom> ?closed . \
                 FILTER NOT EXISTS { ?closed <urn:ledger:ns#ofDecision> ?d } }",
        message: |row| {
            (
                term(row, "d"),
                format!(
                    "merged_from {} matches no filed version of this decision — the reconciliation closed nothing",
                    term(row, "closed")
                ),
            )
        },
    },
    Shape {
        class: GraphClass::G003,
        select: "SELECT DISTINCT ?d WHERE { \
                 ?v <urn:ledger:ns#ofDecision> ?d . \
                 FILTER NOT EXISTS { ?d a <urn:ledger:ns#Decision> } }",
        message: |row| {
            (
                term(row, "d"),
                "no change-set introduces this decision's identity — versions float on nothing"
                    .to_string(),
            )
        },
    },
    Shape {
        // A tip is a version no other version of the decision claims, by
        // `parent` or by `merged_from` — a reconciliation closes the tip it
        // names, which is exactly how an arbitration heals this shape.
        class: GraphClass::G004,
        select: "SELECT ?d ?a ?b WHERE { \
                 ?a a <urn:ledger:ns#DecisionVersion> . \
                 ?a <urn:ledger:ns#ofDecision> ?d . \
                 ?b a <urn:ledger:ns#DecisionVersion> . \
                 ?b <urn:ledger:ns#ofDecision> ?d . \
                 FILTER(STR(?a) < STR(?b)) \
                 FILTER NOT EXISTS { \
                   ?ca a <urn:ledger:ns#DecisionVersion> . \
                   ?ca <urn:ledger:ns#ofDecision> ?d . \
                   ?ca <http://www.w3.org/ns/prov#wasRevisionOf> ?a } \
                 FILTER NOT EXISTS { \
                   ?ma a <urn:ledger:ns#DecisionVersion> . \
                   ?ma <urn:ledger:ns#ofDecision> ?d . \
                   ?ma <urn:ledger:ns#mergedFrom> ?a } \
                 FILTER NOT EXISTS { \
                   ?cb a <urn:ledger:ns#DecisionVersion> . \
                   ?cb <urn:ledger:ns#ofDecision> ?d . \
                   ?cb <http://www.w3.org/ns/prov#wasRevisionOf> ?b } \
                 FILTER NOT EXISTS { \
                   ?mb a <urn:ledger:ns#DecisionVersion> . \
                   ?mb <urn:ledger:ns#ofDecision> ?d . \
                   ?mb <urn:ledger:ns#mergedFrom> ?b } }",
        message: |row| {
            (
                term(row, "d"),
                format!(
                    "the version chain forks: {} and {} are both tips — two writers diverged, and only `ledger merge --resolve` may settle which content stands",
                    short_hash(&term(row, "a")),
                    short_hash(&term(row, "b"))
                ),
            )
        },
    },
    Shape {
        // Two live claimants superseding one decision: L1's write-time
        // refusal met across branches, where it cannot refuse retroactively.
        // Only tips claim — a withdrawn edge (a new version without it) is
        // history, which is exactly how an arbitration settles this shape.
        class: GraphClass::G005,
        select: "SELECT ?d ?a ?b WHERE { \
                 ?va <urn:ledger:ns#supersedes> ?d . \
                 ?va <urn:ledger:ns#ofDecision> ?a . \
                 ?vb <urn:ledger:ns#supersedes> ?d . \
                 ?vb <urn:ledger:ns#ofDecision> ?b . \
                 FILTER(STR(?a) < STR(?b)) \
                 FILTER NOT EXISTS { \
                   ?ca <urn:ledger:ns#ofDecision> ?a . \
                   ?ca <http://www.w3.org/ns/prov#wasRevisionOf> ?va } \
                 FILTER NOT EXISTS { \
                   ?maa <urn:ledger:ns#ofDecision> ?a . \
                   ?maa <urn:ledger:ns#mergedFrom> ?va } \
                 FILTER NOT EXISTS { \
                   ?cb <urn:ledger:ns#ofDecision> ?b . \
                   ?cb <http://www.w3.org/ns/prov#wasRevisionOf> ?vb } \
                 FILTER NOT EXISTS { \
                   ?mab <urn:ledger:ns#ofDecision> ?b . \
                   ?mab <urn:ledger:ns#mergedFrom> ?vb } }",
        message: |row| {
            (
                term(row, "d"),
                format!(
                    "superseded by both {} and {} — the write-time fork refusal is now an arbitration; `ledger merge --resolve` settles which claim stands",
                    term(row, "a"),
                    term(row, "b")
                ),
            )
        },
    },
];

/// Display form of a hash term: the first 12 hex characters.
fn short_hash(term: &str) -> String {
    let hex = term.strip_prefix("sha256:").unwrap_or(term);
    hex.get(..12).unwrap_or(hex).to_string()
}

/// Run every shape over the store's emitted graph. An engine fault surfaces
/// as a finding on the shape itself rather than a panic or a silent pass —
/// a shape that cannot run must never read as a passing one.
pub fn graph_findings(store: &Store) -> Vec<GraphFinding> {
    let ttl = emit(store);
    let mut out = Vec::new();
    for shape in SHAPES {
        match product_core::pf::sparql_rules::select(&ttl, shape.select) {
            Ok(rows) => {
                for row in rows {
                    let (subject, message) = (shape.message)(&row);
                    out.push(GraphFinding { class: shape.class, subject, message });
                }
            }
            Err(e) => out.push(GraphFinding {
                class: shape.class,
                subject: shape.class.code().to_string(),
                message: format!("graph shape could not run: {e}"),
            }),
        }
    }
    out.sort_by(|a, b| (a.class, &a.subject).cmp(&(b.class, &b.subject)));
    out.dedup();
    out
}
