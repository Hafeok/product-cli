//! The projection join — ratified assertion beside its run evidence.
//!
//! Authority keeps the two layers apart: `graphs/` holds ratified assertion,
//! `runs/` holds what the instrument saw. **Consumers read the projection**,
//! where both are loaded into one store, so derivation is queryable there
//! without either layer borrowing the other's status. That is the layer split
//! working as designed rather than a workaround for it.
//!
//! The entailment step is the reason this module has fixtures rather than a
//! comment. `reg:assuranceGrade owl:equivalentProperty reg:derivationConfidence`
//! backfills axis 1 onto every assertion ratified before the split, without a
//! byte changing — but it also means the projection *manufactures*
//! `reg:derivationConfidence` on those older assertions. A shape targeting
//! predicate presence would therefore pass in the authority repo, where
//! validation runs pre-entailment, and fail the moment anyone validated the
//! projection. The shapes target a **class** the writer stamps explicitly, and
//! [`validate_projection`] is what proves the two cannot drift apart again.
//!
//! Oxigraph's serialiser is used here, and only here: a projection is
//! transport, never authority, so order-stability is not its job.

use oxigraph::io::RdfFormat;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::error::{ProductError, Result};
use crate::registry::{evaluate, Finding};

/// The three layers a projection build loads.
#[derive(Debug, Default, Clone)]
pub struct ProjectionSources {
    /// Ratified assertion files, one assertion each.
    pub graphs: Vec<String>,
    /// Run evidence files, keyed by assertion id.
    pub runs: Vec<String>,
    /// The registry's own vocabulary, carrying the equivalence axioms.
    pub vocabulary: String,
}

/// The equivalence entailment the projection applies.
///
/// One rule, stated as a CONSTRUCT rather than pulled from a reasoner, because
/// this is the only OWL axiom the two generations of assertion need to agree
/// on and a whole materialiser for it would hide what is happening.
const EQUIVALENCE: &str = "
CONSTRUCT { ?s ?new ?o }
WHERE {
    ?old <http://www.w3.org/2002/07/owl#equivalentProperty> ?new .
    ?s ?old ?o .
}";

/// Load every layer into one store, apply the equivalence entailment, and
/// serialise the result.
pub fn project(sources: &ProjectionSources) -> Result<String> {
    let store = Store::new().map_err(|e| ProductError::ConfigError(e.to_string()))?;
    for text in sources.graphs.iter().chain(&sources.runs).chain([&sources.vocabulary]) {
        if text.trim().is_empty() {
            continue;
        }
        store
            .load_from_reader(RdfFormat::Turtle, text.as_bytes())
            .map_err(|e| ProductError::ConfigError(format!("projection load: {e}")))?;
    }
    entail_equivalences(&store)?;
    let mut out = Vec::new();
    store
        .dump_graph_to_writer(oxigraph::model::GraphNameRef::DefaultGraph, RdfFormat::NTriples, &mut out)
        .map_err(|e| ProductError::ConfigError(format!("projection dump: {e}")))?;
    String::from_utf8(out)
        .map_err(|e| ProductError::ConfigError(format!("projection is not utf-8: {e}")))
}

/// Apply the equivalence rule once. Drain before insert: a CONSTRUCT's triple
/// iterator borrows the store.
fn entail_equivalences(store: &Store) -> Result<()> {
    let prepared = SparqlEvaluator::new()
        .parse_query(EQUIVALENCE)
        .map_err(|e| ProductError::ConfigError(format!("equivalence rule: {e}")))?;
    let results = prepared
        .on_store(store)
        .execute()
        .map_err(|e| ProductError::ConfigError(format!("equivalence rule: {e}")))?;
    let QueryResults::Graph(triples) = results else {
        return Err(ProductError::ConfigError("equivalence rule is not a CONSTRUCT".into()));
    };
    let drained: Vec<_> = triples
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ProductError::ConfigError(format!("equivalence rule: {e}")))?;
    for triple in drained {
        store
            .insert(triple.in_graph(oxigraph::model::GraphName::DefaultGraph).as_ref())
            .map_err(|e| ProductError::ConfigError(format!("equivalence insert: {e}")))?;
    }
    Ok(())
}

/// Validate the **projection** against the instance's shapes.
///
/// The repo-level check validates `graphs/` before entailment. This validates
/// what a consumer actually reads: both layers joined, the equivalences
/// materialised. A shape that passes one and fails the other is the defect
/// this function exists to catch.
pub fn validate_projection(
    sources: &ProjectionSources,
    shapes_ttl: &str,
) -> Result<(Vec<Finding>, usize)> {
    let projected = project(sources)?;
    if projected.trim().is_empty() || shapes_ttl.trim().is_empty() {
        return Ok((Vec::new(), 0));
    }
    evaluate(&projected, shapes_ttl)
}

/// Run one SELECT over the projection — how a reviewer recovers a class of
/// error after the fact.
pub fn query_projection(sources: &ProjectionSources, select: &str) -> Result<Vec<Vec<String>>> {
    let projected = project(sources)?;
    let store = Store::new().map_err(|e| ProductError::ConfigError(e.to_string()))?;
    store
        .load_from_reader(RdfFormat::NTriples, projected.as_bytes())
        .map_err(|e| ProductError::ConfigError(format!("projection reload: {e}")))?;
    let prepared = SparqlEvaluator::new()
        .parse_query(select)
        .map_err(|e| ProductError::ConfigError(format!("query: {e}")))?;
    let results = prepared
        .on_store(&store)
        .execute()
        .map_err(|e| ProductError::ConfigError(format!("query: {e}")))?;
    let QueryResults::Solutions(solutions) = results else {
        return Err(ProductError::ConfigError("query is not a SELECT".into()));
    };
    let mut rows = Vec::new();
    for solution in solutions {
        let solution = solution.map_err(|e| ProductError::ConfigError(format!("query: {e}")))?;
        rows.push(solution.iter().map(|(_, term)| term.to_string()).collect());
    }
    rows.sort();
    Ok(rows)
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
