//! Repository-diff contract findings — the CI half of enforcement (M8).
//!
//! The vocabulary the shared classifier's diff path reports in: one
//! [`FileContracts`] per changed governed file, one [`ContractEvent`] per
//! classified surface change, each carrying a stable finding id so CI can
//! track a finding across runs (spec §6's named gap). Classification
//! itself happens in `ddd-lsp` (`revdiff`), through the same
//! `classify_edit` the interceptor uses — spec invariant 4.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::surface::SurfaceEvent;

/// Marker for a side of a transition where the file does not exist.
/// Distinct from the hash of an empty file, which is real content.
pub const ABSENT: &str = "absent";

/// Content-address a file's bytes: `sha256:<64 lowercase hex>`.
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

/// The classifier's reading of one revision range.
#[derive(Debug, Default, Serialize)]
pub struct ContractDiffReport {
    /// Resolved base commit id.
    pub base: String,
    /// Resolved head commit id, or `worktree`.
    pub head: String,
    pub files: Vec<FileContracts>,
    /// Files the classifier could not read — a host that failed or text
    /// that is not UTF-8. Reported explicitly, never as "no findings"
    /// (spec invariant 9).
    pub skipped: Vec<SkippedFile>,
}

impl ContractDiffReport {
    /// Every contract-surface event across the range.
    pub fn surface_events(&self) -> impl Iterator<Item = (&FileContracts, &ContractEvent)> {
        self.files
            .iter()
            .flat_map(|f| f.events.iter().filter(|e| e.event.surface).map(move |e| (f, e)))
    }

    /// Head label for display: a short commit id, or `worktree`.
    pub fn head_short(&self) -> &str {
        if self.head == "worktree" {
            &self.head
        } else {
            &self.head[..12.min(self.head.len())]
        }
    }
}

/// One changed file a registered adapter claims, with its classification.
#[derive(Debug, Serialize)]
pub struct FileContracts {
    /// Repo-relative path, forward slashes.
    pub file: String,
    pub language: String,
    pub artifact_class: String,
    /// Content hash of the base side, or [`ABSENT`].
    pub before: String,
    /// Content hash of the head side, or [`ABSENT`].
    pub after: String,
    pub events: Vec<ContractEvent>,
}

/// One classified symbol change, with its stable finding id.
#[derive(Debug, Serialize)]
pub struct ContractEvent {
    pub id: String,
    #[serde(flatten)]
    pub event: SurfaceEvent,
}

/// A file the diff touched but the classifier could not judge.
#[derive(Debug, Serialize)]
pub struct SkippedFile {
    pub file: String,
    pub reason: String,
}

/// The stable finding id for one classified change:
/// `contract/<language>/<file>#<container>/<symbol>@<change>`, with
/// whitespace in symbol names collapsed to `-` so the id stays one token.
pub fn finding_id(language: &str, file: &str, event: &SurfaceEvent) -> String {
    let change = match serde_json::to_value(event.change) {
        Ok(serde_json::Value::String(s)) => s,
        _ => "changed".to_string(),
    };
    let subject = if event.facts.container.is_empty() {
        event.facts.name.clone()
    } else {
        format!("{}/{}", event.facts.container, event.facts.name)
    };
    let subject: String =
        subject.chars().map(|c| if c.is_whitespace() { '-' } else { c }).collect();
    format!("contract/{language}/{file}#{subject}@{change}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{ChangeKind, SymbolFacts};

    fn event(container: &str, name: &str, change: ChangeKind) -> SurfaceEvent {
        SurfaceEvent {
            change,
            facts: SymbolFacts {
                name: name.into(),
                container: container.into(),
                kind: "fn".into(),
                visibility: "public".into(),
                signature: "fn x()".into(),
                decorators: Vec::new(),
                exported: false,
                extra: Default::default(),
                sel_line: 0,
                sel_char: 0,
            },
            before: None,
            surface: true,
            rule: None,
            rule_claim: None,
        }
    }

    #[test]
    fn finding_ids_are_stable_and_single_token() {
        let e = event("", "impl Trait for Type", ChangeKind::SignatureChanged);
        let id = finding_id("rust", "src/lib.rs", &e);
        assert_eq!(id, "contract/rust/src/lib.rs#impl-Trait-for-Type@signature-changed");
        assert!(!id.contains(' '));
        let nested = event("Outer", "x", ChangeKind::Added);
        assert_eq!(
            finding_id("rust", "src/lib.rs", &nested),
            "contract/rust/src/lib.rs#Outer/x@added"
        );
    }

    #[test]
    fn content_hash_distinguishes_empty_from_absent() {
        let empty = content_hash(b"");
        assert!(empty.starts_with("sha256:"));
        assert_ne!(empty, ABSENT);
        assert_eq!(content_hash(b"a"), content_hash(b"a"));
        assert_ne!(content_hash(b"a"), content_hash(b"b"));
    }
}
