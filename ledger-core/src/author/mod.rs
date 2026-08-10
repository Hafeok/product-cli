//! The L1 authoring verbs — thin shells over the gate.
//!
//! Every verb builds the change-set (or set file) it intends to append, runs
//! the *same* [`crate::verify::verify`] pass the CI gate runs over the store
//! as it would look afterwards, and refuses to write if any finding appears
//! that the store did not already carry. There is no second copy of the
//! rules anywhere in this module: a verb can only refuse for exactly what
//! `verify` would report, one command later, with the same class and the
//! same message. The one sanctioned exception is `add` without an
//! allocation, which is the enumerated-but-unallocated intermediate state
//! the format explicitly represents (`L001` is then the gate doing its job,
//! not a write fault).
//!
//! The log stays append-only: verbs only ever create new files
//! (`log/<ulid>.yml`, `sets/<id>.yml`), never edit one.

mod acceptance_ops;
mod declare;
mod decision;
mod report;
mod version_ops;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use acceptance_ops::{AcceptArgs, RevokeArgs};
pub use declare::DeclareArgs;
pub use decision::{AddArgs, AllocationArgs};
pub use report::{blame, log, status};
pub use version_ops::{EscapeArgs, ReviseArgs, SupersedeArgs};

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};

use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::identity::Identity;
use crate::mint::UlidMint;
use crate::store::{self, LoggedChangeSet, Store};
use crate::verify::{self, Options};

/// Why an authoring verb did not write.
#[derive(Debug)]
pub enum AuthorError {
    /// The write would introduce these findings — the same classes and
    /// messages `ledger verify` would report one command later.
    Refused(Vec<Finding>),
    /// A single-writer conflict: the store moved under the verb (revise off
    /// a stale parent, a duplicate live acceptance, a chained supersession).
    Conflict(String),
    /// The input does not name things the store knows.
    Usage(String),
    /// The filesystem or git said no.
    Io(String),
}

impl fmt::Display for AuthorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(findings) => {
                writeln!(f, "refused — the write would fail verify with:")?;
                for finding in findings {
                    writeln!(f, "  - {finding}")?;
                }
                Ok(())
            }
            Self::Conflict(m) | Self::Usage(m) | Self::Io(m) => f.write_str(m),
        }
    }
}

/// What a verb wrote, for the CLI to report.
#[derive(Debug)]
pub struct Applied {
    /// The file the act landed in.
    pub path: PathBuf,
    /// Human-readable lines describing what happened.
    pub lines: Vec<String>,
}

/// One authoring session: who is acting, when, and how ids are minted.
/// Time and identity are injected so every verb is testable without a clock
/// or a global git config.
pub struct Author {
    pub root: PathBuf,
    pub who: Identity,
    pub now: DateTime<Utc>,
    pub(crate) mint: UlidMint,
}

impl Author {
    /// An author acting as `who` at `now`, minting from `mint`.
    pub fn new(root: &Path, who: Identity, now: DateTime<Utc>, mint: UlidMint) -> Self {
        Self { root: root.to_path_buf(), who, now, mint }
    }

    /// The wall-clock author: identity from git config (OD-3), system mint.
    pub fn open(root: &Path) -> Result<Self, AuthorError> {
        let who = crate::whoami::git_identity(root).map_err(AuthorError::Io)?;
        Ok(Self::new(root, who, Utc::now(), UlidMint::system()))
    }

    /// The date write-time expiry rules are judged against.
    pub(crate) fn today(&self) -> NaiveDate {
        self.now.date_naive()
    }

    /// Load the store as it stands on disk.
    pub(crate) fn load(&self) -> Store {
        store::load(&self.root)
    }

    /// The gate, exactly as `verify` runs it at write time: every class, no
    /// blame pass (an uncommitted acceptance is always skipped by `L009`,
    /// so consulting git here could never produce a finding).
    fn gate(&self, store: &Store) -> Vec<Finding> {
        verify::verify(store, &Options { gate: None, today: self.today(), blame: false }).findings
    }

    /// Refuse `candidate` if appending it would introduce findings the
    /// store does not already carry. `allowed` filters findings that are the
    /// sanctioned intermediate state of the verb (only `add` uses it).
    pub(crate) fn refusal_check(
        &self,
        current: &Store,
        candidate: &ChangeSet,
        allowed: impl Fn(&Finding) -> bool,
    ) -> Result<(), AuthorError> {
        let baseline = self.gate(current);
        let mut with = Store {
            root: current.root.clone(),
            dir: current.dir.clone(),
            sets: current.sets.clone(),
            log: current.log.to_vec(),
            schema_findings: current.schema_findings.clone(),
        };
        with.log.push(LoggedChangeSet {
            path: current.dir.join("log").join(candidate.file_name()),
            file: candidate.clone(),
        });
        let fresh: Vec<Finding> = self
            .gate(&with)
            .into_iter()
            .filter(|f| !baseline.contains(f))
            .filter(|f| !allowed(f))
            .collect();
        if fresh.is_empty() {
            Ok(())
        } else {
            Err(AuthorError::Refused(fresh))
        }
    }

    /// Append a change-set as a brand-new log file. Never overwrites: the
    /// log is append-only, and a colliding ULID is an error, not a merge.
    pub(crate) fn append(&self, candidate: &ChangeSet) -> Result<PathBuf, AuthorError> {
        let path = self.root.join(crate::STORE_DIR).join("log").join(candidate.file_name());
        if path.exists() {
            return Err(AuthorError::Io(format!(
                "{} already exists — a log file is written once and never edited",
                path.display()
            )));
        }
        let text = serde_yaml::to_string(candidate)
            .map_err(|e| AuthorError::Io(format!("could not serialise the change-set: {e}")))?;
        product_core::fileops::write_file_atomic(&path, &text)
            .map_err(|e| AuthorError::Io(e.to_string()))?;
        Ok(path)
    }

    /// A fresh change-set shell for one act, attributed to this author.
    pub(crate) fn shell(&mut self, note: Option<String>) -> Result<ChangeSet, AuthorError> {
        Ok(ChangeSet {
            format: crate::format::CURRENT_FORMAT,
            id: self.mint.mint_id("cs").map_err(AuthorError::Io)?,
            created_at: self.now,
            created_by: self.who.clone(),
            parents: Vec::new(),
            note,
            decisions: Vec::new(),
            versions: Vec::new(),
            acceptances: Vec::new(),
            revocations: Vec::new(),
        })
    }
}
