//! What the extractor was able to read from a codebase, as plain data.
//!
//! This is the seam between the LSP half of extraction (which knows about
//! language servers) and the mapping half (which knows about ontology).
//! Everything here is language-neutral and dependency-free: the reader
//! fills it, the mapping rows consume it, and `product-core` therefore
//! needs no LSP client of its own.
//!
//! Nothing here is inferred. A fact is present because an operation
//! answered, or it is absent — and which operations answered is itself a
//! recorded fact ([`CorpusFacts::operations`]), because an absence the
//! instrument caused must never read as an absence the codebase has.

use serde::{Deserialize, Serialize};

/// What a declaration declares itself to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeclKind {
    Class,
    Interface,
    Struct,
    Enum,
    Record,
}

impl DeclKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeclKind::Class => "class",
            DeclKind::Interface => "interface",
            DeclKind::Struct => "struct",
            DeclKind::Enum => "enum",
            DeclKind::Record => "record",
        }
    }
}

/// What kind of member the server reported.
///
/// The declaration surface carries no *visibility*, so this is as close as
/// the extractor gets: a field is usually implementation and a property
/// usually structure, but neither is certain and the mapping row treats them
/// alike. Recorded so the ambiguity is reportable rather than absorbed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemberKind {
    #[default]
    Property,
    Field,
    EnumMember,
}

impl MemberKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberKind::Property => "property",
            MemberKind::Field => "field",
            MemberKind::EnumMember => "enum-member",
        }
    }
}

/// One declared member carrying a type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyDecl {
    pub name: String,
    /// The type as the server rendered it, nullability marker included.
    pub type_name: String,
    /// Property, field, or enum member — as reported, not as judged.
    #[serde(default)]
    pub kind: MemberKind,
}

impl PropertyDecl {
    /// The type with its nullability marker stripped — the name a range
    /// resolves against.
    pub(crate) fn bare_type(&self) -> &str {
        self.type_name.trim_end_matches('?').trim()
    }

    pub(crate) fn nullable(&self) -> bool {
        self.type_name.trim_end().ends_with('?')
    }
}

/// One type declaration as read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    pub name: String,
    pub kind: DeclKind,
    /// The containing module: the project or namespace the server reported.
    pub module: String,
    /// Repo-relative path, so a fact does not carry a machine's layout.
    pub file: String,
    pub start_line: u64,
    pub end_line: u64,
    /// The declaration's own doc summary, when `hover` returned one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default)]
    pub properties: Vec<PropertyDecl>,
}

/// A declared derivation: `sub` names `sup` in its base list.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HierarchyEdge {
    pub sub: String,
    pub sup: String,
}

/// One use site of a declared type, as `references` reported it.
///
/// A reference is a *position*, not yet a relation: turning it into an
/// edge between two types needs the containment join in `entail`, which is
/// the first CONSTRUCT round.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReferenceSite {
    /// The type being referred to.
    pub target: String,
    pub file: String,
    pub line: u64,
}

/// Whether one declaration-level operation answered on this corpus.
///
/// Carried into the facts so a mapping row can report *why* it found
/// nothing. The probe decides this; the mapping half only reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationStatus {
    pub operation: String,
    pub answered: bool,
    pub detail: String,
}

/// Everything one extraction run read from one codebase at one ref.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusFacts {
    /// The corpus's abstracted name, never its real repository name.
    pub corpus: String,
    /// The commit the read was pinned to.
    pub git_ref: String,
    /// What produced these facts, recorded for the assurance field.
    pub instrument: String,
    /// Which of the six operations answered.
    pub operations: Vec<OperationStatus>,
    #[serde(default)]
    pub declarations: Vec<Declaration>,
    #[serde(default)]
    pub hierarchy: Vec<HierarchyEdge>,
    #[serde(default)]
    pub references: Vec<ReferenceSite>,
    /// Files the reader opened but could not read a declaration from, kept
    /// so partial coverage is legible rather than silent.
    #[serde(default)]
    pub unread_files: Vec<String>,
}

impl CorpusFacts {
    /// Did this operation answer? An operation the probe never reported is
    /// treated as not answering — an unmeasured operation is not evidence.
    pub(crate) fn answered(&self, operation: &str) -> bool {
        self.operations.iter().any(|o| o.operation == operation && o.answered)
    }

    /// The declared type names, for resolving a property's range against
    /// the corpus rather than guessing.
    pub(crate) fn declared_names(&self) -> std::collections::BTreeSet<&str> {
        self.declarations.iter().map(|d| d.name.as_str()).collect()
    }
}

#[cfg(test)]
#[path = "facts_tests.rs"]
mod tests;
