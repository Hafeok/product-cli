//! The id concordance — permanent aliases across the M8 ledger migration.
//!
//! `.ddd/concordance.yaml` maps every historical id the migration touched
//! to its ledger identity of record (`dec:<namespace>/<ulid>`), both
//! directions. The aliases are **permanent, not transitional** (M8 ruling
//! 6): code comments, graph edges, ratification notes, and conversation
//! records cite the historical ids, and those references must resolve
//! indefinitely. A row may instead state a disposition — an id that
//! deliberately did not migrate (a claim, which remains the ddd
//! ontology's own) says so explicitly rather than being absent.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The concordance file: format-versioned like every store entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concordance {
    pub format: u32,
    /// Why this file exists and why it never goes away.
    pub note: String,
    #[serde(default)]
    pub rows: Vec<Row>,
}

/// One id mapping, or one explicit non-migration disposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    /// The historical id (`dec/ddd/...`, `risk/...`, `seam/...`,
    /// `<namespace>/<rule>`, or a claim id).
    pub historical: String,
    /// The ledger identity of record, when the entry migrated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<String>,
    /// What the historical id names: decision | risk-acceptance | seam |
    /// manifest-entry | claim | question.
    pub kind: String,
    /// The stated stance for a row with no ledger identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
}

/// Where the concordance lives, under a `.ddd/` directory.
pub fn path(ddd_dir: &Path) -> PathBuf {
    ddd_dir.join("concordance.yaml")
}

/// Load the concordance when one exists; a store that never migrated
/// has none, and that is not an error.
pub fn load(ddd_dir: &Path) -> Option<Concordance> {
    let text = std::fs::read_to_string(path(ddd_dir)).ok()?;
    serde_yaml::from_str(&text).ok()
}

impl Concordance {
    /// The ledger identity a historical id migrated to.
    pub fn ledger_of(&self, historical: &str) -> Option<&str> {
        self.rows
            .iter()
            .find(|r| r.historical == historical)
            .and_then(|r| r.ledger.as_deref())
    }

    /// The historical id behind a ledger identity.
    pub fn historical_of(&self, ledger: &str) -> Option<&str> {
        self.rows
            .iter()
            .find(|r| r.ledger.as_deref() == Some(ledger))
            .map(|r| r.historical.as_str())
    }

    /// The row for a historical id, mapping or disposition alike.
    pub fn row(&self, historical: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.historical == historical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_both_directions_and_states_dispositions() {
        let c = Concordance {
            format: 1,
            note: "permanent".into(),
            rows: vec![
                Row {
                    historical: "dec/ddd/lsp-as-seam".into(),
                    ledger: Some("dec:hafeok.ddd/01ABC".into()),
                    kind: "decision".into(),
                    disposition: None,
                },
                Row {
                    historical: "DDD-method-07".into(),
                    ledger: None,
                    kind: "claim".into(),
                    disposition: Some("remains a .ddd claim".into()),
                },
            ],
        };
        assert_eq!(c.ledger_of("dec/ddd/lsp-as-seam"), Some("dec:hafeok.ddd/01ABC"));
        assert_eq!(c.historical_of("dec:hafeok.ddd/01ABC"), Some("dec/ddd/lsp-as-seam"));
        assert_eq!(c.ledger_of("DDD-method-07"), None);
        assert!(c.row("DDD-method-07").and_then(|r| r.disposition.as_deref()).is_some());
    }
}
