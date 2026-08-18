//! Turning a triple into a proposal — the Reading that carries it.
//!
//! Every proposed assertion gets the §4.1 tuple from its §5.2 row, not
//! from the caller: the row fixes the grade, the provenance and the method,
//! so no mapping rule can quietly claim a better assurance than its row
//! justifies. The instrument comes from the facts, which recorded what
//! actually read the corpus.

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

    /// Propose `triple` as read through `row`, citing `evidence`.
    pub fn propose(&self, row: &'static Row, triple: Triple, evidence: Vec<String>) -> ProposedAssertion {
        ProposedAssertion {
            id: assertion_id(&triple),
            triple,
            row: row.id,
            grade: row.grade,
            provenance: row.provenance,
            assurance: self.assurance(row),
            evidence,
        }
    }

    /// The §4.1 assurance field: grade, method, instrument, row — in one
    /// line a reviewer can read and a query can filter on.
    fn assurance(&self, row: &Row) -> String {
        format!(
            "{} — {}; row {}; instrument {}",
            row.grade.as_str(),
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
    use crate::ground::rows::{row, Grade, Provenance};
    use crate::ground::triple::Term;

    fn a_triple() -> Triple {
        Triple::new("tag:x/A", "tag:x/p", Term::iri("tag:x/B"))
    }

    #[test]
    fn a_proposal_takes_its_grade_from_its_row_not_its_caller() {
        let p = Proposer::new("test-instrument 1.0");
        let declared = p.propose(row("classes").expect("row"), a_triple(), vec![]);
        assert_eq!(declared.grade, Grade::High);
        assert_eq!(declared.provenance, Provenance::Observed);

        let heuristic = p.propose(row("synonyms").expect("row"), a_triple(), vec![]);
        assert_eq!(heuristic.grade, Grade::Low);
        assert_eq!(heuristic.provenance, Provenance::Inferred);
    }

    #[test]
    fn the_assurance_field_names_grade_method_row_instrument() {
        let p = Proposer::new("roslyn 5.11.0");
        let a = p.propose(row("subclass").expect("row"), a_triple(), vec![]);
        assert!(a.assurance.starts_with("high — "), "{}", a.assurance);
        assert!(a.assurance.contains("row subclass"), "{}", a.assurance);
        assert!(a.assurance.contains("instrument roslyn 5.11.0"), "{}", a.assurance);
    }

    /// Two rows proposing the same triple mint the same id, because the id
    /// is the triple's. The file name follows, so the second proposal
    /// collapses onto the first rather than duplicating it.
    #[test]
    fn identity_follows_the_triple_not_the_row() {
        let p = Proposer::new("i");
        let one = p.propose(row("classes").expect("row"), a_triple(), vec![]);
        let two = p.propose(row("modules").expect("row"), a_triple(), vec![]);
        assert_eq!(one.id, two.id);
        assert_eq!(one.file_name(), two.file_name());
    }

    #[test]
    fn citations_are_one_based_for_a_reader() {
        assert_eq!(cite("src/Entities/Settings.cs", 9), "src/Entities/Settings.cs:10");
    }
}
