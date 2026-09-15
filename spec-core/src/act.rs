//! A ratified act, plus the refusal that is its peer.
//!
//! An act is what a principal said the codebase does, named by them. It is
//! never what the importer observed: the importer sees an entry point, and an
//! entry point is a transport. The two meet in `map`, and nowhere earlier.

use chrono::{DateTime, Utc};
use ledger_core::canon::{put, put_set};
use ledger_core::hash::domain_hash;
use ledger_core::identity::Identity;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Domain-separation prefix for an act's digest.
pub const ACT_FORM: &str = "spec.act.v1";
/// Domain-separation prefix for a rejection's digest.
pub const REJECTION_FORM: &str = "spec.rejection.v1";

/// One ratified act.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    pub form: String,
    /// The act's address, e.g. `act/settle-basket`.
    pub id: String,
    /// What a principal called it. Never derived from a route.
    pub name: String,
    /// What the act settles. The question a tick cannot answer.
    pub settles: String,
    /// The entry points this act is realised at. Several is a legitimate
    /// answer: transport boundaries are often finer than act boundaries.
    #[serde(default)]
    pub realised_at: Vec<String>,
    pub ratified_by: Identity,
    pub ratified_at: DateTime<Utc>,
    /// The candidate this act was ratified from, when it came from an import.
    /// Absent for an act authored without one — greenfield names acts first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_candidate: Option<String>,
    pub binds: String,
    /// Signature over [`Self::binds`], hex. See [`crate::signing`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Act {
    /// The digest this act's content computes to.
    pub fn computed_binds(&self) -> String {
        let Self {
            form: _,
            id,
            name,
            settles,
            realised_at,
            ratified_by,
            ratified_at,
            from_candidate,
            binds: _,
            // A signature covers the digest; it cannot be inside it.
            signature: _,
        } = self;

        let mut m = Map::new();
        put(&mut m, "from_candidate", from_candidate.clone());
        put(&mut m, "id", Some(id.clone()));
        put(&mut m, "name", Some(name.clone()));
        put(&mut m, "ratified_at", Some(ratified_at.to_rfc3339()));
        put(&mut m, "ratified_by", Some(ratified_by.as_str().to_string()));
        put_set(&mut m, "realised_at", realised_at.iter().cloned());
        put(&mut m, "settles", Some(settles.clone()));
        domain_hash(ACT_FORM, &canonical_bytes(m))
    }

    /// Whether this act claims realisation at a given entry point.
    pub fn realises(&self, entry_point: &str) -> bool {
        self.realised_at.iter().any(|e| e == entry_point)
    }
}

/// A candidate a principal refused, with their reason.
///
/// Filed rather than discarded: a refusal is a decision, and a flow that
/// forgets its refusals re-asks the same question at every import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rejection {
    pub form: String,
    pub candidate: String,
    /// Why. Prose, read-enforced — no instrument can check it, and it exists
    /// so the next reader can see whether the refusal was reasoned.
    pub reason: String,
    pub principal: Identity,
    pub at: DateTime<Utc>,
    pub binds: String,
    /// Signature over [`Self::binds`], hex. See [`crate::signing`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Rejection {
    /// The digest this rejection's content computes to.
    pub fn computed_binds(&self) -> String {
        let Self { form: _, candidate, reason, principal, at, binds: _, signature: _ } = self;
        let mut m = Map::new();
        put(&mut m, "at", Some(at.to_rfc3339()));
        put(&mut m, "candidate", Some(candidate.clone()));
        put(&mut m, "principal", Some(principal.as_str().to_string()));
        put(&mut m, "reason", Some(reason.clone()));
        domain_hash(REJECTION_FORM, &canonical_bytes(m))
    }
}

fn canonical_bytes(map: Map<String, Value>) -> Vec<u8> {
    serde_json::to_vec(&Value::Object(map)).unwrap_or_default()
}

#[path = "act_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
