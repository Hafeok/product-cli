//! The on-disk act-time record store.
//!
//! One writer, one law: every verb here builds the record it would write,
//! judges it with `check`, then refuses the write if a finding appears. There
//! is no second validation copy, so a rule cannot hold at the gate while
//! being softer at the door.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use ledger_core::identity::Identity;
use product_core::error::{ProductError, Result};
use product_core::fileops::write_file_atomic;

use crate::check::{self, Finding};
use crate::closure::{Closure, ClosureKind};
use crate::digest::closure_digest;
use crate::record::{ActRecord, Opening};

/// The records directory under a repo root.
pub fn records_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".spec").join("records")
}

/// Where one record lives.
pub fn record_path(repo_root: &Path, id: &str) -> PathBuf {
    records_dir(repo_root).join(format!("{id}.yml"))
}

/// Open a record, sealing it to disk.
pub fn open(repo_root: &Path, opening: Opening) -> Result<ActRecord> {
    let record = ActRecord::open(opening);
    write(repo_root, &record)?;
    Ok(record)
}

/// What closing a record did.
///
/// A refusal is not an error: the store was readable, the request was
/// well-formed, and the gate said no. Keeping the two apart is what lets the
/// caller exit `1` for a finding and `2` only when it genuinely could not run.
#[derive(Debug, Clone)]
pub enum Closed {
    /// The closure was sealed to disk.
    Sealed(Box<ActRecord>),
    /// The write would have introduced these findings, so it did not happen.
    Refused(Vec<Finding>),
}

/// Close a record. Refuses any closure that would introduce a finding, with
/// the same class and the same message the gate would report.
pub fn close(
    repo_root: &Path,
    id: &str,
    closing: Closing,
    signer: Option<&ed25519_dalek::SigningKey>,
) -> Result<Closed> {
    let mut record = load(&record_path(repo_root, id))?;
    if let Some(existing) = &record.closure {
        return Err(ProductError::ConfigError(format!(
            "{id} was already closed by {} — a correction is a new record",
            existing.principal.as_str()
        )));
    }
    record.closure = Some(seal(&record, closing, signer));
    let trust = crate::signing::load_trust(repo_root)?;
    let mut findings = check::judge(&record);
    if let Some(closure) = &record.closure {
        findings.extend(crate::gate::judge_signature(
            &record.id,
            &closure.binds,
            &closure.principal,
            closure.signature.as_deref(),
            &trust,
        ));
    }
    if !findings.is_empty() {
        return Ok(Closed::Refused(findings));
    }
    write(repo_root, &record)?;
    Ok(Closed::Sealed(Box::new(record)))
}

/// Every record in the store, ordered by id so a run is reproducible.
pub fn load_all(repo_root: &Path) -> Result<Vec<ActRecord>> {
    let dir = records_dir(repo_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", dir.display())))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yml"))
        .collect();
    paths.sort();
    paths.iter().map(|p| load(p)).collect()
}

/// Everything the flow has filed, plus the last import's projection.
///
/// One load, so every verb judges the same store. A verb that assembled its
/// own view could disagree with the gate about what exists, which is the
/// class of bug that makes a gate untrustworthy rather than merely wrong.
pub fn load_store(repo_root: &Path) -> Result<crate::gate::SpecStore> {
    Ok(crate::gate::SpecStore {
        records: load_all(repo_root)?,
        acts: crate::ratify::load_acts(repo_root)?,
        rejections: crate::ratify::load_rejections(repo_root)?,
        inventory: crate::inventory::Inventory::load_opt(repo_root)?,
        policy_versions: crate::policy::load_versions(repo_root)?,
        trust: crate::signing::load_trust(repo_root)?,
    })
}

/// Read one record.
pub fn load(path: &Path) -> Result<ActRecord> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", path.display())))?;
    serde_yaml::from_str(&text).map_err(|e| ProductError::ParseError {
        file: path.to_path_buf(),
        line: e.location().map(|l| l.line()),
        message: e.to_string(),
    })
}

/// What a caller supplies to close a record.
#[derive(Debug, Clone)]
pub struct Closing {
    pub kind: ClosureKind,
    pub principal: Identity,
    pub at: DateTime<Utc>,
    pub determinations: Vec<String>,
}

fn seal(
    record: &ActRecord,
    closing: Closing,
    signer: Option<&ed25519_dalek::SigningKey>,
) -> Closure {
    let mut closure = Closure {
        kind: closing.kind,
        principal: closing.principal,
        at: closing.at,
        determinations: closing.determinations,
        binds: String::new(),
        signature: None,
    };
    closure.binds = closure_digest(&record.computed_binds(), &closure);
    closure.signature = crate::signing::sign_if_keyed(&closure.binds, signer);
    closure
}

fn write(repo_root: &Path, record: &ActRecord) -> Result<()> {
    let path = record_path(repo_root, &record.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ProductError::WriteError { path: parent.to_path_buf(), message: e.to_string() })?;
    }
    let body = serde_yaml::to_string(record)
        .map_err(|e| ProductError::WriteError { path: path.clone(), message: e.to_string() })?;
    write_file_atomic(&path, &body)
}

#[path = "store_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
