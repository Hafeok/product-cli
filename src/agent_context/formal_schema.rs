//! AISP formal block grammar reference embedded in `product schema` output (FT-049).
//!
//! Exposes a single pure function returning a markdown fragment. The text
//! lives alongside this module as `formal_schema.md` so the function body
//! stays short; the authoritative block type labels live in
//! `src/formal/parser.rs::parse_formal_blocks_with_diagnostics` and this
//! text stays in sync with that enum by eyeball.

const FORMAL_BLOCK_SCHEMA_MD: &str = include_str!("formal_schema.md");

/// Markdown text describing the five AISP formal blocks, their examples, and
/// which `tc-type` values mandate which block (W004 / G002 contract).
pub fn formal_block_schema() -> String {
    FORMAL_BLOCK_SCHEMA_MD.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_all_five_block_labels() {
        let s = formal_block_schema();
        assert!(s.contains("\u{27E6}\u{03A3}:Types\u{27E7}"), "missing Sigma-Types label");
        assert!(s.contains("\u{27E6}\u{0393}:Invariants\u{27E7}"), "missing Gamma-Invariants label");
        assert!(s.contains("\u{27E6}\u{039B}:Scenario\u{27E7}"), "missing Lambda-Scenario label");
        assert!(s.contains("\u{27E6}\u{039B}:ExitCriteria\u{27E7}"), "missing Lambda-ExitCriteria label");
        assert!(s.contains("\u{27E6}\u{0395}\u{27E7}"), "missing Epsilon evidence label");
    }

    #[test]
    fn contains_human_readable_block_names() {
        let s = formal_block_schema();
        for name in &["Sigma-Types", "Gamma-Invariants", "Lambda-Scenario", "Lambda-ExitCriteria", "Epsilon"] {
            assert!(s.contains(name), "missing human-readable block name '{}'", name);
        }
    }

    #[test]
    fn documents_required_by_contract() {
        let s = formal_block_schema();
        assert!(s.contains("W004"), "formal block schema should mention W004");
        assert!(s.contains("invariant"));
        assert!(s.contains("chaos"));
        assert!(s.contains("exit-criteria"));
    }
}
