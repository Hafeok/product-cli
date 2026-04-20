//! Hash-chain verification (FT-042, ADR-039).
//!
//! Verification is a pure read — it never modifies `requests.jsonl`, even when
//! it detects tampering. Four kinds of finding can be emitted:
//!
//! - E017 — per-entry hash mismatch
//! - E018 — chain break (prev-hash does not equal preceding entry's entry-hash)
//! - W021 — git tag with no corresponding log entry (tail-truncation detector)
//! - W-path-absolute — an entry carries an absolute `file:` value that has not
//!   been covered by a `path-relativize` migrate entry (FT-051)
//!
//! FT-051: when a `migrate` entry records the `path-relativize` sentinel,
//! earlier entries' stored `entry-hash` values will no longer match because
//! their `file:` fields were rewritten in place. The verifier tolerates that
//! mismatch as "this is pre-migration content, the migrate entry is the
//! authority" — otherwise the file-relativisation migration would permanently
//! break verification on every historical log.

use super::append::{GENESIS_PREV_HASH, load_all_entries};
use super::entry::{Entry, EntryPayload};
use super::paths::{looks_absolute, PATH_RELATIVIZE_SENTINEL};
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
    let tolerance = compute_relativize_tolerance(&entries);
    let entry_hashes_valid = check_entry_hashes(&entries, &tolerance, &mut findings);
    let chain_links_valid = check_chain(&entries, &mut findings);
    check_absolute_paths(&entries, &tolerance, &mut findings);
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

/// Metadata about `path-relativize` migrate entries in the log, used to
/// tolerate pre-migration entry-hash mismatches and pre-migration absolute
/// `file:` values (FT-051).
struct RelativizeTolerance {
    /// Line-number indices (in the `entries` slice) that mark a
    /// `path-relativize` migrate entry. Entries whose index is strictly less
    /// than the max of these indices are pre-migration.
    migrate_entry_indices: Vec<usize>,
}

impl RelativizeTolerance {
    fn has_any(&self) -> bool {
        !self.migrate_entry_indices.is_empty()
    }
    fn pre_migration(&self, ix: usize) -> bool {
        self.migrate_entry_indices.iter().any(|&m| ix < m)
    }
}

fn compute_relativize_tolerance(entries: &[(usize, Entry)]) -> RelativizeTolerance {
    let mut migrate_entry_indices = Vec::new();
    for (ix, (_, e)) in entries.iter().enumerate() {
        if let EntryPayload::Migrate { created, .. } = &e.payload {
            if created.iter().any(|s| s == PATH_RELATIVIZE_SENTINEL) {
                migrate_entry_indices.push(ix);
            }
        }
    }
    RelativizeTolerance { migrate_entry_indices }
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

fn check_entry_hashes(
    entries: &[(usize, Entry)],
    tolerance: &RelativizeTolerance,
    findings: &mut Vec<VerifyFinding>,
) -> usize {
    let mut valid = 0usize;
    for (ix, (line_no, entry)) in entries.iter().enumerate() {
        let computed = entry.compute_hash();
        if computed == entry.entry_hash {
            valid += 1;
        } else if tolerance.pre_migration(ix) {
            // FT-051: pre-migration entries have had their `file:` values
            // rewritten in place; the stored hash will no longer match. A
            // `path-relativize` migrate entry later in the chain is the
            // authority for the rewrite, so we count this line as valid.
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

/// Emit a `W-path-absolute` warning for any entry carrying an absolute
/// `file:` value that is not shielded by a subsequent `path-relativize`
/// migrate entry (FT-051). Rule:
///
/// - Lines **before** a migrate entry carrying the `path-relativize` sentinel
///   are tolerated — the migrate entry is the authority that documents why
///   those absolute paths exist (the rewrite either happened in place or the
///   paths were escape paths the migration chose to preserve).
/// - Lines **after** the latest migrate entry (or in any log with no migrate
///   entry at all) must have relative `file:` values. An absolute path in
///   such a line produces `W-path-absolute` and surfaces loudly.
fn check_absolute_paths(
    entries: &[(usize, Entry)],
    tolerance: &RelativizeTolerance,
    findings: &mut Vec<VerifyFinding>,
) {
    for (ix, (line_no, entry)) in entries.iter().enumerate() {
        if tolerance.pre_migration(ix) {
            continue;
        }
        let value = entry.to_value();
        let mut absolute_paths = Vec::new();
        collect_absolute_paths(&value, &mut absolute_paths);
        for p in absolute_paths {
            findings.push(VerifyFinding {
                code: "W-path-absolute".into(),
                severity: Severity::Warning,
                line: Some(*line_no),
                entry_id: Some(entry.id.clone()),
                message: format!("absolute `file:` path in log entry: {}", p),
                detail: Some(
                    "run `product request log migrate-paths` to rewrite, or verify that this write was intentional (escape path)".into(),
                ),
            });
        }
    }
    let _ = tolerance.has_any();
}

fn collect_absolute_paths(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, inner) in map.iter() {
                if k == "file" {
                    if let Some(s) = inner.as_str() {
                        if looks_absolute(s) {
                            out.push(s.to_string());
                            continue;
                        }
                    }
                }
                collect_absolute_paths(inner, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for x in arr {
                collect_absolute_paths(x, out);
            }
        }
        _ => {}
    }
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
                created: Vec::new(),
                changed: Vec::new(),
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
