//! The closed set of reasons `ledger verify` fails.
//!
//! This enum *is* the file gate's contract. The gate fails for a schema
//! fault or one of ten semantic classes, and for nothing else — an eleventh
//! reason is a change to the format specification, not an implementation
//! detail (`L010` arrived exactly that way, as the spec v1.1 amendment).
//! Adding a variant here without a row in `docs/ledger-format-v1.md` is
//! caught by the `every_class_is_specified` test.
//!
//! "Allocated, awaiting acceptance" is deliberately absent. The gate polices
//! violations, not pendency: a decision that is enumerated and allocated but
//! not yet signed is the normal state of work in progress, and reporting it
//! as a failure would train people to ignore the gate.

use std::fmt;

use serde::Serialize;

/// Which gate a class blocks (§6). Readiness blocks produce; completeness
/// blocks release, and includes everything readiness covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Readiness,
    Completeness,
}

/// A reason the gate fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum VerifyClass {
    /// The file does not parse against the format it declares. Not one of
    /// the nine; the parse gate that must pass before they can run.
    Schema,
    /// A decision with no allocation.
    L001,
    /// An escape without exposure, acceptor, and review date.
    L002,
    /// An expired acceptance. A hard failure, never a warning (§4.4).
    L003,
    /// A tolerance override at or below its set floor (§4.2.1).
    L004,
    /// A member stranded below a raised floor without re-acceptance.
    L005,
    /// An acceptor resolving to a model or CI identity (§9.3).
    L006,
    /// A stored hash that does not match recomputed content.
    L007,
    /// An acceptance signing a hash that matches no stored version.
    L008,
    /// An acceptance whose actor is not the author of the commit that
    /// introduced it.
    L009,
    /// A judgment whose named actor resolves to a model or CI identity
    /// (spec v1.1 — the judgment-actor gap L0 left open).
    L010,
}

/// Every class, in report order. The list the format document mirrors.
pub const ALL_CLASSES: &[VerifyClass] = &[
    VerifyClass::Schema,
    VerifyClass::L001,
    VerifyClass::L002,
    VerifyClass::L003,
    VerifyClass::L004,
    VerifyClass::L005,
    VerifyClass::L006,
    VerifyClass::L007,
    VerifyClass::L008,
    VerifyClass::L009,
    VerifyClass::L010,
];

impl VerifyClass {
    /// The code as it appears in output and in the format document.
    pub fn code(self) -> &'static str {
        match self {
            Self::Schema => "SCHEMA",
            Self::L001 => "L001",
            Self::L002 => "L002",
            Self::L003 => "L003",
            Self::L004 => "L004",
            Self::L005 => "L005",
            Self::L006 => "L006",
            Self::L007 => "L007",
            Self::L008 => "L008",
            Self::L009 => "L009",
            Self::L010 => "L010",
        }
    }

    /// The one-line name of the class.
    pub fn title(self) -> &'static str {
        match self {
            Self::Schema => "file does not parse against its declared format",
            Self::L001 => "decision with no allocation",
            Self::L002 => "escape without exposure, acceptor, and review date",
            Self::L003 => "expired acceptance",
            Self::L004 => "tolerance override at or below its set floor",
            Self::L005 => "member stranded below a raised set floor",
            Self::L006 => "acceptor resolves to a model or CI identity",
            Self::L007 => "stored hash does not match recomputed content",
            Self::L008 => "acceptance signs a hash matching no stored version",
            Self::L009 => "acceptance actor is not the author of its introducing commit",
            Self::L010 => "judgment actor resolves to a model or CI identity",
        }
    }

    /// Whether this class runs under the given gate.
    ///
    /// Readiness blocks produce: nothing unallocated, nothing tampered,
    /// nobody unaccountable. The two classes it leaves out — an unpriced
    /// escape and a stale acceptance — are dispositions that must hold at
    /// release, not preconditions for starting work.
    pub fn in_gate(self, gate: Gate) -> bool {
        match gate {
            Gate::Completeness => true,
            Gate::Readiness => !matches!(self, Self::L002 | Self::L003),
        }
    }
}

impl fmt::Display for VerifyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// One reason, against one subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub class: VerifyClass,
    /// What the finding is about: a decision id, an acceptance id, or a file.
    pub subject: String,
    pub message: String,
}

/// A classified reason with no subject yet — what a validator deep in the
/// type stack knows before the caller supplies what it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub class: VerifyClass,
    pub message: String,
}

impl Reason {
    pub fn new(class: VerifyClass, message: impl Into<String>) -> Self {
        Self { class, message: message.into() }
    }

    /// A parse-gate reason.
    pub fn schema(message: impl Into<String>) -> Self {
        Self::new(VerifyClass::Schema, message)
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.class.code(), self.message)
    }
}

impl Finding {
    pub fn new(class: VerifyClass, subject: &str, message: impl Into<String>) -> Self {
        Self { class, subject: subject.to_string(), message: message.into() }
    }

    /// Attach a subject to a classified reason.
    pub fn about(subject: &str, reason: Reason) -> Self {
        Self { class: reason.class, subject: subject.to_string(), message: reason.message }
    }

    /// A parse-gate fault, before any semantic class can be judged.
    pub fn schema(subject: &str, message: impl Into<String>) -> Self {
        Self::new(VerifyClass::Schema, subject, message)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.class.code(), self.subject, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_exactly_ten_semantic_classes_plus_the_parse_gate() {
        // The gate fails for these reasons and no others. An eleventh class
        // is a format-specification change — L010 itself shipped as the spec
        // v1.1 amendment, moving this count from nine to ten.
        assert_eq!(ALL_CLASSES.len(), 11);
        let semantic = ALL_CLASSES.iter().filter(|c| **c != VerifyClass::Schema).count();
        assert_eq!(semantic, 10);
    }

    #[test]
    fn every_class_has_a_distinct_code_and_a_title() {
        let mut codes: Vec<&str> = ALL_CLASSES.iter().map(|c| c.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), ALL_CLASSES.len());
        assert!(ALL_CLASSES.iter().all(|c| !c.title().is_empty()));
    }

    #[test]
    fn readiness_leaves_out_the_two_release_dispositions() {
        assert!(!VerifyClass::L002.in_gate(Gate::Readiness));
        assert!(!VerifyClass::L003.in_gate(Gate::Readiness));
        assert!(VerifyClass::L001.in_gate(Gate::Readiness));
        assert!(ALL_CLASSES.iter().all(|c| c.in_gate(Gate::Completeness)));
    }

    #[test]
    fn a_finding_renders_class_subject_and_message() {
        let f = Finding::new(VerifyClass::L001, "dec:x/Y", "no allocation");
        assert_eq!(f.to_string(), "[L001] dec:x/Y: no allocation");
    }
}
