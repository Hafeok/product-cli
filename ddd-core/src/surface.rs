//! Contract-surface vocabulary — symbol facts, change events, policy rows.
//!
//! A policy table is *data*: an array of [`PolicyRow`]s, each one a
//! falsifiable claim ("in this language, this change on this kind at this
//! visibility is boundary-forming"). The classifier walks the table; wrong
//! rows get amended in the adapter's table, never in classifier logic
//! (PRD §9, `dec/ddd/adapter-policy-tables`).
//!
//! The mechanism is language-neutral, so it lives here rather than in the
//! LSP crate: `ddd-lsp` supplies facts from a language server, while
//! [`crate::what`] supplies them from the framework's own typed graph.

use std::collections::BTreeMap;

use serde::Serialize;

/// What one symbol *is*, as derived from LSP output plus source slicing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolFacts {
    pub name: String,
    /// `Outer/Inner` container path; empty at top level.
    pub container: String,
    /// Adapter-normalized kind (e.g. `class`, `method`, `param`).
    pub kind: String,
    /// Adapter-normalized *effective* visibility (container-capped).
    pub visibility: String,
    /// Normalized declaration signature (whitespace-collapsed, body-free).
    pub signature: String,
    /// Decorators/attributes attached to the declaration.
    pub decorators: Vec<String>,
    /// Carries a configured exported-endpoint marker (e.g. `[McpServerTool]`).
    pub exported: bool,
    /// Adapter-specific extras (e.g. Bicep ground provenance).
    pub extra: BTreeMap<String, String>,
    /// Selection position, for follow-up requests (references count).
    pub sel_line: u64,
    pub sel_char: u64,
}

impl SymbolFacts {
    /// The identity a symbol keeps across an edit: container + kind + name.
    pub fn key(&self) -> (String, String, String) {
        (self.container.clone(), self.kind.clone(), self.name.clone())
    }
}

/// How a symbol changed between the before-state and the after-state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    Added,
    Removed,
    SignatureChanged,
    DecoratorChanged,
    VisibilityChanged,
}

/// One falsifiable table row. Empty match lists mean "any".
#[derive(Debug, Clone, Copy)]
pub struct PolicyRow {
    /// Stable row id, e.g. `cs-add-public-member`.
    pub id: &'static str,
    pub changes: &'static [ChangeKind],
    pub kinds: &'static [&'static str],
    pub visibilities: &'static [&'static str],
    /// Row only matches symbols carrying a configured exported marker.
    pub exported_only: bool,
    pub surface: bool,
    /// The claim the row states, quoted in demands plus event rows.
    pub claim: &'static str,
}

impl PolicyRow {
    fn matches(&self, change: ChangeKind, facts: &SymbolFacts, visibility: &str) -> bool {
        (self.changes.is_empty() || self.changes.contains(&change))
            && (self.kinds.is_empty() || self.kinds.contains(&facts.kind.as_str()))
            && (self.visibilities.is_empty() || self.visibilities.contains(&visibility))
            && (!self.exported_only || facts.exported)
    }
}

/// One classified change: the event the interceptor acts on.
#[derive(Debug, Clone, Serialize)]
pub struct SurfaceEvent {
    pub change: ChangeKind,
    /// The symbol's facts on the side that best describes the change
    /// (after-state for additions/changes, before-state for removals).
    pub facts: SymbolFacts,
    /// The before-state facts, when the symbol existed before.
    pub before: Option<SymbolFacts>,
    pub surface: bool,
    /// The policy row that decided, when one matched.
    pub rule: Option<&'static str>,
    pub rule_claim: Option<&'static str>,
}

/// Walk the table top-down; first matching row decides. No match means
/// non-surface (the table names what forms a boundary, not what doesn't).
pub fn decide(
    rows: &[PolicyRow],
    change: ChangeKind,
    facts: &SymbolFacts,
    visibility: &str,
) -> (bool, Option<&'static str>, Option<&'static str>) {
    for row in rows {
        if row.matches(change, facts, visibility) {
            return (row.surface, Some(row.id), Some(row.claim));
        }
    }
    (false, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(kind: &str, vis: &str) -> SymbolFacts {
        SymbolFacts {
            name: "X".into(),
            container: String::new(),
            kind: kind.into(),
            visibility: vis.into(),
            signature: "sig".into(),
            decorators: Vec::new(),
            exported: false,
            extra: BTreeMap::new(),
            sel_line: 0,
            sel_char: 0,
        }
    }

    const ROWS: &[PolicyRow] = &[
        PolicyRow {
            id: "t-add-public",
            changes: &[ChangeKind::Added],
            kinds: &["method"],
            visibilities: &["public"],
            exported_only: false,
            surface: true,
            claim: "adding a public method is boundary-forming",
        },
        PolicyRow {
            id: "t-private",
            changes: &[],
            kinds: &[],
            visibilities: &["private"],
            exported_only: false,
            surface: false,
            claim: "private members are not surface",
        },
    ];

    #[test]
    fn first_matching_row_decides() {
        let f = facts("method", "public");
        let (surface, rule, _) = decide(ROWS, ChangeKind::Added, &f, "public");
        assert!(surface);
        assert_eq!(rule, Some("t-add-public"));
    }

    #[test]
    fn empty_match_lists_mean_any() {
        let f = facts("method", "private");
        let (surface, rule, _) = decide(ROWS, ChangeKind::Added, &f, "private");
        assert!(!surface);
        assert_eq!(rule, Some("t-private"));
    }

    #[test]
    fn no_matching_row_defaults_to_non_surface() {
        let f = facts("method", "internal");
        let (surface, rule, _) = decide(ROWS, ChangeKind::Removed, &f, "internal");
        assert!(!surface);
        assert_eq!(rule, None);
    }
}
