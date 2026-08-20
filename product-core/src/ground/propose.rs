//! Turning a triple into a proposal — the Reading that carries it.
//!
//! Every proposed assertion gets the §4.1 tuple from its §5.2 row, not from
//! the caller: the row fixes **both** grades, the relevance basis, the
//! provenance, the method. No mapping rule can quietly claim a confidence its
//! row does not justify, and none can claim a relevance nobody computed. The
//! instrument comes from the facts, which recorded what actually read the
//! corpus.
//!
//! What the caller *does* supply is the [`Derivation`] — the rule it used, the
//! operations it stood on, the source kinds it saw. That is per-assertion by
//! nature: a row's rule fires over many declarations, and which member kind
//! each one carried is the fact G0 found missing.

use super::derivation::Derivation;
use super::mint::assertion_id;
use super::rows::Row;
use super::triple::{ProposedAssertion, Triple};

/// Builds proposals for one extraction run.
pub struct Proposer {
    /// What read the corpus, verbatim from the facts.
    instrument: String,
}

impl Proposer {
    pub fn new(instrument: impl Into<String>) -> Self {
        Proposer { instrument: instrument.into() }
    }

    /// Propose `triple` as read through `row`, citing `evidence`, recording
    /// `derivation`.
    pub fn propose(
        &self,
        row: &'static Row,
        triple: Triple,
        evidence: Vec<String>,
        derivation: Derivation,
    ) -> ProposedAssertion {
        ProposedAssertion {
            id: assertion_id(&triple),
            triple,
            row: row.id,
            confidence: row.confidence,
            relevance: row.relevance,
            relevance_basis: row.relevance_basis,
            provenance: row.provenance,
            assurance: self.assurance(row),
            evidence,
            derivation,
        }
    }

    /// The §4.1 assurance field: both marks, the method, the instrument, the
    /// row — in one line a reviewer can read and a query can filter on. The
    /// relevance basis rides here too, so `unknown` never reads as a verdict.
    fn assurance(&self, row: &Row) -> String {
        format!(
            "derivation {} · relevance {} ({}) — {}; row {}; instrument {}",
            row.confidence.as_str(),
            row.relevance.as_str(),
            row.relevance_basis.as_str(),
            row.derivation,
            row.id,
            self.instrument
        )
    }
}

/// Cite a position in the corpus, repo-relative, one-based for a reader.
pub fn cite(file: &str, line: u64) -> String {
    format!("{file}:{}", line + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::axes::{DerivationConfidence, DomainRelevance, RelevanceBasis};
    use crate::ground::rows::{row, Provenance};
    use crate::ground::triple::Term;

    fn a_triple() -> Triple {
        Triple::new("tag:x/A", "tag:x/p", Term::iri("tag:x/B"))
    }

    fn a_derivation() -> Derivation {
        Derivation::by("declared-type", &["documentSymbol"])
    }

    #[test]
    fn a_proposal_takes_its_grade_from_its_row_not_its_caller() {
        let p = Proposer::new("test-instrument 1.0");
        let declared = p.propose(row("classes").expect("row"), a_triple(), vec![], a_derivation());
        assert_eq!(declared.confidence, DerivationConfidence::High);
        assert_eq!(declared.provenance, Provenance::Observed);

        let heuristic =
            p.propose(row("synonyms").expect("row"), a_triple(), vec![], a_derivation());
        assert_eq!(heuristic.confidence, DerivationConfidence::Low);
        assert_eq!(heuristic.provenance, Provenance::Inferred);
    }

    #[test]
    fn the_assurance_field_names_both_marks_method_row_instrument() {
        let p = Proposer::new("roslyn 5.11.0");
        let a = p.propose(row("subclass").expect("row"), a_triple(), vec![], a_derivation());
        assert!(a.assurance.starts_with("derivation high · "), "{}", a.assurance);
        assert!(a.assurance.contains("relevance unknown (not-computed)"), "{}", a.assurance);
        assert!(a.assurance.contains("row subclass"), "{}", a.assurance);
        assert!(a.assurance.contains("instrument roslyn 5.11.0"), "{}", a.assurance);
    }

    /// The fixture the whole change turns on: no assertion may carry one mark
    /// without the other, whatever row proposed it.
    #[test]
    fn every_proposal_carries_both_marks() {
        let p = Proposer::new("i");
        for r in crate::ground::rows::ROWS.iter() {
            let a = p.propose(r, a_triple(), vec![], a_derivation());
            assert_eq!(a.confidence, r.confidence, "{}", r.id);
            assert_eq!(a.relevance, DomainRelevance::Unknown, "{}", r.id);
            assert_ne!(a.relevance_basis, RelevanceBasis::Computed, "{}", r.id);
        }
    }

    /// Unknown relevance is representable, is what every row reports, and is
    /// not the same value as high — the mark says *not computed* rather than
    /// standing in for a verdict.
    #[test]
    fn unknown_relevance_is_distinct_from_high() {
        assert_ne!(DomainRelevance::Unknown, DomainRelevance::High);
        let p = Proposer::new("i");
        let a = p.propose(row("properties").expect("row"), a_triple(), vec![], a_derivation());
        assert_eq!(a.relevance.as_str(), "unknown");
        assert_eq!(a.relevance_basis.as_str(), "not-computed");
    }

    /// Two rows proposing the same triple mint the same id, because the id
    /// is the triple's. The file name follows, so the second proposal
    /// collapses onto the first rather than duplicating it.
    #[test]
    fn identity_follows_the_triple_not_the_row() {
        let p = Proposer::new("i");
        let one = p.propose(row("classes").expect("row"), a_triple(), vec![], a_derivation());
        let two = p.propose(row("modules").expect("row"), a_triple(), vec![], a_derivation());
        assert_eq!(one.id, two.id);
        assert_eq!(one.file_name(), two.file_name());
    }

    #[test]
    fn citations_are_one_based_for_a_reader() {
        assert_eq!(cite("src/Entities/Settings.cs", 9), "src/Entities/Settings.cs:10");
    }
}
