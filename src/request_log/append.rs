//! Log append: compute IDs, link chain, write one line atomically (FT-042).

use super::entry::Entry;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Genesis `prev-hash` sentinel (ADR-039 decision 2).
pub const GENESIS_PREV_HASH: &str = "0000000000000000";

/// Load the last entry in the log, if any.
pub fn load_last_entry(log_path: &Path) -> Option<Entry> {
    let file = std::fs::File::open(log_path).ok()?;
    let reader = BufReader::new(file);
    let mut last_line: Option<String> = None;
    for line in reader.lines().map_while(Result::ok) {
        if !line.trim().is_empty() {
            last_line = Some(line);
        }
    }
    let line = last_line?;
    Entry::parse_line(&line).ok().map(|(e, _)| e)
}

/// One entry-or-error with its line number.
pub type EntryLine = Result<(usize, Entry), (usize, String)>;

/// Load every entry in the log, in order (best-effort — malformed lines are
/// returned as errors alongside their line number).
pub fn load_all_entries(log_path: &Path) -> std::io::Result<Vec<EntryLine>> {
    let file = std::fs::File::open(log_path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for (i, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        if line.trim().is_empty() {
            continue;
        }
        match Entry::parse_line(&line) {
            Ok((e, _)) => out.push(Ok((i + 1, e))),
            Err(err) => out.push(Err((i + 1, err))),
        }
    }
    Ok(out)
}

/// Compute the next entry ID: `req-{YYYYMMDD}-{NNN}` where NNN increments
/// within the UTC day, resetting at UTC midnight (ADR-039 decision 7).
pub fn compute_entry_id(applied_at: &str, log_path: &Path) -> String {
    // YYYY-MM-DDTHH:MM:SS... → YYYYMMDD
    let date_prefix: String = applied_at
        .chars()
        .take(10)
        .filter(|c| c.is_ascii_digit())
        .collect();
    let mut seq: u32 = 1;
    if let Ok(file) = std::fs::File::open(log_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok((e, _)) = Entry::parse_line(&line) {
                // id is req-YYYYMMDD-NNN
                if let Some(rest) = e.id.strip_prefix("req-") {
                    if let Some((date_part, n_part)) = rest.split_once('-') {
                        if date_part == date_prefix {
                            if let Ok(n) = n_part.parse::<u32>() {
                                if n >= seq {
                                    seq = n + 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    format!("req-{}-{:03}", date_prefix, seq)
}

/// Append a fully-built entry (with ID + prev-hash + payload set) to the log.
/// Computes the entry hash and writes one canonical-JSON line.
pub fn append_entry(log_path: &Path, mut entry: Entry) -> std::io::Result<Entry> {
    if let Some(parent) = log_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    // Compute entry hash against canonical form with empty entry-hash.
    entry.entry_hash = entry.compute_hash();
    let line = entry.canonical_line();
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(f, "{}", line)?;
    Ok(entry)
}

/// Next prev-hash for a new entry — the preceding entry's `entry-hash`, or the
/// genesis sentinel if the log is empty.
pub fn next_prev_hash(log_path: &Path) -> String {
    load_last_entry(log_path)
        .map(|e| e.entry_hash)
        .unwrap_or_else(|| GENESIS_PREV_HASH.to_string())
}

/// Parameters for `append_apply_entry`.
pub struct ApplyEntryParams<'a> {
    pub entry_type: super::entry::EntryType,
    pub applied_by: &'a str,
    pub commit: &'a str,
    pub reason: &'a str,
    pub request_json: serde_json::Value,
    pub created: Vec<String>,
    pub changed: Vec<String>,
}

/// Shortcut: append an Apply-style entry (`create` / `change` /
/// `create-and-change`) to the log, looking up the previous hash and assigning
/// an ID automatically.
pub fn append_apply_entry(
    log_path: &Path,
    params: ApplyEntryParams<'_>,
) -> std::io::Result<Entry> {
    let ApplyEntryParams {
        entry_type,
        applied_by,
        commit,
        reason,
        request_json,
        created,
        changed,
    } = params;
    let applied_at = chrono_now();
    let id = compute_entry_id(&applied_at, log_path);
    let prev_hash = next_prev_hash(log_path);
    let entry = Entry {
        id,
        applied_at,
        applied_by: applied_by.into(),
        commit: commit.into(),
        entry_type,
        reason: reason.into(),
        prev_hash,
        entry_hash: "".into(),
        payload: super::entry::EntryPayload::Apply {
            request: request_json,
            created,
            changed,
        },
    };
    append_entry(log_path, entry)
}

/// Parameters for `append_verify_entry`.
pub struct VerifyEntryParams<'a> {
    pub applied_by: &'a str,
    pub commit: &'a str,
    pub reason: &'a str,
    pub feature: &'a str,
    pub tcs_run: Vec<String>,
    pub passing: Vec<String>,
    pub failing: Vec<String>,
    pub tag_created: Option<String>,
}

/// Shortcut: append a `verify` entry.
pub fn append_verify_entry(
    log_path: &Path,
    params: VerifyEntryParams<'_>,
) -> std::io::Result<Entry> {
    let VerifyEntryParams {
        applied_by,
        commit,
        reason,
        feature,
        tcs_run,
        passing,
        failing,
        tag_created,
    } = params;
    let applied_at = chrono_now();
    let id = compute_entry_id(&applied_at, log_path);
    let prev_hash = next_prev_hash(log_path);
    let entry = Entry {
        id,
        applied_at,
        applied_by: applied_by.into(),
        commit: commit.into(),
        entry_type: super::entry::EntryType::Verify,
        reason: reason.into(),
        prev_hash,
        entry_hash: "".into(),
        payload: super::entry::EntryPayload::Verify {
            feature: feature.into(),
            tcs_run,
            passing,
            failing,
            tag_created,
        },
    };
    append_entry(log_path, entry)
}

/// Shortcut: append a `migrate` entry (for document migration).
pub fn append_migrate_entry(
    log_path: &Path,
    applied_by: &str,
    commit: &str,
    reason: &str,
    sources: Vec<String>,
    created: Vec<String>,
) -> std::io::Result<Entry> {
    let applied_at = chrono_now();
    let id = compute_entry_id(&applied_at, log_path);
    let prev_hash = next_prev_hash(log_path);
    let entry = Entry {
        id,
        applied_at,
        applied_by: applied_by.into(),
        commit: commit.into(),
        entry_type: super::entry::EntryType::Migrate,
        reason: reason.into(),
        prev_hash,
        entry_hash: "".into(),
        payload: super::entry::EntryPayload::Migrate { sources, created },
    };
    append_entry(log_path, entry)
}

/// Shortcut: append a `schema-upgrade` entry.
pub fn append_schema_upgrade_entry(
    log_path: &Path,
    applied_by: &str,
    commit: &str,
    reason: &str,
    from_version: u32,
    to_version: u32,
    changes: &str,
) -> std::io::Result<Entry> {
    let applied_at = chrono_now();
    let id = compute_entry_id(&applied_at, log_path);
    let prev_hash = next_prev_hash(log_path);
    let entry = Entry {
        id,
        applied_at,
        applied_by: applied_by.into(),
        commit: commit.into(),
        entry_type: super::entry::EntryType::SchemaUpgrade,
        reason: reason.into(),
        prev_hash,
        entry_hash: "".into(),
        payload: super::entry::EntryPayload::SchemaUpgrade {
            from_version,
            to_version,
            changes: changes.into(),
        },
    };
    append_entry(log_path, entry)
}

/// Shortcut: append an `undo` entry that reverses another entry.
pub fn append_undo_entry(
    log_path: &Path,
    applied_by: &str,
    commit: &str,
    reason: &str,
    undoes: &str,
    inverse_request: serde_json::Value,
) -> std::io::Result<Entry> {
    let applied_at = chrono_now();
    let id = compute_entry_id(&applied_at, log_path);
    let prev_hash = next_prev_hash(log_path);
    let entry = Entry {
        id,
        applied_at,
        applied_by: applied_by.into(),
        commit: commit.into(),
        entry_type: super::entry::EntryType::Undo,
        reason: reason.into(),
        prev_hash,
        entry_hash: "".into(),
        payload: super::entry::EntryPayload::Undo {
            undoes: undoes.into(),
            inverse_request,
        },
    };
    append_entry(log_path, entry)
}

fn chrono_now() -> String {
    // Allow `PRODUCT_LOG_NOW` as an override for deterministic testing.
    if let Ok(v) = std::env::var("PRODUCT_LOG_NOW") {
        if !v.is_empty() {
            return v;
        }
    }
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
