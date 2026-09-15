//! Ratifying a candidate, or refusing it.
//!
//! Per candidate, one at a time, always naming a principal. Batching the
//! *sitting* is fine and expected — sixty-eight candidates can be reviewed at
//! a desk in an hour. Batching the *decision* is what this module refuses:
//! each act carries its own principal, its own reason, its own digest.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use ledger_core::identity::Identity;
use product_core::error::{ProductError, Result};
use product_core::fileops::write_file_atomic;

use crate::act::{Act, Rejection, ACT_FORM, REJECTION_FORM};
use crate::check::Finding;

/// Where ratified acts live under a repo root.
pub fn acts_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".spec").join("acts")
}

/// Where refusals live under a repo root.
pub fn rejections_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".spec").join("rejections")
}

/// What a caller supplies to ratify a candidate.
///
/// The three fields are the ones a tick cannot supply, which is the whole
/// reason they are required: ratification is a judgement, and a judgement that
/// needs no words is a judgement nobody made.
#[derive(Debug, Clone)]
pub struct Ratification {
    pub act_id: String,
    pub name: String,
    pub settles: String,
    pub principal: Identity,
    pub at: DateTime<Utc>,
    pub realised_at: Vec<String>,
    pub from_candidate: Option<String>,
}

/// What a caller supplies to refuse one.
#[derive(Debug, Clone)]
pub struct Refusal {
    pub candidate: String,
    pub reason: String,
    pub principal: Identity,
    pub at: DateTime<Utc>,
}

/// What ratifying did. A refusal by the gate is an answer, not an error.
#[derive(Debug, Clone)]
pub enum Ratified {
    Filed(Box<Act>),
    Refused(Vec<Finding>),
}

/// What refusing did.
#[derive(Debug, Clone)]
pub enum Refused {
    Filed(Box<Rejection>),
    Blocked(Vec<Finding>),
}

/// Ratify a candidate as an act.
pub fn accept(repo_root: &Path, ratification: Ratification) -> Result<Ratified> {
    let path = acts_dir(repo_root).join(format!("{}.yml", slug(&ratification.act_id)));
    if path.exists() {
        return Err(ProductError::ConfigError(format!(
            "{} is already ratified — a correction supersedes, it does not overwrite",
            ratification.act_id
        )));
    }

    let mut act = Act {
        form: ACT_FORM.to_string(),
        id: ratification.act_id,
        name: ratification.name,
        settles: ratification.settles,
        realised_at: ratification.realised_at,
        ratified_by: ratification.principal,
        ratified_at: ratification.at,
        from_candidate: ratification.from_candidate,
        binds: String::new(),
    };
    act.binds = act.computed_binds();

    let findings = crate::gate::judge_act(&act);
    if !findings.is_empty() {
        return Ok(Ratified::Refused(findings));
    }
    write_yaml(&path, &act)?;
    Ok(Ratified::Filed(Box::new(act)))
}

/// Refuse a candidate, filing the reason.
pub fn reject(repo_root: &Path, refusal: Refusal) -> Result<Refused> {
    let path = rejections_dir(repo_root).join(format!("{}.yml", slug(&refusal.candidate)));
    if path.exists() {
        return Err(ProductError::ConfigError(format!(
            "{} was already refused — reopening it is a fresh review",
            refusal.candidate
        )));
    }

    let mut rejection = Rejection {
        form: REJECTION_FORM.to_string(),
        candidate: refusal.candidate,
        reason: refusal.reason,
        principal: refusal.principal,
        at: refusal.at,
        binds: String::new(),
    };
    rejection.binds = rejection.computed_binds();

    let findings = crate::gate::judge_rejection(&rejection);
    if !findings.is_empty() {
        return Ok(Refused::Blocked(findings));
    }
    write_yaml(&path, &rejection)?;
    Ok(Refused::Filed(Box::new(rejection)))
}

/// Every ratified act, ordered by id.
pub fn load_acts(repo_root: &Path) -> Result<Vec<Act>> {
    load_dir(&acts_dir(repo_root))
}

/// Every filed refusal, ordered by candidate.
pub fn load_rejections(repo_root: &Path) -> Result<Vec<Rejection>> {
    load_dir(&rejections_dir(repo_root))
}

pub(crate) fn load_dir<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", dir.display())))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yml"))
        .collect();
    paths.sort();
    paths.iter().map(|p| read_yaml(p)).collect()
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", path.display())))?;
    serde_yaml::from_str(&text).map_err(|e| ProductError::ParseError {
        file: path.to_path_buf(),
        line: e.location().map(|l| l.line()),
        message: e.to_string(),
    })
}

pub(crate) fn write_yaml<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ProductError::WriteError {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }
    let body = serde_yaml::to_string(value).map_err(|e| ProductError::WriteError {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    write_file_atomic(path, &body)
}

/// A filesystem-safe form of an address, so `act/settle-basket` is one file.
pub fn slug(id: &str) -> String {
    let mapped: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    mapped.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-")
}

#[path = "ratify_tests.rs"]
#[cfg(test)]
mod tests;
