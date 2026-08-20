//! Identity for extracted things — IRIs for subjects, ids for assertions.
//!
//! **An assertion's identity is derived from the triple it asserts, never
//! from its position in a run.** That is the whole of the Reading-identity
//! question, and it is settled here rather than left to the caller:
//!
//! - Re-running at the same corpus ref mints the same ids, so the second
//!   run's diff against the ratified graph is *empty* — the correct answer
//!   for "nothing changed".
//! - Re-running at a later ref mints the same id for an unchanged triple, a
//!   fresh id for a new one, and leaves a ratified file with no counterpart
//!   when a symbol disappears. §5.5's candidates-added / symbols-disappeared
//!   diff therefore falls out of the file set; no run registry is needed.
//! - Supersession can name an assertion, because the name outlives the run.
//!
//! Sequential numbering would fail all three: one insertion renumbers the
//! tail, and at extraction volume the second run's diff is entirely churn
//! with nothing in it a reviewer could read.

use sha2::{Digest, Sha256};

use super::triple::Triple;

/// How much of the digest the local name carries.
const ID_HEX: usize = 16;

/// The content-derived id of an assertion.
///
/// Hashes **only** subject, predicate and object. Not the run, not the
/// as-of, not the assurance: those describe the reading, and letting them
/// into the identity is what would make a re-read a different assertion.
pub fn assertion_id(triple: &Triple) -> String {
    let mut hasher = Sha256::new();
    hasher.update(triple.subject.as_bytes());
    hasher.update([0x1f]);
    hasher.update(triple.predicate.as_bytes());
    hasher.update([0x1f]);
    hasher.update(triple.object.to_turtle().as_bytes());
    let digest = hasher.finalize();
    digest.iter().take(ID_HEX / 2).map(|b| format!("{b:02x}")).collect()
}

/// A local name safe to write after a prefix in Turtle.
///
/// Declared identifiers are alphanumeric, but namespaces carry dots and
/// generics carry brackets; both reach the extractor, so both are folded to
/// a single safe alphabet rather than trusted.
pub fn local_name(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut last_dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed
    }
}

/// The IRI of a declared type.
pub fn class_iri(base: &str, name: &str) -> String {
    format!("{base}{}", local_name(name))
}

/// The IRI of a declared member, qualified by its owner so two classes
/// carrying a same-named property stay two properties.
pub fn property_iri(base: &str, owner: &str, property: &str) -> String {
    format!("{base}prop-{}-{}", local_name(owner), local_name(property))
}

/// The IRI of a module axis.
pub fn module_iri(base: &str, module: &str) -> String {
    format!("{base}axis-{}", local_name(module))
}

/// The IRI of the extraction run, stable for a given corpus at a given ref
/// so re-running attributes to the same act rather than inventing one.
pub fn run_iri(base: &str, corpus: &str, git_ref: &str) -> String {
    format!("{base}extraction-{}-{}", local_name(corpus), short_ref(git_ref))
}

/// The IRI of the corpus as read at one ref — the `wasDerivedFrom` target.
pub fn corpus_iri(base: &str, corpus: &str, git_ref: &str) -> String {
    format!("{base}source-{}-at-{}", local_name(corpus), short_ref(git_ref))
}

/// The IRI of one operation's probe verdict on one run.
///
/// Qualified by the run, because the verdict is a reading taken at an
/// instant rather than a standing fact about the server: two runs over the
/// same corpus at the same ref may disagree, and they must be able to say so
/// side by side.
pub fn probe_iri(base: &str, corpus: &str, git_ref: &str, operation: &str) -> String {
    format!("{base}probe-{}-{}-{}", local_name(corpus), short_ref(git_ref), local_name(operation))
}

/// A ref abbreviated for a local name, full-length refs kept in a literal.
pub fn short_ref(git_ref: &str) -> String {
    let name = local_name(git_ref);
    name.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::triple::Term;

    fn t(s: &str, p: &str, o: &str) -> Triple {
        Triple::new(s, p, Term::iri(o))
    }

    #[test]
    fn the_same_triple_mints_the_same_id() {
        let a = t("tag:x/Order", "tag:x/hasPart", "tag:x/Line");
        let b = t("tag:x/Order", "tag:x/hasPart", "tag:x/Line");
        assert_eq!(assertion_id(&a), assertion_id(&b));
        assert_eq!(assertion_id(&a).len(), ID_HEX);
    }

    #[test]
    fn a_different_triple_mints_a_different_id() {
        let base = t("tag:x/Order", "tag:x/hasPart", "tag:x/Line");
        for other in [
            t("tag:x/Invoice", "tag:x/hasPart", "tag:x/Line"),
            t("tag:x/Order", "tag:x/hasMember", "tag:x/Line"),
            t("tag:x/Order", "tag:x/hasPart", "tag:x/Item"),
        ] {
            assert_ne!(assertion_id(&base), assertion_id(&other));
        }
    }

    /// The delimiter matters: without it, (`ab`, `c`, …) and (`a`, `bc`, …)
    /// would hash alike and two different assertions would share a file.
    #[test]
    fn field_boundaries_are_separated_in_the_digest() {
        let left = Triple::new("tag:x/ab", "tag:x/c", Term::plain("o"));
        let right = Triple::new("tag:x/a", "tag:x/bc", Term::plain("o"));
        assert_ne!(assertion_id(&left), assertion_id(&right));
    }

    /// An IRI object and a literal object with the same text are different
    /// assertions, and must not collide.
    #[test]
    fn an_iri_object_never_collides_with_a_literal_of_the_same_text() {
        let as_iri = Triple::new("tag:x/A", "tag:x/p", Term::iri("tag:x/B"));
        let as_literal = Triple::new("tag:x/A", "tag:x/p", Term::plain("tag:x/B"));
        assert_ne!(assertion_id(&as_iri), assertion_id(&as_literal));
    }

    /// **The fixture to watch.** g-dec-04 fixes identity as the triple and
    /// nothing else. If the two grading axes or the derivation record entered
    /// the hash, every ratified assertion would become a fresh proposal and
    /// the instance's history would break.
    ///
    /// This is verified rather than assumed: two assertions differing in
    /// confidence, relevance, basis, rule, operations, member kind, container
    /// kind and source path — differing in *everything the split added* —
    /// still mint one id, because they assert the same triple.
    #[test]
    fn the_two_axes_and_the_derivation_stay_out_of_the_identity() {
        use crate::ground::derivation::Derivation;
        use crate::ground::facts::{DeclKind, MemberKind};
        use crate::ground::propose::Proposer;
        use crate::ground::rows::row;

        let triple = t("tag:x/Order", "tag:x/hasPart", "tag:x/Line");
        let p = Proposer::new("roslyn 5.11.0");
        let declared = p.propose(
            row("classes").expect("row"),
            triple.clone(),
            vec![],
            Derivation::by("declared-type", &["documentSymbol"])
                .in_container(DeclKind::Class, "src/A.cs"),
        );
        let heuristic = p.propose(
            row("synonyms").expect("row"),
            triple.clone(),
            vec![],
            Derivation::by("identifier-word-split", &["documentSymbol", "hover"])
                .in_container(DeclKind::Record, "src/Z.cs")
                .of_member(MemberKind::Field),
        );

        assert_ne!(declared.confidence, heuristic.confidence);
        assert_ne!(declared.derivation, heuristic.derivation);
        assert_eq!(declared.id, heuristic.id, "identity is the triple, and only the triple");
        assert_eq!(declared.id, assertion_id(&triple));
        assert_eq!(declared.file_name(), heuristic.file_name());
    }

    #[test]
    fn local_names_survive_namespaces_and_generics() {
        assert_eq!(local_name("Acme.App.Backend.Sql"), "Acme-App-Backend-Sql");
        assert_eq!(local_name("IHandler<Command>"), "IHandler-Command");
        assert_eq!(local_name("project Acme.Sql (net10.0)"), "project-Acme-Sql-net10-0");
        assert_eq!(local_name("plain"), "plain");
        assert_eq!(local_name("!!!"), "unnamed");
    }

    #[test]
    fn property_iris_are_qualified_by_their_owner() {
        let base = "tag:x/";
        assert_ne!(property_iri(base, "Order", "Id"), property_iri(base, "Invoice", "Id"));
    }

    #[test]
    fn run_and_corpus_identity_are_stable_for_a_ref() {
        let base = "tag:x/";
        let long = "3b0a56b33b2282e45e53c73d3df2026ac506bd01";
        assert_eq!(run_iri(base, "corpus-backend", long), "tag:x/extraction-corpus-backend-3b0a56b33b22");
        assert_eq!(corpus_iri(base, "corpus-backend", long), "tag:x/source-corpus-backend-at-3b0a56b33b22");
    }
}
