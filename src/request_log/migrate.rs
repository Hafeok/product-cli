//! One-shot path migration from `.product/request-log.jsonl` to
//! `requests.jsonl` (FT-042, ADR-039 decision 1).
//!
//! On first run of the new binary:
//! - If both old and new paths exist, do nothing (the migration already happened).
//! - If only the new path exists, nothing to do.
//! - If only the old path exists, replay the old entries forward into the new
//!   log (re-computing `prev-hash` / `entry-hash` for each), append a `migrate`
//!   entry documenting the move, and rename the old file with `.migrated` suffix.

use super::append::{append_entry, compute_entry_id, GENESIS_PREV_HASH};
use super::entry::{ArtifactRef, Entry, EntryPayload, EntryType, MIGRATE_LOG_SENTINEL};
use super::paths::{looks_absolute, path_relativize, PATH_RELATIVIZE_SENTINEL};
use super::{legacy_log_path, log_path};
use std::io::BufRead;
use std::path::Path;

/// If the legacy log exists and the new one does not, migrate entries forward
/// and append a `migrate` entry documenting the move.
///
/// Idempotent — returns `Ok(false)` if nothing was migrated.
pub fn migrate_if_needed(repo_root: &Path, requests_rel: Option<&str>) -> std::io::Result<bool> {
    let new_path = log_path(repo_root, requests_rel);
    let legacy = legacy_log_path(repo_root);
    if !legacy.exists() || new_path.exists() {
        return Ok(false);
    }

    let prev_hash = replay_legacy_lines(&legacy, &new_path)?;
    write_final_migrate_entry(&new_path, prev_hash)?;
    let renamed = legacy.with_extension("jsonl.migrated");
    let _ = std::fs::rename(&legacy, &renamed);
    Ok(true)
}

fn replay_legacy_lines(legacy: &Path, new_path: &Path) -> std::io::Result<String> {
    let file = std::fs::File::open(legacy)?;
    let reader = std::io::BufReader::new(file);
    let mut prev_hash = GENESIS_PREV_HASH.to_string();
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(entry) = build_entry_from_legacy(&line, &prev_hash, new_path) {
            let written = append_entry(new_path, entry)?;
            prev_hash = written.entry_hash;
        }
    }
    Ok(prev_hash)
}

fn build_entry_from_legacy(line: &str, prev_hash: &str, new_path: &Path) -> Option<Entry> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;
    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_string();
    let reason = obj.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let kind = obj.get("type").and_then(|v| v.as_str()).unwrap_or("create");
    let entry_type = EntryType::parse(kind).unwrap_or(EntryType::Create);
    let created = extract_artifact_refs(obj, "created");
    let changed = extract_artifact_refs(obj, "changed");
    let id = compute_entry_id(&timestamp, new_path);
    Some(Entry {
        id,
        applied_at: timestamp,
        applied_by: "git:migrated <legacy>".into(),
        commit: "".into(),
        entry_type,
        reason,
        prev_hash: prev_hash.to_string(),
        entry_hash: "".into(),
        payload: EntryPayload::Apply {
            request: serde_json::Value::Null,
            created,
            changed,
        },
    })
}

fn extract_artifact_refs(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Vec<ArtifactRef> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| {
                    if let Some(m) = v.as_object() {
                        let id = m
                            .get("id")
                            .or_else(|| m.get("ref_name"))
                            .and_then(|x| x.as_str())?
                            .to_string();
                        let file = m.get("file").and_then(|x| x.as_str()).map(String::from);
                        Some(ArtifactRef { id, file })
                    } else {
                        v.as_str().map(ArtifactRef::id_only)
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn write_final_migrate_entry(new_path: &Path, prev_hash: String) -> std::io::Result<()> {
    let applied_at = chrono::Utc::now().to_rfc3339();
    let id = compute_entry_id(&applied_at, new_path);
    let migrate_entry = Entry {
        id,
        applied_at,
        applied_by: "git:migrated <legacy>".into(),
        commit: "".into(),
        entry_type: EntryType::Migrate,
        reason: "Promoted .product/request-log.jsonl to requests.jsonl".into(),
        prev_hash,
        entry_hash: "".into(),
        payload: EntryPayload::Migrate {
            sources: vec![super::LEGACY_LOG_PATH.into()],
            created: vec![MIGRATE_LOG_SENTINEL.into()],
        },
    };
    append_entry(new_path, migrate_entry).map(|_| ())
}

/// Outcome of a `rewrite_paths` pass (FT-051).
#[derive(Debug, Clone, Default)]
pub struct RewritePathsOutcome {
    /// IDs of entries whose lines were rewritten.
    pub rewritten: Vec<String>,
    /// ID of the new migrate entry, if one was appended.
    pub migrate_entry_id: Option<String>,
}

impl RewritePathsOutcome {
    pub fn is_noop(&self) -> bool {
        self.rewritten.is_empty() && self.migrate_entry_id.is_none()
    }
}

/// Rewrite absolute `file:` fields inside existing log lines to repo-relative
/// form, preserving each line's stored `entry-hash`, and append a single
/// `migrate` entry carrying the `path-relativize` sentinel (FT-051).
///
/// Hashes are **not** recomputed for rewritten lines — that would break the
/// chain. The migrate entry is the authority for the rewrite; `verify_log`
/// tolerates entry-hash mismatches on lines preceding a migrate entry that
/// records the `path-relativize` sentinel.
///
/// Idempotent: if no line still carries an unmigrated absolute path, the
/// outcome is a no-op (no new migrate entry is appended).
pub fn rewrite_paths(repo_root: &Path, requests_rel: Option<&str>) -> std::io::Result<RewritePathsOutcome> {
    let log_p = log_path(repo_root, requests_rel);
    if !log_p.exists() {
        return Ok(RewritePathsOutcome::default());
    }
    let content = std::fs::read_to_string(&log_p)?;
    let (rewritten_ids, new_lines) = scan_and_rewrite_lines(&content, repo_root);
    if rewritten_ids.is_empty() {
        return Ok(RewritePathsOutcome::default());
    }
    write_updated_lines(&log_p, &new_lines, content.len())?;
    let migrate_id = append_relativize_migrate(&log_p, repo_root, rewritten_ids.clone())?;
    Ok(RewritePathsOutcome {
        rewritten: rewritten_ids,
        migrate_entry_id: Some(migrate_id),
    })
}

/// Scan `content` for entries with absolute `file:` values and return
/// (rewritten-IDs, fully-updated-line-vector). Only lines after any prior
/// `path-relativize` migrate entry are considered — earlier absolute paths
/// are already documented.
fn scan_and_rewrite_lines(content: &str, repo_root: &Path) -> (Vec<String>, Vec<String>) {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, l)| line_marks_path_relativize(l))
        .map(|(i, _)| i + 1)
        .unwrap_or(0);
    let mut rewritten_ids: Vec<String> = Vec::new();
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    for (i, line) in lines.iter().enumerate().skip(start) {
        if let Some((id, rewritten)) = try_rewrite_line(line, repo_root) {
            rewritten_ids.push(id);
            new_lines[i] = rewritten;
        }
    }
    (rewritten_ids, new_lines)
}

/// If `line` is a log entry with at least one absolute `file:` value, return
/// the entry ID and the rewritten canonical line.
fn try_rewrite_line(line: &str, repo_root: &Path) -> Option<(String, String)> {
    if line.trim().is_empty() {
        return None;
    }
    let mut v: serde_json::Value = serde_json::from_str(line).ok()?;
    let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if id.is_empty() {
        return None;
    }
    if !rewrite_paths_in_value(&mut v, repo_root) {
        return None;
    }
    Some((id, super::canonical::canonical_json(&v)))
}

/// Re-serialise `new_lines` and atomically write them back to the log path.
fn write_updated_lines(log_p: &Path, new_lines: &[String], hint_cap: usize) -> std::io::Result<()> {
    let mut out = String::with_capacity(hint_cap);
    for l in new_lines {
        if !l.is_empty() {
            out.push_str(l);
            out.push('\n');
        }
    }
    crate::fileops::write_file_atomic(log_p, &out)
        .map_err(|e| std::io::Error::other(e.to_string()))
}

/// Append a single `migrate` entry carrying the `path-relativize` sentinel
/// and return its entry ID.
fn append_relativize_migrate(
    log_p: &Path,
    repo_root: &Path,
    rewritten_ids: Vec<String>,
) -> std::io::Result<String> {
    let applied_by = super::git_identity::resolve_applied_by(repo_root)
        .unwrap_or_else(|_| "local:unknown".into());
    let commit = super::git_identity::resolve_commit(repo_root);
    let reason = format!(
        "FT-051 path-relativize: rewrote {} entr{} to repo-relative form",
        rewritten_ids.len(),
        if rewritten_ids.len() == 1 { "y" } else { "ies" }
    );
    let migrate = super::append::append_migrate_entry(
        log_p,
        &applied_by,
        &commit,
        &reason,
        rewritten_ids,
        vec![PATH_RELATIVIZE_SENTINEL.into()],
    )?;
    Ok(migrate.id)
}

fn line_marks_path_relativize(line: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return false;
    };
    if v.get("type").and_then(|x| x.as_str()) != Some("migrate") {
        return false;
    }
    let result = match v.get("result").and_then(|x| x.as_object()) {
        Some(r) => r,
        None => return false,
    };
    result
        .get("created")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().any(|v| v.as_str() == Some(PATH_RELATIVIZE_SENTINEL)))
        .unwrap_or(false)
}

/// Walk a log entry's JSON and rewrite every absolute `file:` value to repo
/// relative form. Returns true if any value was changed.
fn rewrite_paths_in_value(v: &mut serde_json::Value, repo_root: &Path) -> bool {
    let mut changed = false;
    walk_rewrite(v, repo_root, &mut changed);
    changed
}

fn walk_rewrite(v: &mut serde_json::Value, repo_root: &Path, changed: &mut bool) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, inner) in map.iter_mut() {
                if k == "file" {
                    if let Some(s) = inner.as_str() {
                        if looks_absolute(s) {
                            if let Some(rel) = rewrite_absolute_file_value(s, repo_root) {
                                *inner = serde_json::Value::String(rel);
                                *changed = true;
                                continue;
                            }
                        }
                    }
                }
                walk_rewrite(inner, repo_root, changed);
            }
        }
        serde_json::Value::Array(arr) => {
            for x in arr.iter_mut() {
                walk_rewrite(x, repo_root, changed);
            }
        }
        _ => {}
    }
}

/// Rewrite an absolute `file:` value recorded in the log to its repo-relative
/// form. First try stripping the current `repo_root` (for logs produced by the
/// current clone). If that fails — the entry was written in a different clone
/// at a different absolute path — fall back to locating any of the known
/// artifact directory prefixes (`docs/features/`, `docs/adrs/`,
/// `docs/tests/`, `docs/dependencies/`) inside the path string and stripping
/// everything before it. Returns `None` if the path is an escape path with no
/// recognisable artifact-directory segment.
fn rewrite_absolute_file_value(s: &str, repo_root: &Path) -> Option<String> {
    let r = path_relativize(s, repo_root);
    if !r.is_escape {
        return Some(r.value);
    }
    // Fallback: find the first artifact-directory segment in the absolute path.
    const PREFIXES: [&str; 4] = [
        "docs/features/",
        "docs/adrs/",
        "docs/tests/",
        "docs/dependencies/",
    ];
    // Normalise backslashes to forward slashes for the search only — this is
    // purely a string operation against the stored log content.
    let normalised = s.replace('\\', "/");
    for prefix in PREFIXES {
        if let Some(pos) = normalised.find(prefix) {
            return Some(normalised[pos..].to_string());
        }
    }
    None
}
