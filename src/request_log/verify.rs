//! Hash-chain verification (FT-042, ADR-039).
//!
//! Verification is a pure read — it never modifies `requests.jsonl`, even when
//! it detects tampering. Three kinds of finding can be emitted:
//!
//! - E017 — per-entry hash mismatch
//! - E018 — chain break (prev-hash does not equal preceding entry's entry-hash)
//! - W021 — git tag with no corresponding log entry (tail-truncation detector)

use super::append::{GENESIS_PREV_HASH, load_all_entries};
use super::entry::{Entry, EntryPayload};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VerifyFinding {
    pub code: String,
    pub severity: Severity,
    pub line: Option<usize>,
    pub entry_id: Option<String>,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl VerifyFinding {
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

#[derive(Debug, Clone, Default)]
pub struct VerifyOptions {
    /// Also cross-reference git tags — detects tail truncation (W021).
    pub against_tags: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    /// Number of entries considered.
    pub entry_count: usize,
    /// Number of entries whose hash verified successfully.
    pub entry_hashes_valid: usize,
    /// Number of chain links that verified successfully.
    pub chain_links_valid: usize,
    /// All findings (errors + warnings) in order of discovery.
    pub findings: Vec<VerifyFinding>,
}

impl VerifyOutcome {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.is_error())
    }
    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|f| !f.is_error())
    }
    /// Exit code per ADR-009: 0 clean, 1 error, 2 warning.
    pub fn exit_code(&self) -> i32 {
        if self.has_errors() {
            1
        } else if self.has_warnings() {
            2
        } else {
            0
        }
    }
}

/// Verify the chain in `log_path` (optionally also cross-reference git tags in
/// `repo_root`). Pure read — the log is never written.
pub fn verify_log(log_path: &Path, repo_root: &Path, options: &VerifyOptions) -> VerifyOutcome {
    let (entries, mut findings) = load_entries_with_findings(log_path);
    let entry_hashes_valid = check_entry_hashes(&entries, &mut findings);
    let chain_links_valid = check_chain(&entries, &mut findings);
    if options.against_tags {
        cross_reference_tags(&entries, repo_root, &mut findings);
    }
    VerifyOutcome {
        entry_count: entries.len(),
        entry_hashes_valid,
        chain_links_valid,
        findings,
    }
}

fn load_entries_with_findings(log_path: &Path) -> (Vec<(usize, Entry)>, Vec<VerifyFinding>) {
    let mut entries: Vec<(usize, Entry)> = Vec::new();
    let mut findings: Vec<VerifyFinding> = Vec::new();
    if let Ok(v) = load_all_entries(log_path) {
        for r in v {
            match r {
                Ok((n, e)) => entries.push((n, e)),
                Err((n, msg)) => findings.push(VerifyFinding {
                    code: "E017".into(),
                    severity: Severity::Error,
                    line: Some(n),
                    entry_id: None,
                    message: "malformed log entry".into(),
                    detail: Some(msg),
                }),
            }
        }
    }
    (entries, findings)
}

fn check_entry_hashes(entries: &[(usize, Entry)], findings: &mut Vec<VerifyFinding>) -> usize {
    let mut valid = 0usize;
    for (line_no, entry) in entries {
        let computed = entry.compute_hash();
        if computed == entry.entry_hash {
            valid += 1;
        } else {
            findings.push(VerifyFinding {
                code: "E017".into(),
                severity: Severity::Error,
                line: Some(*line_no),
                entry_id: Some(entry.id.clone()),
                message: "entry hash mismatch".into(),
                detail: Some(format!(
                    "stored hash:   {}\n  computed hash: {}",
                    entry.entry_hash, computed
                )),
            });
        }
    }
    valid
}

fn check_chain(entries: &[(usize, Entry)], findings: &mut Vec<VerifyFinding>) -> usize {
    let mut prev_expected = GENESIS_PREV_HASH.to_string();
    let mut valid = 0usize;
    for (line_no, entry) in entries {
        if entry.prev_hash == prev_expected {
            valid += 1;
        } else {
            findings.push(VerifyFinding {
                code: "E018".into(),
                severity: Severity::Error,
                line: Some(*line_no),
                entry_id: Some(entry.id.clone()),
                message: "chain break".into(),
                detail: Some(format!(
                    "prev-hash in entry:       {}\n  actual hash of entry N-1: {}",
                    entry.prev_hash, prev_expected
                )),
            });
        }
        prev_expected = entry.entry_hash.clone();
    }
    valid
}

fn cross_reference_tags(entries: &[(usize, Entry)], repo_root: &Path, findings: &mut Vec<VerifyFinding>) {
    // Collect tag names from `git tag --list 'product/*'`.
    let out = match Command::new("git")
        .args(["tag", "--list", "product/*"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return,
    };
    let tags: Vec<String> = out
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if tags.is_empty() {
        return;
    }
    // Any verify entry that records `tag-created` is a match.
    let mut observed: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (_n, e) in entries {
        if let EntryPayload::Verify { tag_created: Some(t), .. } = &e.payload {
            observed.insert(t.clone());
        }
    }
    for tag in tags {
        if !observed.contains(&tag) {
            findings.push(VerifyFinding {
                code: "W021".into(),
                severity: Severity::Warning,
                line: None,
                entry_id: None,
                message: format!("git tag '{}' has no corresponding verify entry", tag),
                detail: Some("possible log truncation or tag created outside Product".into()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request_log::append::append_entry;
    use crate::request_log::entry::{Entry, EntryPayload, EntryType};
    use tempfile::tempdir;

    fn sample(prev: &str, id: &str) -> Entry {
        Entry {
            id: id.into(),
            applied_at: "2026-04-17T12:00:00Z".into(),
            applied_by: "git:T <t@e.com>".into(),
            commit: "abc".into(),
            entry_type: EntryType::Create,
            reason: "r".into(),
            prev_hash: prev.into(),
            entry_hash: "".into(),
            payload: EntryPayload::Apply {
                request: serde_json::Value::Null,
                created: vec![],
                changed: vec![],
            },
        }
    }

    #[test]
    fn clean_log_verifies() {
        let dir = tempdir().expect("tmp");
        let path = dir.path().join("log.jsonl");
        let a = append_entry(&path, sample(GENESIS_PREV_HASH, "req-20260417-001")).expect("a");
        let b = append_entry(&path, sample(&a.entry_hash, "req-20260417-002")).expect("b");
        let _ = b;
        let out = verify_log(&path, dir.path(), &VerifyOptions::default());
        assert_eq!(out.entry_count, 2);
        assert_eq!(out.entry_hashes_valid, 2);
        assert_eq!(out.chain_links_valid, 2);
        assert!(out.findings.is_empty());
    }
}
