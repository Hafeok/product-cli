//! Interception event rows — the correspondence dataset (PRD §8).
//!
//! Every interception outcome on a contract-surface change lands as one
//! YAML row under `.ddd/seams/events/`, carrying the LSP-derived
//! structural metadata (symbol, kind, reference count at the event).
//! Rows live in a subdirectory so they never collide with the seam
//! *declarations* in `.ddd/seams/*.yaml`; M5 files claims against this
//! schema, so its fields are the contract.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use product_core::error::Result;
use product_core::fileops::write_file_atomic;
use serde::{Deserialize, Serialize};

/// Where event rows live under the store directory.
pub const EVENTS_DIR: &str = "seams/events";

/// One interception outcome with its structural metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeamEvent {
    /// Entry format version (v1 is current).
    pub format: u32,
    /// Stable id `seam-event/<seq>`.
    pub id: String,
    /// RFC3339 timestamp, supplied by the serving layer.
    pub at: String,
    pub language: String,
    /// The predicate-vocabulary artifact class the mode was read for.
    pub artifact_class: String,
    /// The interception mode in force (`enforce | warn`).
    pub mode: String,
    /// Repo-relative file the edit targeted.
    pub file: String,
    pub symbol: String,
    /// Container path of the symbol (empty at top level).
    #[serde(default)]
    pub container: String,
    /// Adapter-normalized symbol kind.
    pub kind: String,
    /// The change: added | removed | signature-changed | decorator-changed
    /// | visibility-changed.
    pub change: String,
    pub visibility: String,
    pub signature: String,
    /// References to the symbol at event time, when resolvable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<u64>,
    /// The policy row that classified the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_claim: Option<String>,
    /// applied-linked | applied-warned | rejected.
    pub outcome: String,
    /// The seam/pattern declaration the edit linked to, when one matched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_declaration: Option<String>,
    /// Adapter extras (e.g. Bicep `ground_provenance`).
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

/// The next sequence number: one past the row count on disk.
pub fn next_seq(ddd_dir: &Path) -> u32 {
    let dir = ddd_dir.join(EVENTS_DIR);
    let count = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.path().extension().map(|x| x == "yaml" || x == "yml").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);
    count as u32 + 1
}

/// Write one row atomically; the filename carries the sequence plus slug.
pub fn save(ddd_dir: &Path, event: &SeamEvent) -> Result<PathBuf> {
    let seq = event.id.rsplit('/').next().unwrap_or("0");
    let slug: String = event
        .symbol
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let path = ddd_dir.join(EVENTS_DIR).join(format!("{seq:0>4}-{slug}.yaml"));
    let yaml = serde_yaml::to_string(event).map_err(|e| {
        product_core::error::ProductError::IoError(format!("serialize seam event: {e}"))
    })?;
    write_file_atomic(&path, &yaml)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(seq: u32) -> SeamEvent {
        SeamEvent {
            format: 1,
            id: format!("seam-event/{seq}"),
            at: "2026-08-06T12:00:00Z".into(),
            language: "csharp".into(),
            artifact_class: "code".into(),
            mode: "enforce".into(),
            file: "src/Api.cs".into(),
            symbol: "Ping".into(),
            container: "Api".into(),
            kind: "method".into(),
            change: "added".into(),
            visibility: "public".into(),
            signature: "void Ping(int count)".into(),
            reference_count: Some(3),
            rule: Some("cs-add-exposed".into()),
            rule_claim: None,
            outcome: "rejected".into(),
            linked_declaration: None,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn rows_sequence_and_round_trip() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ddd = tmp.path().join(".ddd");
        assert_eq!(next_seq(&ddd), 1);
        let path = save(&ddd, &event(1)).expect("save");
        assert!(path.to_string_lossy().contains("seams/events/0001-ping.yaml"));
        assert_eq!(next_seq(&ddd), 2);
        let text = std::fs::read_to_string(&path).expect("read");
        let back: SeamEvent = serde_yaml::from_str(&text).expect("parse");
        assert_eq!(back.reference_count, Some(3));
        assert_eq!(back.outcome, "rejected");
    }
}
