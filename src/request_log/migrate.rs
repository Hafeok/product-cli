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
use super::entry::{Entry, EntryPayload, EntryType, MIGRATE_LOG_SENTINEL};
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
    let created = extract_artifact_ids(obj, "created");
    let changed = extract_artifact_ids(obj, "changed");
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

fn extract_artifact_ids(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Vec<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| {
                    v.as_object()
                        .and_then(|m| m.get("id").or_else(|| m.get("ref_name")))
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .or_else(|| v.as_str().map(String::from))
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
