//! The proposed assertion — one triple, with the Reading that took it.
//!
//! An assertion is stored reified, as the registry template's exemplar
//! shows: a node typed `reg:Assertion , reg:Reading`, naming its subject,
//! predicate, object, and carrying the §4.1 tuple. One assertion per file
//! is the registry's file rule, so this struct is also the unit of review.

use serde::Serialize;

use super::rows::{Grade, Provenance};

/// An RDF term in object position.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Term {
    /// A full IRI, written between angle brackets.
    Iri(String),
    /// A literal, optionally typed.
    Literal { value: String, datatype: Option<String> },
}

impl Term {
    pub fn iri(s: impl Into<String>) -> Self {
        Term::Iri(s.into())
    }

    pub fn plain(s: impl Into<String>) -> Self {
        Term::Literal { value: s.into(), datatype: None }
    }

    /// The term in Turtle, IRIs bracketed, literals escaped.
    pub fn to_turtle(&self) -> String {
        match self {
            Term::Iri(iri) => format!("<{iri}>"),
            Term::Literal { value, datatype } => {
                let body = format!("\"{}\"", escape_literal(value));
                match datatype {
                    Some(dt) => format!("{body}^^<{dt}>"),
                    None => body,
                }
            }
        }
    }

    /// The term as a reviewer reads it: an IRI's local name, or the literal.
    pub fn short(&self) -> String {
        match self {
            Term::Iri(iri) => local_part(iri).to_string(),
            Term::Literal { value, .. } => value.clone(),
        }
    }
}

/// Escape a literal for a Turtle quoted string.
pub fn escape_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// The part of an IRI after its last delimiter.
pub fn local_part(iri: &str) -> &str {
    iri.rsplit(['/', '#', ':']).next().unwrap_or(iri)
}

/// One triple, subject and predicate always IRIs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: Term,
}

impl Triple {
    pub fn new(subject: impl Into<String>, predicate: impl Into<String>, object: Term) -> Self {
        Triple { subject: subject.into(), predicate: predicate.into(), object }
    }

    /// The plain-triple form, for loading into a store where entailment runs.
    pub fn to_turtle(&self) -> String {
        format!("<{}> <{}> {} .", self.subject, self.predicate, self.object.to_turtle())
    }

    /// The `reg:value` text — the triple as a reviewer reads it aloud.
    pub fn to_value(&self) -> String {
        format!("{} {} {}", local_part(&self.subject), local_part(&self.predicate), self.object.short())
    }
}

/// A triple proposed for ratification, with everything a reviewer needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProposedAssertion {
    /// Content-derived identity: the same triple mints the same id on every
    /// run, so re-running is a diff rather than a renumbering.
    pub id: String,
    pub triple: Triple,
    /// Which §5.2 row proposed it.
    pub row: &'static str,
    pub grade: Grade,
    pub provenance: Provenance,
    /// Instrument or method, per §4.1.
    pub assurance: String,
    /// Where the evidence sits in the corpus, for the reviewer to check.
    pub evidence: Vec<String>,
}

impl ProposedAssertion {
    /// The file this assertion is stored in.
    pub fn file_name(&self) -> String {
        format!("assertion-{}.ttl", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_are_escaped_for_turtle() {
        let t = Term::plain("a \"quoted\" line\nand a \\ backslash");
        assert_eq!(t.to_turtle(), "\"a \\\"quoted\\\" line\\nand a \\\\ backslash\"");
    }

    #[test]
    fn local_part_reads_every_delimiter() {
        assert_eq!(local_part("tag:x,2026:ground/Order"), "Order");
        assert_eq!(local_part("http://www.w3.org/2000/01/rdf-schema#subClassOf"), "subClassOf");
        assert_eq!(local_part("urn:a:b"), "b");
    }

    #[test]
    fn the_value_field_reads_as_a_sentence() {
        let t = Triple::new(
            "tag:x,2026:ground/Order",
            "http://www.w3.org/2000/01/rdf-schema#subClassOf",
            Term::iri("tag:x,2026:ground/Aggregate"),
        );
        assert_eq!(t.to_value(), "Order subClassOf Aggregate");
        assert_eq!(
            t.to_turtle(),
            "<tag:x,2026:ground/Order> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <tag:x,2026:ground/Aggregate> ."
        );
    }
}
