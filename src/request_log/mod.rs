//! Hash-chained request log (FT-042, ADR-039).
//!
//! `requests.jsonl` at the repository root is the committed, hash-chained,
//! tamper-evident audit trail of every graph mutation. Each entry is one JSON
//! line carrying a canonical-JSON `entry-hash` that chains to the preceding
//! entry's `prev-hash`.
//!
//! This module provides:
//! - Canonical JSON serialisation (deterministic, keys sorted at every level)
//! - Entry construction for all seven entry types
//! - Chain append with hash computation
//! - Chain verification (per-entry + chain + optional git-tag cross-reference)
//! - Replay into a separate directory (never the working tree)
//! - Path migration from `.product/request-log.jsonl`

pub mod canonical;
pub mod entry;
pub mod append;
pub mod verify;
pub mod replay;
pub mod migrate;
pub mod git_identity;

pub use append::{append_entry, compute_entry_id, load_last_entry, GENESIS_PREV_HASH};
pub use canonical::canonical_json;
pub use entry::{Entry, EntryPayload, EntryType, MIGRATE_LOG_SENTINEL};
pub use migrate::migrate_if_needed;
pub use replay::{replay_full, replay_to, ReplayOptions};
pub use verify::{verify_log, VerifyFinding, VerifyOutcome, VerifyOptions};

use std::path::{Path, PathBuf};

/// Default committed log filename (ADR-039).
pub const DEFAULT_LOG_FILENAME: &str = "requests.jsonl";
/// Legacy gitignored path used in FT-041.
pub const LEGACY_LOG_PATH: &str = ".product/request-log.jsonl";

/// Resolve the log path for a given repository root.
///
/// If `product.toml` declares `[paths].requests`, honour that; otherwise use
/// `requests.jsonl` at the root.
pub fn log_path(repo_root: &Path, requests_rel: Option<&str>) -> PathBuf {
    match requests_rel {
        Some(p) if !p.is_empty() => repo_root.join(p),
        _ => repo_root.join(DEFAULT_LOG_FILENAME),
    }
}

/// Legacy path used by FT-041.
pub fn legacy_log_path(repo_root: &Path) -> PathBuf {
    repo_root.join(LEGACY_LOG_PATH)
}
