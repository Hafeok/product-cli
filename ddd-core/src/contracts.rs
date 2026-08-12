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

/// A contract-surface change no signed declaration discharges — the CI
/// finding invariant 5 exists for.
#[derive(Debug, Serialize)]
pub struct Undischarged {
    pub id: String,
    pub file: String,
    pub symbol: String,
    pub reason: String,
}

/// Judge every surface event in the report against the store's signed
/// bindings. A file's transition is discharged by a *chain* of bindings
/// composing from its before-hash to its after-hash (each hop was
/// individually classified and signed at edit time); an event is
/// discharged when a binding on such a chain names its symbol. A
/// declaration that does not bind to the change it claims to discharge
/// is a finding, not a warning (M8 ruling).
pub fn check_discharge(
    report: &ContractDiffReport,
    seams: &[crate::seam::SeamDeclaration],
) -> Vec<Undischarged> {
    let mut out = Vec::new();
    for file in &report.files {
        let bindings: Vec<&crate::binding::SeamBinding> = seams
            .iter()
            .flat_map(|s| s.bindings.iter())
            .filter(|b| b.file == file.file)
            .collect();
        let on_path = on_path_bindings(&bindings, &file.before, &file.after);
        for e in file.events.iter().filter(|e| e.event.surface) {
            let symbol = &e.event.facts.name;
            let reason = match &on_path {
                None => format!(
                    "no signed binding chain composes {} -> {} for this file",
                    short(&file.before),
                    short(&file.after)
                ),
                Some(chain) if !chain.iter().any(|b| &b.symbol == symbol) => format!(
                    "a binding chain covers the file transition, but no binding on it names `{symbol}`"
                ),
                Some(_) => continue,
            };
            out.push(Undischarged {
                id: e.id.clone(),
                file: file.file.clone(),
                symbol: symbol.clone(),
                reason,
            });
        }
    }
    out
}

/// The bindings lying on some chain from `from` to `to`, or `None` when
/// no chain exists. Forward-reachability from `from` intersected with
/// backward-reachability from `to` over the binding edges.
fn on_path_bindings<'a>(
    bindings: &[&'a crate::binding::SeamBinding],
    from: &str,
    to: &str,
) -> Option<Vec<&'a crate::binding::SeamBinding>> {
    let mut forward = reach(bindings, from, |b| (&b.before, &b.after));
    forward.insert(from.to_string());
    if !forward.contains(to) {
        return None;
    }
    let mut backward = reach(bindings, to, |b| (&b.after, &b.before));
    backward.insert(to.to_string());
    Some(
        bindings
            .iter()
            .filter(|b| forward.contains(&b.before) && backward.contains(&b.after))
            .copied()
            .collect(),
    )
}

fn reach<'a>(
    bindings: &[&'a crate::binding::SeamBinding],
    start: &str,
    edge: impl Fn(&'a crate::binding::SeamBinding) -> (&'a String, &'a String),
) -> std::collections::BTreeSet<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut frontier = vec![start.to_string()];
    while let Some(node) = frontier.pop() {
        for b in bindings {
            let (tail, head) = edge(b);
            if tail == &node && seen.insert(head.clone()) {
                frontier.push(head.clone());
            }
        }
    }
    seen
}

fn short(hash: &str) -> &str {
    let hex = hash.strip_prefix("sha256:").unwrap_or(hash);
    hex.get(..12).unwrap_or(hex)
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
