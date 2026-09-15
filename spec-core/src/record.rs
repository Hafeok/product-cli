//! One act-time record: the file `implement` opens.
//!
//! The record is deliberately dumb data. Everything that decides whether it
//! is *acceptable* lives in `check`, so that opening a record can never be
//! the place a verdict is quietly softened.

use chrono::{DateTime, Utc};
use ledger_core::identity::Identity;
use serde::{Deserialize, Serialize};

use crate::closure::Closure;
use crate::digest::{self, RECORD_FORM};

/// What `implement` opened, plus the closure once a principal files one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActRecord {
    /// Format discriminator; see `docs/spec-flow-store-v1.md`.
    pub form: String,
    /// ULID, minted at open.
    pub id: String,
    /// The flow verb that opened the record. `implement` is the only one v1
    /// defines.
    pub act: String,
    /// The slice built against the specification.
    pub slice: String,
    /// The specification act the slice realises.
    pub act_ref: String,
    pub opened_at: DateTime<Utc>,
    /// The opener. May be a machine — building is the delegable half.
    pub opened_by: Identity,
    /// The commit the work started from.
    pub base_revision: String,
    /// This record's own opening digest.
    pub binds: String,
    /// Absent while the record is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure: Option<Closure>,
}

/// Everything an opening needs, so that [`ActRecord::open`] stays one
/// assignment per field rather than a seven-argument call nobody reads.
#[derive(Debug, Clone)]
pub struct Opening {
    pub id: String,
    pub act: String,
    pub slice: String,
    pub act_ref: String,
    pub opened_at: DateTime<Utc>,
    pub opened_by: Identity,
    pub base_revision: String,
}

impl ActRecord {
    /// Open a record, sealing its opening digest.
    pub fn open(opening: Opening) -> Self {
        let mut record = Self {
            form: RECORD_FORM.to_string(),
            id: opening.id,
            act: opening.act,
            slice: opening.slice,
            act_ref: opening.act_ref,
            opened_at: opening.opened_at,
            opened_by: opening.opened_by,
            base_revision: opening.base_revision,
            binds: String::new(),
            closure: None,
        };
        record.binds = digest::record_digest(&record);
        record
    }

    /// Whether the write-back has not happened yet.
    pub fn is_open(&self) -> bool {
        self.closure.is_none()
    }

    /// The digest this record's opening currently computes to, which is not
    /// necessarily the `binds` it carries — an edited record is exactly the
    /// case the two disagreeing detects.
    pub fn computed_binds(&self) -> String {
        digest::record_digest(self)
    }
}

#[path = "record_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
