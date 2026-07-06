//! Authoring scope — a tool's declared write-boundary over the What (§14).
//!
//! An authoring scope makes a tool (Figma, a legacy schema, an Event-Modeling
//! board) a *bounded co-author* of the What: it declares which What-element
//! kinds the tool MAY author (`authors`), which it MUST NOT (`excluded`), and
//! optionally the funnel slice its authorship enters (`process-slice`). The
//! kind vocabulary is the framework's own ontology (§14.2); a scope is a subset
//! over kinds the framework already names, introducing no ontology of its own.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// How completely a tool authors a kind (§14.2): `sufficient` — it can author
/// the kind fully; `partial` — it contributes, other authors expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Completeness {
    Sufficient,
    Partial,
}

/// One kind this tool MAY author, with how completely and through which channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authored {
    /// A What-element kind from the framework ontology.
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completeness: Option<Completeness>,
    /// How authored meaning is carried in the tool (e.g. `native-annotation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

/// A tool-adapter's declared authoring scope over the What (§14.2). The adapter
/// *enforces* this scope; it never *owns* one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthoringScope {
    /// The authoring surface (e.g. `figma`, `jira`, `legacy-crm-schema`).
    pub tool: String,
    /// The adapter that enforces this scope (e.g. `figma-bridge`).
    pub adapter: String,
    /// What-element kinds this tool MAY author.
    #[serde(default)]
    pub authors: Vec<Authored>,
    /// What-element kinds this tool MUST NOT author (its Polanyi boundary).
    #[serde(default)]
    pub excluded: Vec<String>,
    /// OPTIONAL. The funnel slice this authorship enters (§2.1).
    #[serde(rename = "process-slice", default, skip_serializing_if = "Option::is_none")]
    pub process_slice: Option<String>,
}

/// The framework's authorable What-element kind vocabulary (§14.2). A scope's
/// `authors`/`excluded` kinds must come from here (or [`DERIVED_KINDS`]).
pub const AUTHORABLE_KINDS: &[&str] = &[
    "domain-structure",
    "trigger",
    "command",
    "event",
    "view",
    "decider",
    "projector",
    "ui-step",
    "aio",
    "state-annotation",
    "page-graph",
    "context-of-use",
    "accessibility-criteria",
    "content-reference",
    "journey",
    "quality-demand",
    "data-shape",
    "token-source",
];

/// Kinds the framework *derives* rather than authors (§14.2): a read model's
/// `state-space` derives from the projection's shape, so no tool authors it —
/// a derived kind belongs in every tool's `excluded` list, never in `authors`.
pub const DERIVED_KINDS: &[&str] = &["state-space"];

impl AuthoringScope {
    /// Parse a scope from YAML (which also parses the JSON reference form).
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid authoring-scope: {e}")))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize authoring-scope: {e}")))
    }

    /// The kinds this tool declares it may author.
    pub fn authored_kinds(&self) -> Vec<&str> {
        self.authors.iter().map(|a| a.kind.as_str()).collect()
    }
}

/// True if `kind` is a known What-element kind (authorable or derived).
pub fn is_known_kind(kind: &str) -> bool {
    AUTHORABLE_KINDS.contains(&kind) || DERIVED_KINDS.contains(&kind)
}

/// True if `kind` is derived by the framework rather than authored.
pub fn is_derived_kind(kind: &str) -> bool {
    DERIVED_KINDS.contains(&kind)
}

/// Validate a scope (§14.2): schema wholeness (tool, adapter, ≥1 author),
/// kind-vocabulary membership, and the derived-kind rule (a derived kind must
/// never appear in `authors`).
pub fn validate_scope(scope: &AuthoringScope) -> Vec<Violation> {
    let mut out = Vec::new();
    let tool = if scope.tool.trim().is_empty() { "<scope>" } else { scope.tool.trim() };

    if scope.tool.trim().is_empty() {
        out.push(v(tool, "tool", "§14.2 a scope must name the authoring surface (`tool`)."));
    }
    if scope.adapter.trim().is_empty() {
        out.push(v(tool, "adapter", "§14.2 a scope must name the enforcing `adapter`."));
    }
    if scope.authors.is_empty() {
        out.push(v(tool, "authors", "§14.2 a scope must declare at least one authored kind."));
    }

    for a in &scope.authors {
        if is_derived_kind(&a.kind) {
            out.push(v(tool, "authors", &format!(
                "§14.2 '{}' is a DERIVED kind — no tool authors it; move it to `excluded`.", a.kind)));
        } else if !is_known_kind(&a.kind) {
            out.push(v(tool, "authors", &format!(
                "§14.2 '{}' is not a framework What-element kind.", a.kind)));
        }
    }
    for e in &scope.excluded {
        if !is_known_kind(e) {
            out.push(v(tool, "excluded", &format!(
                "§14.2 excluded kind '{e}' is not a framework What-element kind.")));
        }
    }
    out
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
#[path = "authoring_scope_tests.rs"]
mod tests;
