//! The L2 graph — a materialised, disposable view of the log.
//!
//! The log is the source of truth; this module derives from it and never
//! feeds back. Three pieces: [`turtle`] emits the RDF view (byte-
//! deterministic, so `reindex` after deletion reproduces the index
//! byte-identically — the PRD §5 correctness test), [`index`] writes it
//! under `.decisions/index/`, and [`shapes`] runs the SPARQL shape checks
//! for cross-entry invariants the per-file gate cannot name. The SPARQL
//! engine is `product-core`'s (OD-2): oxigraph stays that crate's
//! dependency, never this one's.
//!
//! Graph findings are a **distinct stage** of `verify`, outside the file
//! gate's closed ten classes: `G001`–`G003`, reported by the same command
//! with unchanged exit semantics.

pub mod index;
pub mod shapes;
pub mod turtle;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use std::fmt;

use serde::Serialize;

/// The closed set of graph-stage finding classes. Distinct from
/// [`crate::finding::VerifyClass`] on purpose: the file gate's ten are the
/// format's import surface, and these are the reference implementation's
/// graph stage — an outside implementation of the *format* need not run
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum GraphClass {
    /// A `supersedes` edge whose target decision is not filed anywhere.
    G001,
    /// A version's `parent` hash matching no filed version of its decision.
    G002,
    /// A version naming a decision no change-set ever introduced.
    G003,
}

/// Every graph class, in report order.
pub const ALL_GRAPH_CLASSES: &[GraphClass] = &[GraphClass::G001, GraphClass::G002, GraphClass::G003];

impl GraphClass {
    pub fn code(self) -> &'static str {
        match self {
            Self::G001 => "G001",
            Self::G002 => "G002",
            Self::G003 => "G003",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::G001 => "supersession target is not a filed decision",
            Self::G002 => "parent hash matches no filed version of the decision",
            Self::G003 => "version names a decision no change-set introduced",
        }
    }
}

impl fmt::Display for GraphClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// One graph-stage reason, against one subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphFinding {
    pub class: GraphClass,
    pub subject: String,
    pub message: String,
}

impl fmt::Display for GraphFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.class.code(), self.subject, self.message)
    }
}
